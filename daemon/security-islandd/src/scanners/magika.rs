use crate::scanners::{IncidentTemplate, Scanner, ScannerStage};
use shared::{CapabilityRequest, IncidentState, Severity, control::AllowedAction};
use std::process::Command;

pub struct MagikaScanner;

impl MagikaScanner {
    pub fn new() -> Self {
        Self
    }
    
    // Simplistic extraction: split by space, look for things that might be paths
    fn extract_paths(payload: &str) -> Vec<String> {
        payload
            .split_whitespace()
            .filter(|s| s.starts_with('/') || s.starts_with('.') || s.starts_with('~'))
            .map(|s| s.trim_matches(|c| c == '\'' || c == '"').to_string())
            .collect()
    }

    fn run_magika(path: &str) -> Option<String> {
        let output = Command::new("magika")
            .arg(path)
            .output()
            .ok()?;
            
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Some(stdout.trim().to_string())
        } else {
            None
        }
    }
}

impl Scanner for MagikaScanner {
    fn scan(&self, req: &CapabilityRequest) -> Vec<IncidentTemplate> {
        let paths = Self::extract_paths(&req.payload);
        let mut findings = Vec::new();

        for path in paths {
            if let Some(magika_output) = Self::run_magika(&path) {
                // Heuristic: If magika detects an executable or something suspicious
                // For demonstration, we'll watch if magika gives any output, or specifically look for binaries
                if magika_output.contains("executable") || magika_output.contains("mach-o") || magika_output.contains("elf") {
                    findings.push(IncidentTemplate {
                        stage: ScannerStage::PreForwardBlocking,
                        state: IncidentState::PendingDecision,
                        severity: Severity::High,
                        reason: "Magika detected executable file",
                        rule_id: "SI-MAGIKA-01",
                        risk: 80,
                        evidence: vec![format!("Magika scan result: {}", magika_output)],
                        regex: None,
                        bash_ast: None,
                        allowed_actions: vec![
                            AllowedAction::AllowOnce,
                            AllowedAction::Kill,
                        ],
                    });
                } else if magika_output.contains("secret") || magika_output.contains("key") {
                     findings.push(IncidentTemplate {
                        stage: ScannerStage::PreForwardBlocking,
                        state: IncidentState::PendingDecision,
                        severity: Severity::High,
                        reason: "Magika detected secret/key file",
                        rule_id: "SI-MAGIKA-02",
                        risk: 75,
                        evidence: vec![format!("Magika scan result: {}", magika_output)],
                        regex: None,
                        bash_ast: None,
                        allowed_actions: vec![
                            AllowedAction::AllowOnce,
                            AllowedAction::Kill,
                        ],
                    });
                }
            }
        }
        findings
    }
}
