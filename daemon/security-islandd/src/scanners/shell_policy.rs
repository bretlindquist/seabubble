use crate::scanners::{IncidentTemplate, Scanner};
use shared::{CapabilityRequest, IncidentState, Severity, control::AllowedAction};
use regex::Regex;

pub struct ShellPolicyScanner {
    regex: Regex,
}

impl ShellPolicyScanner {
    pub fn new() -> Self {
        let pattern = r"curl\s+.*\|\s*(?:bash|sh)|wget\s+.*\|\s*(?:bash|sh)|chmod\s+\+x|sudo\s+|rm\s+-rf|launchctl|osascript|ssh\s+";
        Self {
            regex: Regex::new(pattern).unwrap(),
        }
    }
}

impl Default for ShellPolicyScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner for ShellPolicyScanner {
    fn scan(&self, req: &CapabilityRequest) -> Option<IncidentTemplate> {
        if (req.capability == "terminal.exec" || req.capability == "terminal.send_text") 
            && self.regex.is_match(&req.payload) 
        {
            return Some(IncidentTemplate {
                    state: IncidentState::PendingDecision,
                    severity: Severity::High,
                    reason: "shell request matched high-risk execution pattern",
                    rule_id: "SI-TERM-01",
                    risk: 90,
                    evidence: vec![format!("terminal payload matched high-risk shell pattern: {}", req.payload)],
                    regex: Some(self.regex.as_str().to_string()),
                    allowed_actions: vec![
                        AllowedAction::AllowOnce,
                        AllowedAction::ContinueWatched,
                        AllowedAction::Kill,
                        AllowedAction::LlmJudge,
                    ],
                });
        }
        None
    }
}
