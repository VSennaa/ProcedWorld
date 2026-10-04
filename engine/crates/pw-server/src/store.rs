//! World persistence behind a trait. The file adapter records the initial snapshot,
//! accepted-command log and session tokens needed to reproduce a world.

use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use pw_engine::{
    hash::StateHash,
    ids::{CivId, TileIndex, TurnNumber},
    world::{replay, AcceptedCommand, CommandLog, WorldId, WorldSnapshot, WorldState},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct WorldRecord {
    /// Immutable starting point; `state` is derived from this plus the sealed log.
    pub snapshot: WorldSnapshot,
    pub log: CommandLog,
    pub home_tiles: BTreeMap<CivId, TileIndex>,
    pub session_tokens: BTreeMap<CivId, String>,
    pub state: WorldState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    AlreadyExists,
    NotFound,
    Io(String),
    Invalid(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyExists => formatter.write_str("world already exists"),
            Self::NotFound => formatter.write_str("world not found"),
            Self::Io(message) | Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for StoreError {}

pub trait WorldStore: Send + Sync {
    fn create(&self, record: WorldRecord) -> Result<(), StoreError>;
    fn append_command(&self, world: WorldId, command: &AcceptedCommand) -> Result<(), StoreError>;
    fn commit_turn(
        &self,
        world: WorldId,
        commands: &[AcceptedCommand],
        sealed_turn: TurnNumber,
        hash: StateHash,
        next: &WorldState,
    ) -> Result<(), StoreError>;
    fn save_session_tokens(
        &self,
        world: WorldId,
        tokens: &BTreeMap<CivId, String>,
    ) -> Result<(), StoreError>;
    fn load(&self, world: WorldId) -> Result<Option<WorldRecord>, StoreError>;
    fn list(&self) -> Result<Vec<WorldId>, StoreError>;
}

#[derive(Default)]
pub struct InMemoryStore {
    worlds: Mutex<HashMap<WorldId, WorldRecord>>,
}
impl InMemoryStore {
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<WorldId, WorldRecord>> {
        self.worlds.lock().unwrap_or_else(|p| p.into_inner())
    }
}
impl WorldStore for InMemoryStore {
    fn create(&self, record: WorldRecord) -> Result<(), StoreError> {
        let mut worlds = self.lock();
        let id = record.state.world_id;
        if worlds.contains_key(&id) {
            return Err(StoreError::AlreadyExists);
        }
        worlds.insert(id, record);
        Ok(())
    }
    fn append_command(&self, world: WorldId, command: &AcceptedCommand) -> Result<(), StoreError> {
        self.lock()
            .get_mut(&world)
            .ok_or(StoreError::NotFound)?
            .log
            .append(command.clone());
        Ok(())
    }
    fn commit_turn(
        &self,
        world: WorldId,
        commands: &[AcceptedCommand],
        sealed_turn: TurnNumber,
        hash: StateHash,
        next: &WorldState,
    ) -> Result<(), StoreError> {
        let mut worlds = self.lock();
        let record = worlds.get_mut(&world).ok_or(StoreError::NotFound)?;
        for command in commands {
            record.log.append(command.clone());
        }
        record.log.record_turn(sealed_turn, hash);
        record.state = next.clone();
        Ok(())
    }
    fn save_session_tokens(
        &self,
        world: WorldId,
        tokens: &BTreeMap<CivId, String>,
    ) -> Result<(), StoreError> {
        self.lock()
            .get_mut(&world)
            .ok_or(StoreError::NotFound)?
            .session_tokens = tokens.clone();
        Ok(())
    }
    fn load(&self, world: WorldId) -> Result<Option<WorldRecord>, StoreError> {
        Ok(self.lock().get(&world).cloned())
    }
    fn list(&self) -> Result<Vec<WorldId>, StoreError> {
        let mut ids: Vec<_> = self.lock().keys().copied().collect();
        ids.sort();
        Ok(ids)
    }
}

/// JSON files, one directory per world. Replacement files are fully written before their rename.
pub struct FileStore {
    root: PathBuf,
}
#[derive(Serialize, Deserialize)]
struct Metadata {
    home_tiles: BTreeMap<CivId, TileIndex>,
    session_tokens: BTreeMap<CivId, String>,
}

/// Mutable turn data is one file so a replacement never exposes a new log with an old snapshot.
#[derive(Serialize, Deserialize)]
struct TurnState {
    log: CommandLog,
    snapshot: WorldSnapshot,
}

impl FileStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(io_error)?;
        Ok(Self { root })
    }
    fn world_dir(&self, world: WorldId) -> PathBuf {
        self.root.join(world.0.to_string())
    }
    fn read_json<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<T, StoreError> {
        serde_json::from_slice(&fs::read(path).map_err(io_error)?)
            .map_err(|e| StoreError::Invalid(format!("{}: {e}", path.display())))
    }
    fn write_json<T: Serialize>(&self, path: &Path, value: &T) -> Result<(), StoreError> {
        let bytes =
            serde_json::to_vec_pretty(value).map_err(|e| StoreError::Invalid(e.to_string()))?;
        let temp = path.with_extension("tmp");
        let mut file = fs::File::create(&temp).map_err(io_error)?;
        file.write_all(&bytes).map_err(io_error)?;
        file.sync_all().map_err(io_error)?;
        fs::rename(&temp, path).map_err(io_error)
    }
    fn write_record(&self, record: &WorldRecord) -> Result<(), StoreError> {
        let dir = self.world_dir(record.state.world_id);
        self.write_json(&dir.join("initial_snapshot.json"), &record.snapshot)?;
        self.write_json(
            &dir.join("turn.json"),
            &TurnState {
                log: record.log.clone(),
                snapshot: WorldSnapshot::new(record.state.clone(), record.snapshot.versions.clone()),
            },
        )?;
        self.write_json(
            &dir.join("metadata.json"),
            &Metadata {
                home_tiles: record.home_tiles.clone(),
                session_tokens: record.session_tokens.clone(),
            },
        )
    }
}
fn io_error(error: io::Error) -> StoreError {
    StoreError::Io(error.to_string())
}

impl WorldStore for FileStore {
    fn create(&self, record: WorldRecord) -> Result<(), StoreError> {
        let dir = self.world_dir(record.state.world_id);
        match fs::create_dir(&dir) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                return Err(StoreError::AlreadyExists)
            }
            Err(e) => return Err(io_error(e)),
        }
        self.write_record(&record)
    }
    fn append_command(&self, world: WorldId, command: &AcceptedCommand) -> Result<(), StoreError> {
        let mut record = self.load(world)?.ok_or(StoreError::NotFound)?;
        record.log.append(command.clone());
        self.write_json(
            &self.world_dir(world).join("turn.json"),
            &TurnState {
                log: record.log,
                snapshot: WorldSnapshot::new(record.state, record.snapshot.versions),
            },
        )
    }
    fn commit_turn(
        &self,
        world: WorldId,
        commands: &[AcceptedCommand],
        sealed_turn: TurnNumber,
        hash: StateHash,
        next: &WorldState,
    ) -> Result<(), StoreError> {
        let mut record = self.load(world)?.ok_or(StoreError::NotFound)?;
        for command in commands {
            record.log.append(command.clone());
        }
        record.log.record_turn(sealed_turn, hash);
        record.state = next.clone();
        self.write_json(
            &self.world_dir(world).join("turn.json"),
            &TurnState {
                log: record.log,
                snapshot: WorldSnapshot::new(next.clone(), record.snapshot.versions),
            },
        )
    }
    fn save_session_tokens(
        &self,
        world: WorldId,
        tokens: &BTreeMap<CivId, String>,
    ) -> Result<(), StoreError> {
        let record = self.load(world)?.ok_or(StoreError::NotFound)?;
        self.write_json(
            &self.world_dir(world).join("metadata.json"),
            &Metadata {
                home_tiles: record.home_tiles,
                session_tokens: tokens.clone(),
            },
        )
    }
    fn load(&self, world: WorldId) -> Result<Option<WorldRecord>, StoreError> {
        let dir = self.world_dir(world);
        if !dir.is_dir() {
            return Ok(None);
        }
        let snapshot: WorldSnapshot = self.read_json(&dir.join("initial_snapshot.json"))?;
        let turn: TurnState = self.read_json(&dir.join("turn.json"))?;
        let head = turn.snapshot;
        let log = turn.log;
        let metadata: Metadata = self.read_json(&dir.join("metadata.json"))?;
        if snapshot.state.world_id != world || head.state.world_id != world {
            return Err(StoreError::Invalid(
                "world id does not match its directory".into(),
            ));
        }
        let state = replay(&snapshot, &log)
            .map_err(|e| StoreError::Invalid(format!("replay rejected: {e:?}")))?
            .state;
        if !head.verify() || state.state_hash() != head.state_hash {
            return Err(StoreError::Invalid(
                "replay hash differs from snapshot".into(),
            ));
        }
        Ok(Some(WorldRecord {
            snapshot,
            log,
            home_tiles: metadata.home_tiles,
            session_tokens: metadata.session_tokens,
            state,
        }))
    }
    fn list(&self) -> Result<Vec<WorldId>, StoreError> {
        let mut worlds = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            if !entry.file_type().map_err(io_error)?.is_dir() {
                continue;
            }
            if let Ok(id) = entry.file_name().to_string_lossy().parse::<u64>() {
                worlds.push(WorldId(id));
            }
        }
        worlds.sort();
        Ok(worlds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pw_engine::{
        world::{step, AcceptedCommand, CommandKind, CommandOrigin, CommandPayload},
    };

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("pw-server-{name}-{}", std::process::id()))
    }

    fn record(seed: u64) -> (WorldRecord, pw_engine::world::SimulationVersions) {
        let (state, versions, homes) = pw_harness::initial_world(seed, 2).expect("world builds");
        (
            WorldRecord {
                snapshot: WorldSnapshot::new(state.clone(), versions.clone()),
                log: CommandLog::default(),
                home_tiles: homes,
                session_tokens: BTreeMap::new(),
                state,
            },
            versions,
        )
    }

    fn end_turn(world: WorldId, turn: TurnNumber, actor: CivId) -> AcceptedCommand {
        AcceptedCommand {
            command_id: 1,
            world_id: world,
            turn,
            accepted_sequence: 1,
            actor_id: actor,
            origin: CommandOrigin::Player,
            kind: CommandKind::EndTurn,
            payload: CommandPayload::EndTurn,
            grounding: Vec::new(),
            intent_evidence: None,
            mandate: None,
        }
    }

    #[test]
    fn file_store_reloads_the_same_turn_and_hash() {
        let root = test_dir("reload");
        let _ = fs::remove_dir_all(&root);
        let store = FileStore::open(&root).unwrap();
        let (record, versions) = record(91);
        let world = record.state.world_id;
        let command = end_turn(world, TurnNumber::ZERO, CivId(0));
        let result = step(&record.state, &[command.clone()], record.state.seed, &versions);
        store.create(record).unwrap();
        store
            .commit_turn(
                world,
                &[command],
                TurnNumber::ZERO,
                result.state_hash,
                &result.state,
            )
            .unwrap();
        let loaded = store.load(world).unwrap().expect("saved world");
        assert_eq!(loaded.state.turn, result.state.turn);
        assert_eq!(loaded.state.state_hash(), result.state_hash);
        let tokens = BTreeMap::from([(CivId(0), "resume-token".to_owned())]);
        store.save_session_tokens(world, &tokens).unwrap();
        assert_eq!(store.load(world).unwrap().unwrap().session_tokens, tokens);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn file_store_recovers_from_interrupted_turn_replacement() {
        let root = test_dir("divergent");
        let _ = fs::remove_dir_all(&root);
        let store = FileStore::open(&root).unwrap();
        let (record, versions) = record(92);
        let world = record.state.world_id;
        let command = end_turn(world, TurnNumber::ZERO, CivId(0));
        let result = step(&record.state, &[command.clone()], record.state.seed, &versions);
        store.create(record).unwrap();
        store
            .commit_turn(
                world,
                &[command],
                TurnNumber::ZERO,
                result.state_hash,
                &result.state,
            )
            .unwrap();
        fs::write(store.world_dir(world).join("turn.tmp"), b"incomplete replacement").unwrap();
        let loaded = store.load(world).unwrap().expect("last complete turn survives");
        assert_eq!(loaded.state.state_hash(), result.state_hash);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn file_store_refuses_snapshot_that_differs_from_replay() {
        let root = test_dir("snapshot-divergent");
        let _ = fs::remove_dir_all(&root);
        let store = FileStore::open(&root).unwrap();
        let (record, versions) = record(93);
        let world = record.state.world_id;
        let initial = record.state.clone();
        let command = end_turn(world, TurnNumber::ZERO, CivId(0));
        let result = step(&initial, &[command.clone()], initial.seed, &versions);
        store.create(record).unwrap();
        store
            .commit_turn(world, &[command], TurnNumber::ZERO, result.state_hash, &result.state)
            .unwrap();
        let mut turn: TurnState = store.read_json(&store.world_dir(world).join("turn.json")).unwrap();
        turn.snapshot = WorldSnapshot::new(initial, versions);
        store.write_json(&store.world_dir(world).join("turn.json"), &turn).unwrap();
        assert!(
            matches!(store.load(world), Err(StoreError::Invalid(message)) if message.contains("replay hash differs from snapshot"))
        );
        let _ = fs::remove_dir_all(&root);
    }
}
