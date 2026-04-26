use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AllowedAction {
    AllowOnce,
    ContinueWatched,
    Kill,
    LlmJudge,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecisionCommand {
    pub action: AllowedAction,
    pub incident_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FocusEvent {
    pub surface_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ControlMessage {
    Incident(crate::Incident),
    Focus(FocusEvent),
}
