use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub capability: String,
    pub payload: String,
    pub cwd: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorContext {
    pub uid: u32,
    pub process: String,
    pub agent_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmuxContext {
    pub workspace_id: String,
    pub surface_id: String,
    pub socket_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Incident {
    pub incident_id: String,
    pub actor: ActorContext,
    pub cmux: CmuxContext,
    pub request: CapabilityRequest,
}
