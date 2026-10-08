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

use crate::session;

#[derive(Clone, Debug)]
pub struct WorldRecord {
    /// Immutable starting point; `state` is derived from this plus the sealed log.
    pub snapshot: WorldSnapshot,
    pub log: CommandLog,
    pub home_tiles: BTreeMap<CivId, TileIndex>,
    /// Per-seat salted verifiers (`session::make_verifier`), never the tokens themselves.
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
    fn staging_dir(&self, world: WorldId) -> PathBuf {
        self.root.join(format!("{}.creating", world.0))
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
    /// Writes the three world files into `dir` (a not yet published directory).
    fn write_record(&self, dir: &Path, record: &WorldRecord) -> Result<(), StoreError> {
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
/// Best effort durability for a directory entry (not supported on every platform).
fn sync_dir(path: &Path) {
    if let Ok(dir) = fs::File::open(path) {
        let _ = dir.sync_all();
    }
}
fn io_error(error: io::Error) -> StoreError {
    StoreError::Io(error.to_string())
}

impl WorldStore for FileStore {
    /// Snapshot, turn and metadata are written to `<id>.creating` and published by one directory
    /// rename, so a crash leaves either no world or a complete one.
    fn create(&self, record: WorldRecord) -> Result<(), StoreError> {
        let dir = self.world_dir(record.state.world_id);
        if dir.exists() {
            return Err(StoreError::AlreadyExists);
        }
        let staging = self.staging_dir(record.state.world_id);
        if staging.exists() {
            fs::remove_dir_all(&staging).map_err(io_error)?;
        }
        fs::create_dir(&staging).map_err(io_error)?;
        let written = self.write_record(&staging, &record).and_then(|()| {
            sync_dir(&staging);
            fs::rename(&staging, &dir).map_err(io_error)
        });
        if written.is_err() {
            let _ = fs::remove_dir_all(&staging);
        }
        written?;
        sync_dir(&self.root);
        Ok(())
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
        if ["initial_snapshot.json", "turn.json", "metadata.json"]
            .iter()
            .any(|name| !dir.join(name).is_file())
        {
            eprintln!(
                "ignoring incomplete world directory {} (interrupted creation)",
                world.0
            );
            return Ok(None);
        }
        let snapshot: WorldSnapshot = self.read_json(&dir.join("initial_snapshot.json"))?;
        let turn: TurnState = self.read_json(&dir.join("turn.json"))?;
        let head = turn.snapshot;
        let log = turn.log;
        let mut metadata: Metadata = self.read_json(&dir.join("metadata.json"))?;
        // Migration: files written before verifiers existed hold the plaintext token. Convert them
        // in place; the original token keeps working because the verifier is derived from it.
        if metadata.session_tokens.values().any(|v| !session::is_verifier(v)) {
            for value in metadata.session_tokens.values_mut() {
                if !session::is_verifier(value) {
                    *value = session::make_verifier(value);
                }
            }
            self.write_json(&dir.join("metadata.json"), &metadata)?;
            eprintln!("migrated plaintext session tokens of world {} to verifiers", world.0);
        }
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
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Ok(id) = name.parse::<u64>() {
                worlds.push(WorldId(id));
            } else if name.ends_with(".creating") {
                eprintln!("ignoring interrupted world creation {name}");
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
        let tokens = BTreeMap::from([(CivId(0), session::make_verifier("resume-token"))]);
        store.save_session_tokens(world, &tokens).unwrap();
        assert_eq!(store.load(world).unwrap().unwrap().session_tokens, tokens);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn plaintext_tokens_are_migrated_on_load_and_still_verify() {
        let root = test_dir("migrate");
        let _ = fs::remove_dir_all(&root);
        let store = FileStore::open(&root).unwrap();
        let (record, _) = record(94);
        let world = record.state.world_id;
        store.create(record).unwrap();
        // Simulate a legacy file: plaintext token in metadata.
        let path = store.world_dir(world).join("metadata.json");
        let mut metadata: Metadata = store.read_json(&path).unwrap();
        metadata.session_tokens.insert(CivId(0), "legacy-plain-token".to_owned());
        store.write_json(&path, &metadata).unwrap();

        let loaded = store.load(world).unwrap().unwrap();
        let verifier = &loaded.session_tokens[&CivId(0)];
        assert!(session::is_verifier(verifier));
        assert!(session::verify("legacy-plain-token", verifier));
        let on_disk = fs::read_to_string(&path).unwrap();
        assert!(!on_disk.contains("legacy-plain-token"), "token must be gone from disk");
        // Second load is stable (already migrated).
        assert_eq!(store.load(world).unwrap().unwrap().session_tokens[&CivId(0)], *verifier);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn interrupted_world_creation_is_ignored_and_can_be_retried() {
        let root = test_dir("creating");
        let _ = fs::remove_dir_all(&root);
        let store = FileStore::open(&root).unwrap();
        let (record, _) = record(95);
        let world = record.state.world_id;
        // Crash during creation: staging directory with only some files.
        let staging = store.staging_dir(world);
        fs::create_dir(&staging).unwrap();
        fs::write(staging.join("initial_snapshot.json"), b"{}").unwrap();
        assert!(store.list().unwrap().is_empty());
        assert!(store.load(world).unwrap().is_none());
        // Old-style half-written directory under the final name is skipped, not fatal.
        let half = store.world_dir(WorldId(777));
        fs::create_dir(&half).unwrap();
        fs::write(half.join("initial_snapshot.json"), b"{}").unwrap();
        assert!(store.load(WorldId(777)).unwrap().is_none());
        // Retrying the creation succeeds and publishes all files at once.
        store.create(record).unwrap();
        assert!(!staging.exists());
        assert!(store.load(world).unwrap().is_some());
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
