use std::sync::Mutex;
use extism::{Plugin, Manifest, Wasm};
use plugin_api::{Scanner, CapabilityRequest, IncidentTemplate, ScannerStage};
use shared::{IncidentState, Severity, control::AllowedAction};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WasmScannerStage {
    PreForwardBlocking,
    PreForwardFast,
    ArtifactInspect,
}

impl From<WasmScannerStage> for ScannerStage {
    fn from(stage: WasmScannerStage) -> Self {
        match stage {
            WasmScannerStage::PreForwardBlocking => ScannerStage::PreForwardBlocking,
            WasmScannerStage::PreForwardFast => ScannerStage::PreForwardFast,
            WasmScannerStage::ArtifactInspect => ScannerStage::ArtifactInspect,
        }
    }
}

#[derive(Deserialize)]
pub struct WasmIncidentTemplate {
    pub stage: WasmScannerStage,
    pub state: IncidentState,
    pub severity: Severity,
    pub reason: String,
    pub rule_id: String,
    pub risk: u8,
    pub evidence: Vec<String>,
    pub regex: Option<String>,
    pub bash_ast: Option<String>,
    pub magika: Option<String>,
    pub allowed_actions: Vec<AllowedAction>,
}

impl From<WasmIncidentTemplate> for IncidentTemplate {
    fn from(template: WasmIncidentTemplate) -> Self {
        Self {
            stage: template.stage.into(),
            state: template.state,
            severity: template.severity,
            reason: Box::leak(template.reason.into_boxed_str()),
            rule_id: Box::leak(template.rule_id.into_boxed_str()),
            risk: template.risk,
            evidence: template.evidence,
            regex: template.regex,
            bash_ast: template.bash_ast,
            magika: template.magika,
            allowed_actions: template.allowed_actions,
        }
    }
}

pub struct WasmScanner {
    plugin: Mutex<Plugin>,
}

impl WasmScanner {
    pub fn new(wasm_bytes: &[u8]) -> anyhow::Result<Self> {
        let manifest = Manifest::new([Wasm::data(wasm_bytes)]);
        let plugin = Plugin::new(&manifest, [], true)?;
        Ok(Self {
            plugin: Mutex::new(plugin),
        })
    }
}

impl Scanner for WasmScanner {
    fn scan(&self, req: &CapabilityRequest) -> Vec<IncidentTemplate> {
        let mut plugin = match self.plugin.lock() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to acquire lock on WASM plugin: {}", e);
                return vec![];
            }
        };

        let req_json = match serde_json::to_string(req) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize capability request: {}", e);
                return vec![];
            }
        };

        let output = match plugin.call::<&str, &str>("scan_capability", &req_json) {
            Ok(out) => out,
            Err(e) => {
                eprintln!("WASM plugin call failed: {}", e);
                return vec![];
            }
        };

        let wasm_templates: Vec<WasmIncidentTemplate> = match serde_json::from_str(output) {
            Ok(templates) => templates,
            Err(e) => {
                eprintln!("Failed to deserialize WASM output: {}", e);
                return vec![];
            }
        };

        wasm_templates.into_iter().map(|t| t.into()).collect()
    }
}

pub fn load_plugins_from_dir(dir: &std::path::Path) -> Vec<WasmScanner> {
    let mut scanners = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
                if let Ok(bytes) = std::fs::read(&path) {
                    if let Ok(scanner) = WasmScanner::new(&bytes) {
                        scanners.push(scanner);
                    } else {
                        eprintln!("Failed to instantiate WASM plugin from {:?}", path);
                    }
                }
            }
        }
    }
    scanners
}