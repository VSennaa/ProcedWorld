//! World persistence behind a trait. PostgreSQL comes later (docs/sdd/09-persistencia.md);
//! the in-memory implementation keeps the same event-sourced shape: an initial snapshot, the
//! append-only accepted-command log with one sealed hash per resolved turn, and the head state.

use std::{collections::{BTreeMap, HashMap}, sync::Mutex};

use pw_engine::{
    hash::StateHash,
    ids::{CivId, TileIndex, TurnNumber},
    world::{AcceptedCommand, CommandLog, WorldId, WorldSnapshot, WorldState},
};

#[derive(Clone, Debug)]
pub struct WorldRecord {
    pub snapshot: WorldSnapshot,
    pub log: CommandLog,
    pub home_tiles: BTreeMap<CivId, TileIndex>,
    /// Current head state (derivable by replaying `log` from `snapshot`).
    pub state: WorldState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    AlreadyExists,
    NotFound,
}

pub trait WorldStore: Send + Sync {
    fn create(&self, record: WorldRecord) -> Result<(), StoreError>;
    /// Appends a command accepted in the open turn.
    fn append_command(&self, world: WorldId, command: &AcceptedCommand) -> Result<(), StoreError>;
    /// Atomically appends the Governor commands of the turn, seals the turn hash and moves the head.
    fn commit_turn(&self, world: WorldId, commands: &[AcceptedCommand], sealed_turn: TurnNumber, hash: StateHash, next: &WorldState) -> Result<(), StoreError>;
    fn load(&self, world: WorldId) -> Result<Option<WorldRecord>, StoreError>;
    fn list(&self) -> Result<Vec<WorldId>, StoreError>;
}

#[derive(Default)]
pub struct InMemoryStore {
    worlds: Mutex<HashMap<WorldId, WorldRecord>>,
}

impl InMemoryStore {
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<WorldId, WorldRecord>> {
        self.worlds.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
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
        self.lock().get_mut(&world).ok_or(StoreError::NotFound)?.log.append(command.clone());
        Ok(())
    }

    fn commit_turn(&self, world: WorldId, commands: &[AcceptedCommand], sealed_turn: TurnNumber, hash: StateHash, next: &WorldState) -> Result<(), StoreError> {
        let mut worlds = self.lock();
        let record = worlds.get_mut(&world).ok_or(StoreError::NotFound)?;
        for command in commands {
            record.log.append(command.clone());
        }
        record.log.record_turn(sealed_turn, hash);
        record.state = next.clone();
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
