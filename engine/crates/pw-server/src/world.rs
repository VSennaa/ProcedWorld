//! Live (in-memory) world: seats, open-turn command list and turn resolution.
//!
//! All methods are synchronous and are called with the world mutex held, so no `await` ever
//! happens while a world is locked. The engine decides every rule; this module only orders
//! commands, tracks presence/readiness (ADR-0008) and asks the Governor to play for civilizations
//! without a present human.

use std::{
    collections::{hash_map::RandomState, BTreeMap},
    hash::{BuildHasher, Hasher},
    sync::Arc,
};

use pw_engine::{
    entropy::{bundled_catalog, respond_commands, EntropyDirector},
    governor::{Governor, Mandate, MandatePreset},
    ids::{CivId, TileIndex, UnitId},
    world::{
        idle_units_with, step, AcceptedCommand, CommandKind, CommandOrigin, CommandPayload,
        DomainEvent, RejectionReason, SimulationVersions, WorldState,
    },
};
use serde_json::{json, Value};
use tokio::sync::mpsc::Sender;

use crate::{
    protocol::{frame, ErrorReason},
    store::WorldStore,
    view::{events_for, state_hash_string, view_for},
};

/// Outgoing channel of one WebSocket connection.
pub struct Conn {
    pub id: u64,
    pub tx: Sender<String>,
}

/// A civilization claimed by a human. Without a seat, a civilization is a bot (Governor-played).
struct Seat {
    token: String,
    conn: Option<Conn>,
    ready: bool,
    /// Reconnect grace expired: the Governor plays this civilization until the human returns.
    absent: bool,
    /// Bumped on every join/disconnect so a stale grace timer cannot mark a returned player absent.
    generation: u64,
}

pub struct SubmitError {
    pub reason: ErrorReason,
    pub engine_reason: Option<RejectionReason>,
    pub detail: String,
}

impl SubmitError {
    fn plain(reason: ErrorReason, detail: &str) -> Self {
        Self {
            reason,
            engine_reason: None,
            detail: detail.into(),
        }
    }
}

pub struct LiveWorld {
    state: WorldState,
    versions: SimulationVersions,
    homes: BTreeMap<CivId, TileIndex>,
    seats: BTreeMap<CivId, Seat>,
    /// Commands accepted in the open turn, in `accepted_sequence` order.
    pending: Vec<AcceptedCommand>,
    next_command_id: u64,
    mandate: Mandate,
    store: Arc<dyn WorldStore>,
}

/// Session tokens come from the OS-seeded `RandomState`. They are unguessable enough for the
/// current stage but are not cryptographic; real authentication replaces them (docs/sdd/10 sec. 4).
fn new_token() -> String {
    let mut token = String::new();
    for round in 0..2u64 {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u64(round);
        token.push_str(&format!("{:016x}", hasher.finish()));
    }
    token
}

fn kind_of(payload: &CommandPayload) -> CommandKind {
    match payload {
        CommandPayload::EndTurn => CommandKind::EndTurn,
        CommandPayload::MoveUnit { .. } => CommandKind::MoveUnit,
        CommandPayload::FoundCity { .. } => CommandKind::FoundCity,
        CommandPayload::SetCityFocus { .. } => CommandKind::SetCityFocus,
        CommandPayload::SetResearch { .. } => CommandKind::SetResearch,
        CommandPayload::SetResearchInvestment { .. } => CommandKind::SetResearchInvestment,
        CommandPayload::ActivatePractice { .. } => CommandKind::ActivatePractice,
        CommandPayload::DeactivatePractice { .. } => CommandKind::DeactivatePractice,
        CommandPayload::QueueUnit { .. } => CommandKind::QueueUnit,
        CommandPayload::RemoveQueuedUnit { .. } => CommandKind::RemoveQueuedUnit,
        CommandPayload::MoveQueuedUnit { .. } => CommandKind::MoveQueuedUnit,
        CommandPayload::DeclareAttack { .. } => CommandKind::DeclareAttack,
        CommandPayload::Explore { .. } => CommandKind::Explore,
        CommandPayload::SetUnitOrder { .. } => CommandKind::SetUnitOrder,
        CommandPayload::SkipUnit { .. } => CommandKind::SkipUnit,
        CommandPayload::KeepPlan => CommandKind::KeepPlan,
        CommandPayload::ProposeDiplomacy { .. } => CommandKind::ProposeDiplomacy,
        CommandPayload::BreakTreaty { .. } => CommandKind::BreakTreaty,
        CommandPayload::DeclareWar { .. } => CommandKind::DeclareWar,
        CommandPayload::ApplyEvent { .. } => CommandKind::ApplyEvent,
        CommandPayload::RespondToEvent { .. } => CommandKind::RespondToEvent,
    }
}

