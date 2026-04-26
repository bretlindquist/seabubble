use std::process::Command;

use plugin_api::{extract_artifact_refs, IncidentTemplate, Scanner, ScannerStage};
use shared::{control::AllowedAction, CapabilityRequest, IncidentState, Severity};

pub struct MagikaScanner;

impl MagikaScanner {
    pub fn new() -> Self {
        Self
    }

    fn run_magika(path: &str) -> Option<String> {
        let output = Command::new("magika").arg(path).output().ok()?;
        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn classify_output(output: &str) -> Option<(IncidentState, Severity, u8, &'static str)> {
        let normalized = output.to_lowercase();

        if normalized.contains("mach-o")
            || normalized.contains("elf")
            || normalized.contains("pe32")
            || normalized.contains("executable")
        {
            return Some((
                IncidentState::Watch,
                Severity::High,
                75,
                "magika classified an executable artifact",
            ));
        }

        if normalized.contains("shell script")
            || normalized.contains("javascript")
            || normalized.contains("python")
        {
            return Some((
                IncidentState::Watch,
                Severity::Medium,
                60,
                "magika classified an active code artifact",
            ));
        }

        None
    }
}

impl Default for MagikaScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner for MagikaScanner {
    fn scan(&self, req: &CapabilityRequest) -> Vec<IncidentTemplate> {
        let mut findings = Vec::new();

        for artifact in extract_artifact_refs(req) {
            let Some(output) = Self::run_magika(&artifact.resolved_path) else {
                continue;
            };
            let Some((state, severity, risk, reason)) = Self::classify_output(&output) else {
                continue;
            };

            findings.push(IncidentTemplate {
                stage: ScannerStage::ArtifactInspect,
                state,
                severity,
                reason,
                rule_id: "SI-MAGIKA-01",
                risk,
                evidence: vec![format!(
                    "magika classified artifact {} from {}",
                    artifact.resolved_path, artifact.original
                )],
                regex: None,
                bash_ast: None,
                magika: Some(output),
                allowed_actions: vec![AllowedAction::ContinueWatched, AllowedAction::Kill],
            });
        }

        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(payload: &str, cwd: &str) -> CapabilityRequest {
        CapabilityRequest {
            capability: "terminal.exec".to_string(),
            payload: payload.to_string(),
            cwd: cwd.to_string(),
        }
    }

    #[test]
    fn returns_no_findings_when_no_artifacts_exist() {
        let scanner = MagikaScanner::new();
        assert!(scanner.scan(&request("ls -la", "/tmp")).is_empty());
    }

    #[test]
    fn classifies_executable_output() {
        let classified = MagikaScanner::classify_output("Mach-O 64-bit executable");
        let (state, severity, risk, _) = classified.expect("expected classification");
        assert!(matches!(state, IncidentState::Watch));
        assert!(matches!(severity, Severity::High));
        assert_eq!(risk, 75);
    }

    #[test]
    fn ignores_plain_text_output() {
        assert!(MagikaScanner::classify_output("ASCII text").is_none());
    }
}
