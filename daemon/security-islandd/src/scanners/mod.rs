use shared::{control::AllowedAction, CapabilityRequest, IncidentState, Severity};

pub mod secrets;
pub mod shell_policy;
pub mod magika;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScannerStage {
    PreForwardBlocking,
    PreForwardFast,
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
    pub allowed_actions: Vec<AllowedAction>,
}

pub trait Scanner: Send + Sync {
    fn scan(&self, req: &CapabilityRequest) -> Vec<IncidentTemplate>;
}

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

impl IncidentTemplate {
    pub fn effect(&self) -> FindingEffect {
        match self.state {
            IncidentState::PendingDecision => FindingEffect::Block,
            _ => FindingEffect::Watch,
        }
    }
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
        ScannerStage::PreForwardBlocking => 1,
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
            allowed_actions: vec![],
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
        let blocking = template(FindingEffect::Watch, 50, ScannerStage::PreForwardBlocking);
        assert!(candidate_precedes(&blocking, &fast));
    }

    #[test]
    fn higher_risk_breaks_ties() {
        let stronger = template(FindingEffect::Watch, 70, ScannerStage::PreForwardFast);
        let weaker = template(FindingEffect::Watch, 60, ScannerStage::PreForwardFast);
        assert!(candidate_precedes(&stronger, &weaker));
    }
}
