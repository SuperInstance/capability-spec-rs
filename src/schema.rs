//! Schema types for capability specifications.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A single capability with confidence, recency, and dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub last_used: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String { "1.0.0".into() }

impl Capability {
    pub fn new(name: &str, confidence: f64) -> Self {
        assert!((0.0..=1.0).contains(&confidence), "confidence must be 0-1");
        Self { name: name.into(), confidence, ..Default::default() }
    }
}

impl Default for Capability {
    fn default() -> Self {
        Self { name: String::new(), confidence: 0.0, last_used: String::new(), description: String::new(), requires: Vec::new(), version: "1.0.0".into() }
    }
}

/// Agent metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "type")]
    pub agent_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub avatar: String,
    #[serde(default)]
    pub home_repo: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub last_active: String,
    #[serde(default)]
    pub runtime: HashMap<String, serde_json::Value>,
}

/// Communication config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunicationConfig {
    #[serde(default)]
    pub bottles: bool,
    #[serde(default)]
    pub bottle_path: String,
    #[serde(default)]
    pub mud: bool,
    #[serde(default)]
    pub mud_home: String,
    #[serde(default)]
    pub issues: bool,
    #[serde(default)]
    pub pr_reviews: bool,
}

/// Resource config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceConfig {
    #[serde(default)]
    pub compute: String,
    pub cpu_cores: Option<f64>,
    pub ram_gb: Option<f64>,
    pub storage_gb: Option<f64>,
    #[serde(default)]
    pub cuda: bool,
    #[serde(default)]
    pub languages: Vec<String>,
}

/// Constraint config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstraintConfig {
    #[serde(default)]
    pub max_task_duration: String,
    #[serde(default)]
    pub requires_approval: Vec<String>,
    #[serde(default)]
    pub refuses: Vec<String>,
    pub budget_tokens_per_day: Option<f64>,
}

/// Associate config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssociateConfig {
    #[serde(default)]
    pub reports_to: String,
    #[serde(default)]
    pub collaborates: Vec<String>,
    #[serde(default)]
    pub manages: Vec<String>,
    #[serde(default)]
    pub trusts: HashMap<String, f64>,
}

/// Top-level capability schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySchema {
    #[serde(default)]
    pub agent: AgentInfo,
    #[serde(default)]
    pub capabilities: HashMap<String, Capability>,
    #[serde(default)]
    pub communication: CommunicationConfig,
    #[serde(default)]
    pub resources: ResourceConfig,
    #[serde(default)]
    pub constraints: ConstraintConfig,
    #[serde(default)]
    pub associates: AssociateConfig,
    #[serde(default = "default_schema_version")]
    pub version: String,
}

impl Default for CapabilitySchema {
    fn default() -> Self {
        Self {
            agent: AgentInfo::default(),
            capabilities: HashMap::new(),
            communication: CommunicationConfig::default(),
            resources: ResourceConfig::default(),
            constraints: ConstraintConfig::default(),
            associates: AssociateConfig::default(),
            version: default_schema_version(),
        }
    }
}

fn default_schema_version() -> String { "1.0.0".into() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_new() {
        let cap = Capability::new("test", 0.8);
        assert_eq!(cap.name, "test");
        assert!((cap.confidence - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_schema_default() {
        let schema = CapabilitySchema::default();
        assert!(schema.capabilities.is_empty());
        assert_eq!(schema.version, "1.0.0");
    }

    #[test]
    fn test_schema_serde_roundtrip() {
        let schema = CapabilitySchema::default();
        let json = serde_json::to_string(&schema).unwrap();
        let back: CapabilitySchema = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, schema.version);
    }
}
