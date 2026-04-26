use std::path::{Component, Path, PathBuf};

use shared::{control::AllowedAction, CapabilityRequest, IncidentState, Severity};

pub mod magika;
pub mod secrets;
pub mod state;
pub mod shell_policy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScannerStage {
    PreForwardBlocking,
    PreForwardFast,
    ArtifactInspect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingEffect {
    Watch,
    Block,
}

#[derive(Debug)]
pub enum PolicyDecision {
    Allow { reason: &'static str },
    Watch(IncidentTemplate),
    Block(IncidentTemplate),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRef {
    pub original: String,
    pub resolved_path: String,
    pub source_capability: String,
}

#[derive(Debug)]
pub struct IncidentTemplate {
    pub stage: ScannerStage,
    pub state: IncidentState,
    pub severity: Severity,
    pub reason: &'static str,
    pub rule_id: &'static str,
    pub risk: u8,
    pub evidence: Vec<String>,
    pub regex: Option<String>,
    pub bash_ast: Option<String>,
    pub magika: Option<String>,
    pub allowed_actions: Vec<AllowedAction>,
}

pub trait Scanner: Send + Sync {
    fn scan(&self, req: &CapabilityRequest) -> Vec<IncidentTemplate>;
}

pub struct Pipeline {
    scanners: Vec<Box<dyn Scanner>>,
}

impl Pipeline {
    #[cfg(not(feature = "stateful-ast"))]
    pub fn new() -> Self {
        Self {
            scanners: vec![
                Box::new(secrets::SecretScanner::new()),
                Box::new(shell_policy::ShellPolicyScanner::new()),
                Box::new(magika::MagikaScanner::new()),
            ],
        }
    }

    #[cfg(feature = "stateful-ast")]
    pub fn new() -> Self {
        let tracker = std::sync::Arc::new(state::StateTracker::new());
        Self {
            scanners: vec![
                Box::new(secrets::SecretScanner::new()),
                Box::new(shell_policy::ShellPolicyScanner::with_state(tracker)),
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

impl IncidentTemplate {
    pub fn effect(&self) -> FindingEffect {
        match self.state {
            IncidentState::PendingDecision => FindingEffect::Block,
            _ => FindingEffect::Watch,
        }
    }
}

pub fn extract_artifact_refs(req: &CapabilityRequest) -> Vec<ArtifactRef> {
    if !matches!(
        req.capability.as_str(),
        "terminal.read_file" | "terminal.exec" | "terminal.send_text"
    ) {
        return Vec::new();
    }

    let operators = ["|", "&&", "||", ";", ">", ">>", "<", "2>", "2>>"];
    let tokens = req.payload.split_whitespace().collect::<Vec<_>>();
    let mut refs = Vec::new();

    for token in tokens {
        let trimmed = token.trim_matches(|c| c == '\'' || c == '"' || c == '`');
        if trimmed.is_empty()
            || trimmed.starts_with('-')
            || operators.contains(&trimmed)
            || trimmed.contains("://")
        {
            continue;
        }

        if !looks_like_path(trimmed) {
            continue;
        }

        let resolved = resolve_path(&req.cwd, trimmed);
        if refs.iter().any(|existing: &ArtifactRef| existing.resolved_path == resolved) {
            continue;
        }

        refs.push(ArtifactRef {
            original: trimmed.to_string(),
            resolved_path: resolved,
            source_capability: req.capability.clone(),
        });
    }

    refs
}

fn looks_like_path(token: &str) -> bool {
    token.starts_with("./")
        || token.starts_with("../")
        || token.starts_with('/')
        || token.starts_with('~')
        || token.contains('/')
        || token.contains('.')
}

fn resolve_path(cwd: &str, token: &str) -> String {
    if token.starts_with('~') || token.starts_with('/') {
        return token.to_string();
    }

    normalize_path(&Path::new(cwd).join(token)).display().to_string()
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}

fn candidate_precedes(candidate: &IncidentTemplate, current: &IncidentTemplate) -> bool {
    (
        candidate.effect(),
        stage_rank(candidate.stage),
        candidate.risk,
    ) > (
        current.effect(),
        stage_rank(current.stage),
        current.risk,
    )
}

fn stage_rank(stage: ScannerStage) -> u8 {
    match stage {
        ScannerStage::PreForwardFast => 0,
        ScannerStage::ArtifactInspect => 1,
        ScannerStage::PreForwardBlocking => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn returns_no_artifacts_for_ls() {
        let refs = extract_artifact_refs(&request("terminal.exec", "ls -la", "/tmp/workspace"));
        assert!(refs.is_empty());
    }
}