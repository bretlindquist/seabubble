pub mod control;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentState {
    Safe,
    Watch,
    PendingDecision,
    QueuedForLlm,
    ResolvedAllowed,
    ContinuedWatched,
    Killed,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterResults {
    pub regex: Option<String>,
    pub bash_ast: Option<String>,
    pub magika: Option<String>,
    pub llm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub capability: String,
    pub payload: String,
    pub cwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    pub uid: u32,
    pub process: String,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmuxContext {
    pub workspace_id: String,
    pub surface_id: String,
    pub socket_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub incident_id: String,
    pub actor: ActorContext,
    pub cmux: CmuxContext,
    pub request: CapabilityRequest,
    pub pid: i32,
    pub pgid: i32,
    pub state: IncidentState,
    pub risk: u8,
    pub severity: Severity,
    pub reason: String,
    pub rule_id: Option<String>,
    pub evidence: Vec<String>,
    pub filter_results: FilterResults,
    pub created_at: String,
    pub allowed_actions: Vec<control::AllowedAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIdentifyParams {
    pub agent_id: String,
    pub session_nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIdentifyHandshake {
    pub method: String,
    pub params: SecurityIdentifyParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityRecord {
    pub agent_id: String,
    pub session_nonce: String,
    pub uid: u32,
    pub socket_dir: String,
    pub socket_path: String,
    pub pid: Option<u32>,
    pub pgid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum RegistrationMessage {
    RegisterAgent(IdentityRecord),
    AgentStarted(IdentityRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationAck {
    pub accepted: bool,
    pub message: Option<String>,
}

pub fn hash_session_nonce(session_nonce: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(session_nonce.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(target_os = "macos")]
pub mod darwin;