impl LiveWorld {
    pub fn new(
        state: WorldState,
        versions: SimulationVersions,
        homes: BTreeMap<CivId, TileIndex>,
        store: Arc<dyn WorldStore>,
    ) -> Self {
        Self {
            state,
            versions,
            homes,
            seats: BTreeMap::new(),
            pending: Vec::new(),
            next_command_id: 1,
            mandate: Mandate::preset(MandatePreset::GrowCautiously, 1),
            store,
        }
    }

    /// Rebuilds a live world after a process restart. Restored seats are absent until their
    /// owner proves possession of the persisted session token.
    pub fn restore(record: crate::store::WorldRecord, store: Arc<dyn WorldStore>) -> Self {
        let next_command_id = record
            .log
            .commands
            .iter()
            .map(|command| command.command_id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let pending = record
            .log
            .commands
            .iter()
            .filter(|command| command.turn == record.state.turn)
            .cloned()
            .collect();
        let seats = record
            .session_tokens
            .into_iter()
            .map(|(civ, token)| {
                (
                    civ,
                    Seat {
                        token,
                        conn: None,
                        ready: false,
                        absent: true,
                        generation: 0,
                    },
                )
            })
            .collect();
        Self {
            state: record.state,
            versions: record.snapshot.versions,
            homes: record.home_tiles,
            seats,
            pending,
            next_command_id,
            mandate: Mandate::preset(MandatePreset::GrowCautiously, 1),
            store,
        }
    }

    fn persist_sessions(&self) -> Result<(), ErrorReason> {
        let tokens = self
            .seats
            .iter()
            .map(|(civ, seat)| (*civ, seat.token.clone()))
            .collect();
        self.store
            .save_session_tokens(self.state.world_id, &tokens)
            .map_err(|_| ErrorReason::Internal)
    }

    pub fn state(&self) -> &WorldState {
        &self.state
    }

    pub fn civs(&self) -> Vec<u32> {
        self.state.civilizations.keys().map(|civ| civ.0).collect()
    }

    pub fn snapshot_for(&self, civ: CivId) -> Value {
        view_for(
            &self.state,
            civ,
            self.homes.get(&civ).copied(),
            &self.pending,
        )
    }

    /// Idle units that block `ready` (SDD 03 / SDD 15 section 4.3): only for a present human seat,
    /// counting the orders already accepted in the open turn. Bots and absent seats are never blocked.
    pub fn idle_blocking(&self, civ: CivId) -> Vec<UnitId> {
        if !self.seats.get(&civ).is_some_and(|seat| !seat.absent) {
            return Vec::new();
        }
        idle_units_with(&self.state, civ, &self.pending)
    }

    /// Claims a civilization (new seat) or resumes one with the session token.
    pub fn join(
        &mut self,
        civ: CivId,
        token: Option<&str>,
        conn: Conn,
    ) -> Result<String, ErrorReason> {
        if !self.state.civilizations.contains_key(&civ) {
            return Err(ErrorReason::CivNotFound);
        }
        match self.seats.get_mut(&civ) {
            Some(seat) => match token {
                None => Err(ErrorReason::CivTaken),
                Some(given) if given != seat.token => Err(ErrorReason::InvalidSessionToken),
                Some(_) => {
                    if seat.absent {
                        seat.ready = false;
                    }
                    seat.absent = false;
                    seat.generation += 1;
                    seat.conn = Some(conn);
                    Ok(seat.token.clone())
                }
            },
            None => {
                if token.is_some() {
                    return Err(ErrorReason::InvalidSessionToken);
                }
                let token = new_token();
                self.seats.insert(
                    civ,
                    Seat {
                        token: token.clone(),
                        conn: Some(conn),
                        ready: false,
                        absent: false,
                        generation: 0,
                    },
                );
                self.persist_sessions()?;
                Ok(token)
            }
        }
    }

    /// Marks the connection as gone. Returns the generation for the grace timer, or `None` when
    /// the seat has already been taken over by another connection.
    pub fn disconnect(&mut self, civ: CivId, conn_id: u64) -> Option<u64> {
        let seat = self.seats.get_mut(&civ)?;
        if seat.conn.as_ref().map(|conn| conn.id) != Some(conn_id) {
            return None;
        }
        seat.conn = None;
        seat.generation += 1;
        Some(seat.generation)
    }

    /// Called when the technical reconnect grace elapsed. True when the seat became absent.
    pub fn grace_expired(&mut self, civ: CivId, generation: u64) -> bool {
        match self.seats.get_mut(&civ) {
            Some(seat) if seat.conn.is_none() && seat.generation == generation && !seat.absent => {
                seat.absent = true;
                true
            }
            _ => false,
        }
    }

    pub fn set_ready(&mut self, civ: CivId, ready: bool) -> Result<(), ErrorReason> {
        let seat = self
            .seats
            .get_mut(&civ)
            .filter(|seat| !seat.absent)
            .ok_or(ErrorReason::NotJoined)?;
        seat.ready = ready;
        Ok(())
    }

    pub fn ready_payload(&self) -> Value {
        let present: Vec<u32> = self
            .seats
            .iter()
            .filter(|(_, s)| !s.absent)
            .map(|(civ, _)| civ.0)
            .collect();
        let ready: Vec<u32> = self
            .seats
            .iter()
            .filter(|(_, s)| !s.absent && s.ready)
            .map(|(civ, _)| civ.0)
            .collect();
        json!({ "turn": self.state.turn, "present": present, "ready": ready })
    }

    /// Pushes an unsolicited frame to every connected seat except `exclude` (a connection id).
    pub fn broadcast(&self, kind: &str, payload: &Value, exclude: Option<u64>) {
        let text = frame(None, kind, payload.clone());
        for seat in self.seats.values() {
            if let Some(conn) = &seat.conn {
                if Some(conn.id) != exclude {
                    // A full queue means a slow client: it can recover with `get_snapshot`.
                    let _ = conn.tx.try_send(text.clone());
                }
            }
        }
    }

    /// Validates a player command with the engine (dry run of the open turn plus this command),
    /// then stores it with the next `accepted_sequence`. On rejection nothing changes.
    pub fn submit(
        &mut self,
        civ: CivId,
        payload: CommandPayload,
    ) -> Result<AcceptedCommand, SubmitError> {
        let seat = self
            .seats
            .get(&civ)
            .filter(|seat| !seat.absent)
            .ok_or_else(|| {
                SubmitError::plain(ErrorReason::NotJoined, "join a civilization first")
            })?;
        if seat.ready {
            return Err(SubmitError::plain(
                ErrorReason::AlreadyReady,
                "send unready before changing orders",
            ));
        }
        let command = AcceptedCommand {
            command_id: self.next_command_id,
            world_id: self.state.world_id,
            turn: self.state.turn,
            accepted_sequence: self.pending.len() as u64 + 1,
            actor_id: civ,
            origin: CommandOrigin::Player,
            kind: kind_of(&payload),
            payload,
            grounding: Vec::new(),
            intent_evidence: None,
            mandate: None,
        };
        let mut trial = self.pending.clone();
        trial.push(command.clone());
        let result = step(&self.state, &trial, self.state.seed, &self.versions);
        let rejection = result.events.iter().find_map(|event| match event {
            DomainEvent::CommandRejected { command_id, reason }
                if *command_id == command.command_id =>
            {
                Some(*reason)
            }
            _ => None,
        });
        if let Some(reason) = rejection {
            return Err(SubmitError {
                reason: ErrorReason::CommandRejected,
                engine_reason: Some(reason),
                detail: format!("{reason:?}"),
            });
        }
        self.store
            .append_command(self.state.world_id, &command)
            .map_err(|_| SubmitError::plain(ErrorReason::Internal, "command log unavailable"))?;
        self.pending.push(command.clone());
        self.next_command_id += 1;
        Ok(command)
    }

    /// Resolves the turn when at least one human is present and every present human is Ready.
    /// Civilizations without a present human are played by their Governor (ADR-0008).
    pub fn try_advance(&mut self) -> Result<bool, ErrorReason> {
        let mut present = self.seats.values().filter(|seat| !seat.absent).peekable();
        if present.peek().is_none() || !present.all(|seat| seat.ready) {
            return Ok(false);
        }
        let mut turn_commands = self.pending.clone();
        let mut governor_commands = Vec::new();
        let mut next_id = self.next_command_id;
        for civ in pw_harness::rotating_order(self.state.seed, self.state.turn, &self.homes) {
            if self.seats.get(&civ).is_some_and(|seat| !seat.absent) {
                continue;
            }
            let Some(home) = self.homes.get(&civ).copied() else {
                continue;
            };
            let governor = Governor {
                mandate: &self.mandate,
                decision_port: None,
            };
            let decision = governor.decide(
                &self.state,
                civ,
                home,
                true,
                next_id,
                turn_commands.len() as u64 + 1,
            );
            next_id = next_id.saturating_add(decision.commands.len().max(1) as u64);
            governor_commands.extend(decision.commands.iter().cloned());
            turn_commands.extend(decision.commands);
            // The Governor also answers pending Entropy events of the civilizations it plays.
            let responses = respond_commands(
                &self.state,
                civ,
                &self.mandate,
                next_id,
                turn_commands.len() as u64 + 1,
            );
            next_id = next_id.saturating_add(responses.len() as u64);
            governor_commands.extend(responses.iter().cloned());
            turn_commands.extend(responses);
        }
        // World-scoped Entropy director: events are commands with origin Entropy, validated by the engine.
        let catalog = bundled_catalog();
        let director = EntropyDirector {
            catalog: &catalog,
            decision_port: None,
        };
        let proposed = director.propose(&self.state, next_id, turn_commands.len() as u64 + 1);
        next_id = next_id.saturating_add(proposed.len() as u64);
        governor_commands.extend(proposed.iter().cloned());
        turn_commands.extend(proposed);
        let sealed = self.state.turn;
        let result = step(&self.state, &turn_commands, self.state.seed, &self.versions);
        self.store
            .commit_turn(
                self.state.world_id,
                &governor_commands,
                sealed,
                result.state_hash,
                &result.state,
            )
            .map_err(|_| ErrorReason::Internal)?;
        self.state = result.state;
        self.pending.clear();
        self.next_command_id = next_id;
        for seat in self.seats.values_mut() {
            seat.ready = false;
        }
        for (civ, seat) in &self.seats {
            let Some(conn) = &seat.conn else { continue };
            let own: Vec<&AcceptedCommand> = turn_commands
                .iter()
                .filter(|command| command.actor_id == *civ)
                .collect();
            // The engine has no delta representation yet: the diff carries the civilization's
            // filtered full view plus its own command results (see README, protocol gaps).
            let payload = json!({
                "from_turn": sealed,
                "turn": self.state.turn,
                "state_hash": state_hash_string(&self.state),
                "commands": own,
                "events": events_for(&self.state, *civ, &turn_commands, &result.events),
                "view": view_for(&self.state, *civ, self.homes.get(civ).copied(), &[]),
            });
            let _ = conn.tx.try_send(frame(None, "turn_diff", payload));
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{InMemoryStore, WorldRecord};
    use pw_engine::world::{CommandLog, UnitOrder, UnitState, WorldSnapshot};

    fn live_with_idle_unit() -> LiveWorld {
        let (mut state, versions, homes) = pw_harness::initial_world(5, 2).expect("world builds");
        state.units.insert(
            UnitId(900),
            UnitState {
                owner: CivId(0),
                tile: homes[&CivId(0)],
                unit_type: "unit.scout".into(),
                hit_points: 100,
                movement_left: 3,
                explored: false,
                order: UnitOrder::Idle,
                skipped_turn: None,
            },
        );
        let store = Arc::new(InMemoryStore::default());
        store
            .create(WorldRecord {
                snapshot: WorldSnapshot::new(state.clone(), versions.clone()),
                log: CommandLog::default(),
                home_tiles: homes.clone(),
                session_tokens: Default::default(),
                state: state.clone(),
            })
            .expect("store accepts the world");
        LiveWorld::new(state, versions, homes, store)
    }

    fn conn(id: u64) -> Conn {
        Conn {
            id,
            tx: tokio::sync::mpsc::channel(16).0,
        }
    }

    fn unit_json(live: &LiveWorld) -> Value {
        let view = live.snapshot_for(CivId(0));
        view["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|unit| unit["id"] == 900)
            .unwrap()["unit"]
            .clone()
    }

    #[test]
    fn idle_unit_blocks_ready_until_ordered_or_skipped_and_skip_expires_next_turn() {
        let mut live = live_with_idle_unit();
        live.join(CivId(0), None, conn(1)).unwrap();
        assert_eq!(live.idle_blocking(CivId(0)), vec![UnitId(900)]);
        assert_eq!(live.snapshot_for(CivId(0))["idle_units"], json!([900]));

        assert!(live
            .submit(
                CivId(0),
                CommandPayload::SkipUnit {
                    unit_id: UnitId(900)
                }
            )
            .is_ok());
        assert!(live.idle_blocking(CivId(0)).is_empty());
        assert_eq!(live.snapshot_for(CivId(0))["idle_units"], json!([]));
        assert_eq!(unit_json(&live)["skipped_turn"], json!(0));

        live.set_ready(CivId(0), true).unwrap();
        assert!(live.try_advance().unwrap());
        // The skip only covered the turn it was given in.
        assert_eq!(live.idle_blocking(CivId(0)), vec![UnitId(900)]);

        assert!(live
            .submit(
                CivId(0),
                CommandPayload::SetUnitOrder {
                    unit_id: UnitId(900),
                    order: UnitOrder::Fortify
                }
            )
            .is_ok());
        assert!(live.idle_blocking(CivId(0)).is_empty());
        assert_eq!(unit_json(&live)["order"], json!({ "type": "fortify" }));
    }

    #[test]
    fn home_tile_is_published_only_until_the_first_city_exists() {
        let mut live = live_with_idle_unit();
        let home = live.homes[&CivId(0)];
        live.join(CivId(0), None, conn(1)).unwrap();
        assert_eq!(live.snapshot_for(CivId(0))["home_tile"], json!(home));
        assert!(
            live.snapshot_for(CivId(1))["home_tile"].is_number(),
            "every civilization without a city gets its starting tile"
        );
        // The first city needs no settler: the capital id is the civilization id (bots do the same).
        assert!(live
            .submit(
                CivId(0),
                CommandPayload::FoundCity {
                    city_id: pw_engine::ids::CityId(0),
                    target: home
                }
            )
            .is_ok());
        // A second FoundCity with the same id is a clean rejection, not a state change.
        let duplicate = live.submit(
            CivId(0),
            CommandPayload::FoundCity {
                city_id: pw_engine::ids::CityId(0),
                target: home,
            },
        );
        assert!(matches!(
            duplicate,
            Err(SubmitError {
                reason: ErrorReason::CommandRejected,
                ..
            })
        ));
        assert!(live
            .submit(
                CivId(0),
                CommandPayload::SkipUnit {
                    unit_id: UnitId(900)
                }
            )
            .is_ok());
        live.set_ready(CivId(0), true).unwrap();
        assert!(live.try_advance().unwrap());
        let view = live.snapshot_for(CivId(0));
        assert_eq!(view["home_tile"], Value::Null);
        assert_eq!(
            view["cities"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| c["city"]["owner"] == 0)
                .count(),
            1
        );
        // A settler about to found a city no longer blocks Ready (it is consumed at resolution).
        let site = TileIndex(home.0 + 2);
        live.state.units.insert(
            UnitId(901),
            UnitState {
                owner: CivId(0),
                tile: site,
                unit_type: "unit.settler".into(),
                hit_points: 100,
                movement_left: 3,
                explored: false,
                order: UnitOrder::Idle,
                skipped_turn: None,
            },
        );
        assert!(live.idle_blocking(CivId(0)).contains(&UnitId(901)));
        assert!(live
            .submit(
                CivId(0),
                CommandPayload::FoundCity {
                    city_id: pw_engine::ids::CityId(1_000_901),
                    target: site
                }
            )
            .is_ok());
        assert!(!live.idle_blocking(CivId(0)).contains(&UnitId(901)));
    }

    #[test]
    fn absent_or_unseated_civilizations_are_never_blocked() {
        let mut live = live_with_idle_unit();
        assert!(live.idle_blocking(CivId(0)).is_empty());
        live.join(CivId(0), None, conn(1)).unwrap();
        let generation = live.disconnect(CivId(0), 1).unwrap();
        assert!(live.grace_expired(CivId(0), generation));
        assert!(live.idle_blocking(CivId(0)).is_empty());
    }
}
