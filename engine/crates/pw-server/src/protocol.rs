//! Versioned JSON envelope and message payloads (docs/sdd/10-protocolo.md).
//!
//! Every frame is `{"protocol_version", "request_id", "type", "payload"}`. Server replies echo
//! the request id sent by the client; unsolicited pushes carry `request_id: null`.

use pw_engine::world::{CommandPayload, RejectionReason};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Text version, `"major.minor"`. A client with the same major is accepted (docs/sdd/10 section 3).
pub const PROTOCOL_VERSION: &str = "1.0";

/// True when `version` is a `"major.minor"` (or bare `"major"`) text whose major is ours.
pub fn version_compatible(version: &str) -> bool {
    let major = |text: &str| text.split('.').next().and_then(|part| part.parse::<u32>().ok());
    major(version).is_some() && major(version) == major(PROTOCOL_VERSION)
}

/// Incoming frame before the payload is decoded (the version is checked first).
#[derive(Debug, Deserialize)]
pub struct Envelope {
    pub protocol_version: String,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub payload: Value,
}

/// Enumerated error reasons sent in `error` frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorReason {
    ProtocolVersionMismatch,
    MalformedMessage,
    UnknownMessageType,
    WorldNotFound,
    WorldExists,
    TooManyWorlds,
    InvalidWorldParams,
    NotJoined,
    AlreadyJoined,
    CivNotFound,
    CivTaken,
    InvalidSessionToken,
    AlreadyReady,
    UnitsAwaitingOrders,
    CommandRejected,
    Internal,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorld {
    pub seed: u64,
    pub civs: u32,
}

#[derive(Debug, Deserialize)]
pub struct Join {
    pub world_id: u64,
    pub civ: u32,
    #[serde(default)]
    pub session_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitCommand {
    pub command: CommandPayload,
}

/// Builds a serialized outgoing frame.
pub fn frame(request_id: Option<&str>, kind: &str, payload: Value) -> String {
    json!({
        "protocol_version": PROTOCOL_VERSION,
        "request_id": request_id,
        "type": kind,
        "payload": payload,
    })
    .to_string()
}

/// `units_awaiting_orders`: `detail` is the comma-separated unit ids, `unit_ids` the same as numbers.
pub fn units_awaiting_frame(request_id: Option<&str>, units: &[pw_engine::ids::UnitId]) -> String {
    let ids: Vec<u32> = units.iter().map(|unit| unit.0).collect();
    let detail = ids.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
    frame(request_id, "error", json!({ "reason": ErrorReason::UnitsAwaitingOrders, "detail": detail, "engine_reason": null, "unit_ids": ids }))
}

pub fn error_frame(request_id: Option<&str>, reason: ErrorReason, detail: &str, engine_reason: Option<RejectionReason>) -> String {
    frame(request_id, "error", json!({ "reason": reason, "detail": detail, "engine_reason": engine_reason }))
}
