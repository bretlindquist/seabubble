use crate::scanners::{IncidentTemplate, Scanner, ScannerStage};
use regex::Regex;
use shared::{control::AllowedAction, CapabilityRequest, IncidentState, Severity};

pub struct SecretScanner {
    browser_secret_regex: Regex,
    terminal_secret_regex: Regex,
}

impl SecretScanner {
    pub fn new() -> Self {
        Self {
            browser_secret_regex: Regex::new(
                r"(?i)document\.cookie|localstorage|sessionstorage|\bbearer\b\s+|api_key|\btoken\b",
            )
            .unwrap(),
            terminal_secret_regex: Regex::new(
                r"(?i)\.env|\.ssh|id_rsa|id_ed25519|/etc/passwd|aws_secret_access_key|ghp_[0-9a-zA-Z]{36}|\bbearer\b\s+|api_key|\btoken\b",
            )
            .unwrap(),
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
        if req.capability == "browser.eval" && self.browser_secret_regex.is_match(&req.payload) {
            return Some(IncidentTemplate {
                stage: ScannerStage::PreForwardBlocking,
                state: IncidentState::PendingDecision,
                severity: Severity::Critical,
                reason: "browser script targets browser-held secrets",
                rule_id: "SI-BROWSER-01",
                risk: 98,
                evidence: vec![format!(
                    "browser.eval payload matched secret access pattern: {}",
                    req.payload
                )],
                regex: Some(self.browser_secret_regex.as_str().to_string()),
                bash_ast: None,
                allowed_actions: vec![
                    AllowedAction::AllowOnce,
                    AllowedAction::ContinueWatched,
                    AllowedAction::Kill,
                    AllowedAction::LlmJudge,
                ],
            });
        }

        if matches!(
            req.capability.as_str(),
            "terminal.read_file" | "terminal.exec" | "terminal.send_text"
        ) && self.terminal_secret_regex.is_match(&req.payload)
        {
            return Some(IncidentTemplate {
                stage: ScannerStage::PreForwardFast,
                state: IncidentState::Watch,
                severity: Severity::Medium,
                reason: "request touched a sensitive file or token pattern",
                rule_id: "SI-DATA-01",
                risk: 65,
                evidence: vec![format!(
                    "payload matched sensitive path or token indicator: {}",
                    req.payload
                )],
                regex: Some(self.terminal_secret_regex.as_str().to_string()),
                bash_ast: None,
                allowed_actions: vec![AllowedAction::ContinueWatched, AllowedAction::Kill],
            });
        }

        if req.capability == "browser.eval" {
            return Some(IncidentTemplate {
                stage: ScannerStage::PreForwardFast,
                state: IncidentState::Watch,
                severity: Severity::Medium,
                reason: "browser script execution is allowed only under watch in MVP policy",
                rule_id: "SI-BROWSER-00",
                risk: 55,
                evidence: vec![format!(
                    "browser.eval executed without secret-match payload: {}",
                    req.payload
                )],
                regex: None,
                bash_ast: None,
                allowed_actions: vec![AllowedAction::ContinueWatched, AllowedAction::Kill],
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(capability: &str, payload: &str) -> CapabilityRequest {
        CapabilityRequest {
            capability: capability.to_string(),
            payload: payload.to_string(),
            cwd: "/tmp".to_string(),
        }
    }

    #[test]
    fn blocks_browser_cookie_access() {
        let scanner = SecretScanner::new();
        let finding = scanner.scan(&request("browser.eval", "console.log(document.cookie)"));
        let finding = finding.expect("expected finding");
        assert_eq!(finding.rule_id, "SI-BROWSER-01");
        assert!(matches!(finding.state, IncidentState::PendingDecision));
        assert!(finding.regex.is_some());
    }

    #[test]
    fn watches_dotenv_reads() {
        let scanner = SecretScanner::new();
        let finding = scanner.scan(&request("terminal.read_file", "cat .env"));
        let finding = finding.expect("expected finding");
        assert_eq!(finding.rule_id, "SI-DATA-01");
        assert!(matches!(finding.state, IncidentState::Watch));
    }

    #[test]
    fn watches_ssh_key_reads() {
        let scanner = SecretScanner::new();
        let finding = scanner.scan(&request("terminal.read_file", "cat ~/.ssh/id_rsa"));
        let finding = finding.expect("expected finding");
        assert_eq!(finding.rule_id, "SI-DATA-01");
    }

    #[test]
    fn ignores_benign_terminal_payload() {
        let scanner = SecretScanner::new();
        assert!(scanner.scan(&request("terminal.exec", "ls -la")).is_none());
    }
}
