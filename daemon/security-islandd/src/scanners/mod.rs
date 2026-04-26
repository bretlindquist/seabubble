use shared::{CapabilityRequest, IncidentState, Severity, control::AllowedAction};

pub mod secrets;
pub mod shell_policy;

#[derive(Debug)]
pub enum PolicyDecision {
    Allow {
        reason: &'static str,
    },
    Watch(IncidentTemplate),
    Block(IncidentTemplate),
}

#[derive(Debug)]
pub struct IncidentTemplate {
    pub state: IncidentState,
    pub severity: Severity,
    pub reason: &'static str,
    pub rule_id: &'static str,
    pub risk: u8,
    pub evidence: Vec<String>,
    pub regex: Option<String>,
    pub allowed_actions: Vec<AllowedAction>,
}

pub trait Scanner: Send + Sync {
    fn scan(&self, req: &CapabilityRequest) -> Option<IncidentTemplate>;
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
            ],
        }
    }

    pub fn classify(&self, req: &CapabilityRequest) -> PolicyDecision {
        for scanner in &self.scanners {
            if let Some(incident) = scanner.scan(req) {
                if matches!(incident.state, IncidentState::PendingDecision) {
                    return PolicyDecision::Block(incident);
                } else {
                    return PolicyDecision::Watch(incident);
                }
            }
        }
        PolicyDecision::Allow {
            reason: "request passed all scanners",
        }
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
