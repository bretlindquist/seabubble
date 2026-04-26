use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CapabilityRequest {
    pub capability: String,
    pub payload: String,
}

#[derive(Serialize)]
pub enum WasmIncidentSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Serialize)]
pub enum WasmIncidentAction {
    Log,
    Block,
    Alert,
}

#[derive(Serialize)]
pub struct WasmIncidentTemplate {
    pub plugin_name: String,
    pub severity: WasmIncidentSeverity,
    pub action: WasmIncidentAction,
    pub message: String,
}

#[plugin_fn]
pub fn scan_capability(input: String) -> FnResult<String> {
    // Parse the JSON input into our internal struct
    let req: CapabilityRequest = match serde_json::from_str(&input) {
        Ok(r) => r,
        Err(_) => {
            // Return an empty list if we can't parse it
            return Ok("[]".to_string());
        }
    };

    let mut incidents = Vec::new();

    // The business logic
    if req.capability == "terminal.exec" {
        // Simple heuristic for blocked commands
        if req.payload.contains("nmap") || req.payload.contains("nc ") || req.payload == "nc" {
            incidents.push(WasmIncidentTemplate {
                plugin_name: "demo-scanner".to_string(),
                severity: WasmIncidentSeverity::Critical,
                action: WasmIncidentAction::Block,
                message: format!("Blocked potentially dangerous execution: {}", req.payload),
            });
        }
    }

    // Serialize the resulting array of incidents back to JSON
    let out = serde_json::to_string(&incidents)?;
    Ok(out)
}
