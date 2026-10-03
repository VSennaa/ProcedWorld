//! HTTP routes and the WebSocket session loop.

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Json, Router,
};
use pw_engine::{
    ids::CivId,
    world::{CommandLog, WorldId, WorldSnapshot},
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::{
    config::ServerConfig,
    protocol::{error_frame, frame, CreateWorld, Envelope, ErrorReason, Join, SubmitCommand, PROTOCOL_VERSION},
    store::{StoreError, WorldRecord, WorldStore},
    view::state_hash_string,
    world::{Conn, LiveWorld},
};

/// Largest accepted WebSocket message, in bytes.
const MAX_MESSAGE_BYTES: usize = 64 * 1024;
const OUTBOX_CAPACITY: usize = 64;

type SharedWorld = Arc<Mutex<LiveWorld>>;

pub struct AppState {
    pub config: ServerConfig,
    store: Arc<dyn WorldStore>,
    worlds: Mutex<HashMap<u64, SharedWorld>>,
    next_conn: AtomicU64,
}

impl AppState {
    pub fn new(config: ServerConfig, store: Arc<dyn WorldStore>) -> Self {
        Self { config, store, worlds: Mutex::new(HashMap::new()), next_conn: AtomicU64::new(1) }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn build_app(state: Arc<AppState>) -> Router {
    Router::new().route("/health", get(health)).route("/ws", get(ws_handler)).with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "protocol_version": PROTOCOL_VERSION }))
}

async fn ws_handler(State(app): State<Arc<AppState>>, upgrade: WebSocketUpgrade) -> Response {
    upgrade
        .max_message_size(MAX_MESSAGE_BYTES)
        .max_frame_size(MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| connection(app, socket))
}

struct Bound {
    world: SharedWorld,
    civ: CivId,
}

async fn connection(app: Arc<AppState>, mut socket: WebSocket) {
    let conn_id = app.next_conn.fetch_add(1, Ordering::Relaxed);
    let (tx, mut rx) = mpsc::channel::<String>(OUTBOX_CAPACITY);
    let mut bound: Option<Bound> = None;
    'session: loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(text))) => {
                    for reply in handle(&app, &mut bound, conn_id, &tx, &text).await {
                        if socket.send(Message::Text(reply)).await.is_err() {
                            break 'session;
                        }
                    }
                }
                Some(Ok(Message::Binary(_))) => {
                    let reply = error_frame(None, ErrorReason::MalformedMessage, "binary frames are not supported", None);
                    if socket.send(Message::Text(reply)).await.is_err() {
                        break 'session;
                    }
                }
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break 'session,
                Some(Ok(_)) => {}
            },
            Some(push) = rx.recv() => {
                if socket.send(Message::Text(push)).await.is_err() {
                    break 'session;
                }
            }
        }
    }
    if let Some(bound) = bound {
        on_disconnect(&app, bound, conn_id);
    }
}

/// Starts the technical reconnect grace (ADR-0008). It is not a game clock: it only decides when a
/// vanished connection stops holding the turn and the Governor plays for that civilization.
fn on_disconnect(app: &Arc<AppState>, bound: Bound, conn_id: u64) {
    let civ = bound.civ;
    let generation = lock(&bound.world).disconnect(civ, conn_id);
    let Some(generation) = generation else { return };
    let grace = app.config.reconnect_grace;
    let world = bound.world;
    tokio::spawn(async move {
        tokio::time::sleep(grace).await;
        let mut world = lock(&world);
        if world.grace_expired(civ, generation) {
            if let Ok(false) = world.try_advance() {
                let payload = world.ready_payload();
                world.broadcast("ready_state", &payload, None);
            }
        }
    });
}

fn decode<T: DeserializeOwned>(payload: Value) -> Result<T, String> {
    serde_json::from_value(payload).map_err(|error| error.to_string())
}

