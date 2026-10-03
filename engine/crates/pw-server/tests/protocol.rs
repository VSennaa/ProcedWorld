//! End-to-end tests: a real server on a loopback ephemeral port and real WebSocket clients.

use std::{net::SocketAddr, sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use pw_engine::{
    ids::{CivId, TurnNumber},
    world::{replay, CommandOrigin, WorldId},
};
use pw_server::{build_app, AppState, InMemoryStore, ServerConfig, WorldStore};
use serde_json::{json, Value};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn start(grace: Duration) -> (SocketAddr, Arc<InMemoryStore>) {
    let store = Arc::new(InMemoryStore::default());
    let config = ServerConfig { reconnect_grace: grace, ..ServerConfig::default() };
    let app = build_app(Arc::new(AppState::new(config, store.clone())));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (addr, store)
}

async fn connect(addr: SocketAddr) -> Ws {
    connect_async(format!("ws://{addr}/ws")).await.unwrap().0
}

async fn next_frame(ws: &mut Ws) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(20), ws.next()).await.expect("timed out").expect("closed").unwrap();
        if let Message::Text(text) = message {
            return serde_json::from_str(&text).unwrap();
        }
    }
}

/// Sends a request and returns the first frame that echoes its request id.
async fn request(ws: &mut Ws, kind: &str, payload: Value) -> Value {
    let rid = format!("r-{kind}");
    let frame = json!({ "protocol_version": 1, "request_id": rid, "type": kind, "payload": payload });
    ws.send(Message::Text(frame.to_string())).await.unwrap();
    loop {
        let reply = next_frame(ws).await;
        if reply["request_id"] == rid.as_str() {
            return reply;
        }
    }
}

/// Waits for an unsolicited frame of the given type.
async fn push_of(ws: &mut Ws, kind: &str) -> Value {
    loop {
        let frame = next_frame(ws).await;
        if frame["type"] == kind {
            return frame["payload"].clone();
        }
    }
}

async fn create(ws: &mut Ws, seed: u64, civs: u32) {
    let reply = request(ws, "create_world", json!({ "seed": seed, "civs": civs })).await;
    assert_eq!(reply["type"], "world_created", "{reply}");
}

async fn join(ws: &mut Ws, world: u64, civ: u32) -> Value {
    let reply = request(ws, "join", json!({ "world_id": world, "civ": civ })).await;
    assert_eq!(reply["type"], "joined", "{reply}");
    reply["payload"].clone()
}

#[tokio::test]
async fn health_is_ok() {
    let (addr, _) = start(Duration::from_secs(60)).await;
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream.write_all(b"GET /health HTTP/1.0\r\nHost: localhost\r\n\r\n").await.unwrap();
    let mut body = String::new();
    stream.read_to_string(&mut body).await.unwrap();
    assert!(body.starts_with("HTTP/1.0 200") || body.starts_with("HTTP/1.1 200"), "{body}");
    assert!(body.contains("\"status\":\"ok\""), "{body}");
}

#[tokio::test]
async fn create_join_ready_advances_a_turn_and_the_log_replays() {
    let (addr, store) = start(Duration::from_secs(60)).await;
    let mut ws = connect(addr).await;
    create(&mut ws, 7, 2).await;
    join(&mut ws, 7, 0).await;

    let accepted = request(&mut ws, "submit_command", json!({ "command": { "type": "keep_plan" } })).await;
    assert_eq!(accepted["type"], "command_accepted", "{accepted}");
    assert_eq!(accepted["payload"]["accepted_sequence"], 1);

    let ready = request(&mut ws, "ready", json!({})).await;
    assert_eq!(ready["type"], "ready_state");
    let diff = push_of(&mut ws, "turn_diff").await;
    assert_eq!(diff["from_turn"], 0);
    assert_eq!(diff["turn"], 1);
    assert_eq!(diff["view"]["civ"], 0);

    let record = store.load(WorldId(7)).unwrap().unwrap();
    assert_eq!(record.state.turn, TurnNumber(1));
    assert!(record.log.turn_hashes.contains_key(&TurnNumber(0)));
    let replayed = replay(&record.snapshot, &record.log).expect("the stored log replays bit for bit");
    assert_eq!(replayed.state, record.state);
    assert!(record.log.commands.iter().any(|c| c.actor_id == CivId(0) && c.origin == CommandOrigin::Player));
}

