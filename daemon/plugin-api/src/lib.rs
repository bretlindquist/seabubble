use std::path::{Component, Path, PathBuf};

pub use shared::{control::AllowedAction, CapabilityRequest, IncidentState, Severity};

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

    let tokens = req.payload.split_whitespace().collect::<Vec<_>>();
    let command = tokens.first().and_then(|token| clean_token(token));
    let mut refs = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        let Some(candidate) = clean_token(token) else {
            continue;
        };
        if index == 0 {
            continue;
        }
        if !looks_like_artifact_arg(candidate, command) {
            continue;
        }

        let resolved = resolve_path(&req.cwd, candidate);
        if refs.iter().any(|existing: &ArtifactRef| existing.resolved_path == resolved) {
            continue;
        }

        refs.push(ArtifactRef {
            original: candidate.to_string(),
            resolved_path: resolved,
            source_capability: req.capability.clone(),
        });
    }

    refs
}

fn clean_token(token: &str) -> Option<&str> {
    let trimmed = token.trim_matches(|c| c == '\'' || c == '"' || c == '`');
    let trimmed = trimmed.trim_end_matches([';', ',', ')', '(']);
    let trimmed = trimmed
        .split(['>', '<', '|', '&'])
        .next()
        .unwrap_or(trimmed)
        .trim();

    if trimmed.is_empty()
        || trimmed.starts_with('-')
        || matches!(trimmed, "|" | "&&" | "||" | ";")
        || trimmed.contains("://")
    {
        return None;
    }

    Some(trimmed)
}

fn looks_like_artifact_arg(token: &str, command: Option<&str>) -> bool {
    if token.starts_with("./")
        || token.starts_with("../")
        || token.starts_with('/')
        || token.starts_with('~')
        || token.contains('/')
        || token.contains('.')
    {
        return true;
    }

    if token.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')) {
        return matches!(command, Some("python" | "python3" | "bash" | "sh" | "zsh" | "source"));
    }

    false
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

pub fn candidate_precedes(candidate: &IncidentTemplate, current: &IncidentTemplate) -> bool {
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
