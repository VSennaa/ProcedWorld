# pw-server

Authoritative server (ADR-0001): `GET /health` and a versioned JSON WebSocket at `/ws`
(docs/sdd/10-protocolo.md). Rules stay in `pw-engine`; this crate validates, orders, stores and
broadcasts. Turns have no clock (ADR-0008): a turn resolves when every present human is Ready;
civilizations without a present human (bots, absent humans) are played by their Governor.

## Run

`PW_SERVER_BIND` (default `127.0.0.1:8100`, never `0.0.0.0` by default), `PW_RECONNECT_GRACE_SECS`
(default 60, technical grace, not a game clock), `PW_MAX_WORLDS` (16), `PW_MAX_CIVS` (16), and optional
`PW_DATA_DIR`. Without `PW_DATA_DIR`, worlds remain in memory only. With it, each world is stored in
its own directory as `initial_snapshot.json`, `turn.json`, and `metadata.json`; `turn.json` contains
the log and its resulting snapshot as one unit, replaced via a temporary sibling and rename. Startup replays every sealed log and refuses any world whose replay
does not match its sealed hash. Session tokens are stored in that same world metadata, so `Entrar`
can resume a seat after a restart.

## Protocol v1

Frame: `{"protocol_version":"1.0","request_id":"..","type":"..","payload":{..}}`. The version is text
(`"major.minor"`); any `1.x` is accepted, a number or another major yields
`error/protocol_version_mismatch`. Replies echo `request_id`; pushes use `null`.

Client to server: `create_world {seed, civs}`, `join {world_id, civ, session_token?}`,
`get_snapshot`, `submit_command {command}` (engine `CommandPayload` JSON), `ready`, `unready`.
Server to client: `world_created`, `joined` (returns `session_token`), `catalog` (push right after
`joined`: `{hash, versions, technologies[{id,name,branch,cost,prerequisites}], units[{id,name,role,movement,strength,requires_technology}], event_templates[{id,name,category,narrative?,choices[{id,label}]}], improvements[{id,name,cost,requires_technology,biomes}]}`),
`state_snapshot`,
`command_accepted {command_id, accepted_sequence, turn}`, `ready_state {turn, present, ready}`,
`turn_diff`, `error {reason, detail, engine_reason}`.

Error reasons: `protocol_version_mismatch`, `malformed_message`, `unknown_message_type`,
`world_not_found`, `world_exists`, `too_many_worlds`, `invalid_world_params`, `not_joined`,
`already_joined`, `civ_not_found`, `civ_taken`, `invalid_session_token`, `already_ready`,
`units_awaiting_orders` (`ready` refused: `detail` is the comma-separated idle unit ids, `unit_ids` the same as numbers),
`command_rejected` (with the engine `RejectionReason` in `engine_reason`), `internal`.

A command is validated by a dry-run of the open turn in the engine; rejected commands change
nothing. Unit orders are ordinary commands: `{"type":"set_unit_order","data":{"unit_id":3,"order":{"type":"move_to","data":{"target":42}}}}`
(orders `idle`, `fortify`, `explore`, `move_to`) and `{"type":"skip_unit","data":{"unit_id":3}}`. The view carries `idle_units`
and, per own unit, `order` and `skipped_turn` (pending orders of the open turn included); foreign units never show them.
Accepted ones get the next `accepted_sequence`, go to the command log, and are applied by
`step` when the turn resolves. After `ready`, orders are locked until `unready`.

## Dependencies (ADR-0007)

- `axum` (with `ws`), `tokio`, `serde`, `serde_json`: approved stack.
- `pw-harness`: reused for initial world construction (`initial_world`) and the seeded
  rotating civilization order, so server and harness cannot drift.
- Dev only: `tokio-tungstenite` (real WebSocket client in tests) and `futures-util` (its
  stream/sink traits).

## Storage

`WorldStore` has an in-memory implementation and `FileStore`. Both retain the initial snapshot,
append-only accepted commands, and a sealed hash per turn. `FileStore` reconstructs the head only by
replay, never trusting a mutable live-state file. PostgreSQL (docs/sdd/09-persistencia.md) replaces
the file adapter later.

## Known gaps

- `turn_diff` carries the civilization's filtered full view plus its own command results, not a
  true delta: the engine has no delta representation yet. Chunked map delivery is not implemented.
- Visibility filtering uses the engine `visibility` map (terrain for visible/remembered tiles;
  foreign cities and units only on visible tiles). The ledger and other civilizations' internals are
  never sent. Fog quality depends on the engine's `resolve_visibility`.
- No real authentication: `create_world` is open, and the session token is an OS-seeded random
  string (not cryptographic), persisted only to let a seat resume after restart.
- No per-connection rate limiting; only a 64 KiB message cap and a bounded outbox (slow clients
  drop pushes and recover with `get_snapshot`).
- The Governor uses a fixed `GrowCautiously` mandate; player-configured Mandates, T1/T2 ports,
  notifications and the Entropy director are not wired in.
- World generation reads catalogs through the harness path baked in at compile time.