#[tokio::test]
async fn absent_player_is_played_by_the_governor() {
    let (addr, store) = start(Duration::from_millis(100)).await;
    let mut a = connect(addr).await;
    create(&mut a, 11, 2).await;
    join(&mut a, 11, 0).await;
    let mut b = connect(addr).await;
    join(&mut b, 11, 1).await;

    // B is present, so A alone cannot close the turn; B leaves and the grace expires.
    request(&mut a, "ready", json!({})).await;
    assert_eq!(store.load(WorldId(11)).unwrap().unwrap().state.turn, TurnNumber(0));
    b.close(None).await.unwrap();
    drop(b);
    push_of(&mut a, "turn_diff").await;

    for _ in 0..4 {
        request(&mut a, "ready", json!({})).await;
        push_of(&mut a, "turn_diff").await;
    }
    let record = store.load(WorldId(11)).unwrap().unwrap();
    assert_eq!(record.state.turn, TurnNumber(5));
    let by_governor = record.log.commands.iter().filter(|c| c.actor_id == CivId(1) && c.origin == CommandOrigin::Governor).count();
    assert!(by_governor > 0, "the Governor must have played the absent civilization");
    assert!(record.log.commands.iter().all(|c| !(c.actor_id == CivId(1) && c.origin == CommandOrigin::Player)));
    assert!(replay(&record.snapshot, &record.log).is_ok());
}

#[tokio::test]
async fn invalid_command_is_rejected_without_state_change() {
    let (addr, store) = start(Duration::from_secs(60)).await;
    let mut ws = connect(addr).await;
    create(&mut ws, 3, 2).await;
    join(&mut ws, 3, 0).await;
    let before = request(&mut ws, "get_snapshot", json!({})).await["payload"]["state_hash"].clone();

    let reply = request(&mut ws, "submit_command", json!({ "command": { "type": "move_unit", "data": { "unit_id": 999, "target": 0 } } })).await;
    assert_eq!(reply["type"], "error", "{reply}");
    assert_eq!(reply["payload"]["reason"], "command_rejected");
    assert_eq!(reply["payload"]["engine_reason"], "unknown_unit");

    let after = request(&mut ws, "get_snapshot", json!({})).await["payload"]["state_hash"].clone();
    assert_eq!(before, after);
    let record = store.load(WorldId(3)).unwrap().unwrap();
    assert!(record.log.commands.is_empty());
    assert_eq!(record.state.turn, TurnNumber(0));
}

#[tokio::test]
async fn protocol_version_mismatch_is_rejected() {
    let (addr, _) = start(Duration::from_secs(60)).await;
    let mut ws = connect(addr).await;
    let bad = json!({ "protocol_version": 99, "request_id": "x1", "type": "create_world", "payload": { "seed": 1, "civs": 2 } });
    ws.send(Message::Text(bad.to_string())).await.unwrap();
    let reply = next_frame(&mut ws).await;
    assert_eq!(reply["type"], "error");
    assert_eq!(reply["request_id"], "x1");
    assert_eq!(reply["payload"]["reason"], "protocol_version_mismatch");

    ws.send(Message::Text("not json".into())).await.unwrap();
    assert_eq!(next_frame(&mut ws).await["payload"]["reason"], "malformed_message");
}

#[tokio::test]
async fn a_claimed_civilization_needs_its_session_token() {
    let (addr, _) = start(Duration::from_secs(60)).await;
    let mut a = connect(addr).await;
    create(&mut a, 5, 2).await;
    let joined = join(&mut a, 5, 0).await;
    let token = joined["session_token"].as_str().unwrap().to_string();

    let mut b = connect(addr).await;
    let refused = request(&mut b, "join", json!({ "world_id": 5, "civ": 0 })).await;
    assert_eq!(refused["payload"]["reason"], "civ_taken");
    let forged = request(&mut b, "join", json!({ "world_id": 5, "civ": 0, "session_token": "nope" })).await;
    assert_eq!(forged["payload"]["reason"], "invalid_session_token");
    let resumed = request(&mut b, "join", json!({ "world_id": 5, "civ": 0, "session_token": token })).await;
    assert_eq!(resumed["type"], "joined");
}
