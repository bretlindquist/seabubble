use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllowedAction {
    AllowOnce,
    ContinueWatched,
    Kill,
    LlmJudge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionCommand {
    pub action: AllowedAction,
    pub incident_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusEvent {
    pub surface_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionAck {
    pub incident_id: String,
    pub action: AllowedAction,
    pub accepted: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ControlMessage {
    Incident(Box<crate::Incident>),
    Focus(FocusEvent),
    DecisionAck(DecisionAck),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ActorContext, CapabilityRequest, CmuxContext, FilterResults, Incident, IncidentState,
        Severity,
    };

    #[test]
    fn allowed_actions_use_snake_case() {
        let command = DecisionCommand {
            action: AllowedAction::ContinueWatched,
            incident_id: "SI-1".to_string(),
        };

        let json = serde_json::to_string(&command).unwrap();
        assert!(json.contains("continue_watched"));

        let decoded: DecisionCommand = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded.action, AllowedAction::ContinueWatched));
    }

    #[test]
    fn control_messages_use_snake_case_tags() {
        let message = ControlMessage::DecisionAck(DecisionAck {
            incident_id: "SI-1".to_string(),
            action: AllowedAction::AllowOnce,
            accepted: true,
            message: None,
        });

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("\"type\":\"decision_ack\""));
        assert!(json.contains("\"action\":\"allow_once\""));
    }

    #[test]
    fn incident_payload_contains_swift_required_fields() {
        let message = ControlMessage::Incident(Box::new(Incident {
            incident_id: "SI-1".to_string(),
            actor: ActorContext {
                uid: 501,
                process: "codex".to_string(),
                agent_id: "agent-1".to_string(),
            },
            cmux: CmuxContext {
                workspace_id: "workspace:1".to_string(),
                surface_id: "surface:1".to_string(),
                socket_path: "/tmp/security-island/501/agents/agent-1/cmux.sock".to_string(),
            },
            request: CapabilityRequest {
                capability: "browser.eval".to_string(),
                payload: "document.cookie".to_string(),
                cwd: "https://internal.admin.panel".to_string(),
            },
            pid: 123,
            pgid: 123,
            state: IncidentState::PendingDecision,
            risk: 98,
            severity: Severity::Critical,
            reason: "browser_cookie_exfiltration".to_string(),
            rule_id: Some("SI-BROWSER-01".to_string()),
            evidence: vec!["document.cookie access".to_string()],
            filter_results: FilterResults {
                regex: Some("matched document.cookie".to_string()),
                bash_ast: None,
                magika: Some("not_applicable".to_string()),
                llm: None,
            },
            created_at: "2026-04-26T00:00:00Z".to_string(),
            allowed_actions: vec![AllowedAction::AllowOnce, AllowedAction::Kill],
        }));

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("\"incident_id\""));
        assert!(json.contains("\"pid\""));
        assert!(json.contains("\"pgid\""));
        assert!(json.contains("\"state\":\"pending_decision\""));
        assert!(json.contains("\"severity\":\"critical\""));
        assert!(json.contains("\"filter_results\""));
        assert!(json.contains("\"allowed_actions\""));
    }
}
