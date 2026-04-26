pub mod magika;
pub mod secrets;
pub mod shell_policy;

use plugin_api::{
    candidate_precedes, CapabilityRequest, FindingEffect, PolicyDecision, Scanner,
};

pub struct Pipeline {
    scanners: Vec<Box<dyn Scanner>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            scanners: vec![
                Box::new(secrets::SecretScanner::new()),
                Box::new(shell_policy::ShellPolicyScanner::new()),
                Box::new(magika::MagikaScanner::new()),
            ],
        }
    }

    pub fn classify(&self, req: &CapabilityRequest) -> PolicyDecision {
        let mut findings = self
            .scanners
            .iter()
            .flat_map(|scanner| scanner.scan(req))
            .collect::<Vec<_>>();

        findings.sort_by(|left, right| {
            if candidate_precedes(left, right) {
                std::cmp::Ordering::Less
            } else if candidate_precedes(right, left) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });

        let mut findings_iter = findings.into_iter();
        let Some(mut primary) = findings_iter.next() else {
            return PolicyDecision::Allow {
                reason: "request passed all scanners",
            };
        };

        for extra in findings_iter {
            if extra.rule_id != primary.rule_id {
                primary
                    .evidence
                    .push(format!("additional finding [{}]: {}", extra.rule_id, extra.reason));
            }
        }

        match primary.effect() {
            FindingEffect::Block => PolicyDecision::Block(primary),
            FindingEffect::Watch => PolicyDecision::Watch(primary),
        }
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plugin_api::{extract_artifact_refs, IncidentState, ScannerStage, Severity};

    fn template(effect: FindingEffect, risk: u8, stage: ScannerStage) -> IncidentTemplate {
        IncidentTemplate {
            stage,
            state: match effect {
                FindingEffect::Watch => IncidentState::Watch,
                FindingEffect::Block => IncidentState::PendingDecision,
            },
            severity: Severity::Medium,
            reason: "test",
            rule_id: "TEST-01",
            risk,
            evidence: vec![],
            regex: None,
            bash_ast: None,
            magika: None,
            allowed_actions: vec![],
        }
    }

    fn request(capability: &str, payload: &str, cwd: &str) -> CapabilityRequest {
        CapabilityRequest {
            capability: capability.to_string(),
            payload: payload.to_string(),
            cwd: cwd.to_string(),
        }
    }

    #[test]
    fn block_beats_watch() {
        let block = template(FindingEffect::Block, 50, ScannerStage::PreForwardBlocking);
        let watch = template(FindingEffect::Watch, 99, ScannerStage::PreForwardBlocking);
        assert!(candidate_precedes(&block, &watch));
    }

    #[test]
    fn higher_stage_beats_lower_with_same_effect() {
        let fast = template(FindingEffect::Watch, 50, ScannerStage::PreForwardFast);
        let artifact = template(FindingEffect::Watch, 50, ScannerStage::ArtifactInspect);
        assert!(candidate_precedes(&artifact, &fast));
    }

    #[test]
    fn higher_risk_breaks_ties() {
        let stronger = template(FindingEffect::Watch, 70, ScannerStage::PreForwardFast);
        let weaker = template(FindingEffect::Watch, 60, ScannerStage::PreForwardFast);
        assert!(candidate_precedes(&stronger, &weaker));
    }

    #[test]
    fn resolves_relative_artifacts_against_cwd() {
        let refs = extract_artifact_refs(&request(
            "terminal.exec",
            "chmod +x ./tool",
            "/tmp/workspace",
        ));
        assert_eq!(refs[0].resolved_path, "/tmp/workspace/tool");
    }

    #[test]
    fn resolves_parent_relative_artifacts_against_cwd() {
        let refs = extract_artifact_refs(&request(
            "terminal.read_file",
            "cat ../config/.env",
            "/tmp/workspace/app",
        ));
        assert_eq!(refs[0].resolved_path, "/tmp/workspace/config/.env");
    }

    #[test]
    fn ignores_flags_and_operators() {
        let refs = extract_artifact_refs(&request(
            "terminal.exec",
            "cat ./file.txt | grep token -n",
            "/tmp/workspace",
        ));
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].resolved_path, "/tmp/workspace/file.txt");
    }

    #[test]
    fn ignores_urls() {
        let refs = extract_artifact_refs(&request(
            "terminal.exec",
            "curl https://example.com/tool.sh",
            "/tmp/workspace",
        ));
        assert!(refs.is_empty());
    }

    #[test]
    fn resolves_bare_relative_artifacts_against_cwd() {
        let refs = extract_artifact_refs(&request("terminal.exec", "python payload", "/tmp/workspace"));
        assert_eq!(refs[0].resolved_path, "/tmp/workspace/payload");
    }

    #[test]
    fn strips_attached_shell_punctuation() {
        let refs = extract_artifact_refs(&request(
            "terminal.exec",
            "chmod +x ./tool;",
            "/tmp/workspace",
        ));
        assert_eq!(refs[0].resolved_path, "/tmp/workspace/tool");
    }

    #[test]
    fn returns_no_artifacts_for_ls() {
        let refs = extract_artifact_refs(&request("terminal.exec", "ls -la", "/tmp/workspace"));
        assert!(refs.is_empty());
    }
}
