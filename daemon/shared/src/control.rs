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
