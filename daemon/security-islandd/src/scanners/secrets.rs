use crate::scanners::{IncidentTemplate, Scanner};
use shared::{CapabilityRequest, IncidentState, Severity, control::AllowedAction};
use regex::Regex;

pub struct SecretScanner {
    regex: Regex,
}

impl SecretScanner {
    pub fn new() -> Self {
        let pattern = r"(?i)\.env|\.ssh|id_rsa|id_ed25519|/etc/passwd|aws_secret_access_key|ghp_[0-9a-zA-Z]{36}|document\.cookie|localStorage|sessionStorage|bearer|api_key|token";
        Self {
            regex: Regex::new(pattern).unwrap(),
        }
    }
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner for SecretScanner {
    fn scan(&self, req: &CapabilityRequest) -> Option<IncidentTemplate> {
        if req.capability == "browser.eval" && self.regex.is_match(&req.payload) {
             return Some(IncidentTemplate {
                state: IncidentState::PendingDecision,
                severity: Severity::Critical,
                reason: "browser script targets browser-held secrets",
                rule_id: "SI-BROWSER-01",
                risk: 98,
                evidence: vec![format!("browser.eval payload matched secret access pattern: {}", req.payload)],
                regex: Some(self.regex.as_str().to_string()),
                allowed_actions: vec![
                    AllowedAction::AllowOnce,
                    AllowedAction::ContinueWatched,
                    AllowedAction::Kill,
                    AllowedAction::LlmJudge,
                ],
            });
        }

        if (req.capability == "terminal.read_file" || req.capability == "terminal.exec" || req.capability == "terminal.send_text") 
            && self.regex.is_match(&req.payload) {
             return Some(IncidentTemplate {
                state: IncidentState::Watch,
                severity: Severity::Medium,
                reason: "request touched a sensitive file or token pattern",
                rule_id: "SI-DATA-01",
                risk: 65,
                evidence: vec![format!("payload matched sensitive path or token indicator: {}", req.payload)],
                regex: Some(self.regex.as_str().to_string()),
                allowed_actions: vec![AllowedAction::ContinueWatched, AllowedAction::Kill],
            });
        }

        if req.capability == "browser.eval" {
            return Some(IncidentTemplate {
                state: IncidentState::Watch,
                severity: Severity::Medium,
                reason: "browser script execution is allowed only under watch in MVP policy",
                rule_id: "SI-BROWSER-00",
                risk: 55,
                evidence: vec![format!("browser.eval executed without secret-match payload: {}", req.payload)],
                regex: None,
                allowed_actions: vec![AllowedAction::ContinueWatched, AllowedAction::Kill],
            });
        }

        None
    }
}