async fn handle(app: &Arc<AppState>, bound: &mut Option<Bound>, conn_id: u64, tx: &mpsc::Sender<String>, text: &str) -> Vec<String> {
    let envelope: Envelope = match serde_json::from_str(text) {
        Ok(envelope) => envelope,
        Err(error) => {
            // A well-formed JSON object with a different protocol_version must report the mismatch.
            let version = serde_json::from_str::<Value>(text).ok().and_then(|v| v.get("protocol_version").and_then(Value::as_u64));
            if matches!(version, Some(v) if v != u64::from(PROTOCOL_VERSION)) {
                return vec![version_mismatch(None)];
            }
            return vec![error_frame(None, ErrorReason::MalformedMessage, &error.to_string(), None)];
        }
    };
    let rid = envelope.request_id.as_deref();
    if envelope.protocol_version != PROTOCOL_VERSION {
        return vec![version_mismatch(rid)];
    }
    let fail = |reason, detail: &str| vec![error_frame(rid, reason, detail, None)];
    match envelope.kind.as_str() {
        "create_world" => {
            let params: CreateWorld = match decode(envelope.payload) {
                Ok(params) => params,
                Err(detail) => return fail(ErrorReason::MalformedMessage, &detail),
            };
            create_world(app, rid, params).await
        }
        "join" => {
            let params: Join = match decode(envelope.payload) {
                Ok(params) => params,
                Err(detail) => return fail(ErrorReason::MalformedMessage, &detail),
            };
            if bound.is_some() {
                return fail(ErrorReason::AlreadyJoined, "this connection already holds a civilization");
            }
            let Some(world) = lock(&app.worlds).get(&params.world_id).cloned() else {
                return fail(ErrorReason::WorldNotFound, "no such world");
            };
            let civ = CivId(params.civ);
            let replies = {
                let mut live = lock(&world);
                match live.join(civ, params.session_token.as_deref(), Conn { id: conn_id, tx: tx.clone() }) {
                    Err(reason) => return fail(reason, "join refused"),
                    Ok(token) => {
                        let ready = live.ready_payload();
                        live.broadcast("ready_state", &ready, Some(conn_id));
                        vec![
                            frame(rid, "joined", json!({ "world_id": params.world_id, "civ": civ, "session_token": token, "turn": live.state().turn })),
                            frame(rid, "state_snapshot", live.snapshot_for(civ)),
                            frame(rid, "ready_state", ready),
                        ]
                    }
                }
            };
            *bound = Some(Bound { world, civ });
            replies
        }
        "get_snapshot" => match bound {
            Some(b) => vec![frame(rid, "state_snapshot", lock(&b.world).snapshot_for(b.civ))],
            None => fail(ErrorReason::NotJoined, "join first"),
        },
        "submit_command" => {
            let params: SubmitCommand = match decode(envelope.payload) {
                Ok(params) => params,
                Err(detail) => return fail(ErrorReason::MalformedMessage, &detail),
            };
            let Some(b) = bound else { return fail(ErrorReason::NotJoined, "join first") };
            match lock(&b.world).submit(b.civ, params.command) {
                Ok(command) => vec![frame(
                    rid,
                    "command_accepted",
                    json!({ "command_id": command.command_id, "accepted_sequence": command.accepted_sequence, "turn": command.turn }),
                )],
                Err(error) => vec![error_frame(rid, error.reason, &error.detail, error.engine_reason)],
            }
        }
        "ready" | "unready" => {
            let Some(b) = bound else { return fail(ErrorReason::NotJoined, "join first") };
            let mut live = lock(&b.world);
            if let Err(reason) = live.set_ready(b.civ, envelope.kind == "ready") {
                return fail(reason, "no active seat");
            }
            let advanced = match live.try_advance() {
                Ok(advanced) => advanced,
                Err(reason) => return fail(reason, "turn resolution failed"),
            };
            let payload = live.ready_payload();
            if !advanced {
                live.broadcast("ready_state", &payload, Some(conn_id));
            }
            vec![frame(rid, "ready_state", payload)]
        }
        _ => fail(ErrorReason::UnknownMessageType, &envelope.kind),
    }
}

fn version_mismatch(rid: Option<&str>) -> String {
    error_frame(rid, ErrorReason::ProtocolVersionMismatch, &format!("this server speaks protocol_version {PROTOCOL_VERSION}"), None)
}

async fn create_world(app: &Arc<AppState>, rid: Option<&str>, params: CreateWorld) -> Vec<String> {
    let fail = |reason, detail: &str| vec![error_frame(rid, reason, detail, None)];
    if params.civs == 0 || params.civs > app.config.max_civs {
        return fail(ErrorReason::InvalidWorldParams, "civs out of range");
    }
    {
        let worlds = lock(&app.worlds);
        if worlds.contains_key(&params.seed) {
            return fail(ErrorReason::WorldExists, "a world with this seed already exists");
        }
        if worlds.len() >= app.config.max_worlds {
            return fail(ErrorReason::TooManyWorlds, "world limit reached");
        }
    }
    let (seed, civs) = (params.seed, params.civs);
    let built = match tokio::task::spawn_blocking(move || pw_harness::initial_world(seed, civs)).await {
        Ok(Ok(built)) => built,
        Ok(Err(detail)) => return fail(ErrorReason::InvalidWorldParams, &detail),
        Err(_) => return fail(ErrorReason::Internal, "world generation failed"),
    };
    let (state, versions, homes) = built;
    let record = WorldRecord {
        snapshot: WorldSnapshot::new(state.clone(), versions.clone()),
        log: CommandLog::default(),
        home_tiles: homes.clone(),
        state: state.clone(),
    };
    let world_id: WorldId = state.world_id;
    let mut worlds = lock(&app.worlds);
    if worlds.contains_key(&params.seed) {
        return fail(ErrorReason::WorldExists, "a world with this seed already exists");
    }
    if worlds.len() >= app.config.max_worlds {
        return fail(ErrorReason::TooManyWorlds, "world limit reached");
    }
    match app.store.create(record) {
        Ok(()) => {}
        Err(StoreError::AlreadyExists) => return fail(ErrorReason::WorldExists, "a world with this seed already exists"),
        Err(StoreError::NotFound) => return fail(ErrorReason::Internal, "store unavailable"),
    }
    let live = LiveWorld::new(state.clone(), versions, homes, app.store.clone());
    let civs = live.civs();
    worlds.insert(params.seed, Arc::new(Mutex::new(live)));
    vec![frame(rid, "world_created", json!({ "world_id": world_id, "civs": civs, "turn": state.turn, "state_hash": state_hash_string(&state) }))]
}
