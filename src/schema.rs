//! Schema types for capability specifications.
//!
//! This module defines the core data types that make up a [`CapabilitySchema`] — the
//! top-level structure parsed from a `CAPABILITY.toml` file. Every type derives
//! [`serde::Serialize`] and [`serde::Deserialize`] for lossless round-tripping through
//! TOML, JSON, or any serde-compatible format.
//!
//! # Types
//!
//! - [`CapabilitySchema`] — Root document: agent info + capabilities + configs
//! - [`Capability`] — A single named skill with confidence, recency, version, and deps
//! - [`AgentInfo`] — Who the agent is: name, type, status, model, runtime
//! - [`CommunicationConfig`] — How the agent communicates (bottles, MUD, issues)
//! - [`ResourceConfig`] — What the agent needs (CPU, RAM, CUDA, languages)
//! - [`ConstraintConfig`] — What the agent won't/can't do (durations, refusals, budgets)
//! - [`AssociateConfig`] — Organizational relationships (reports-to, trusts, manages)
//!
//! # Example
//!
//! ```rust
//! use capability_spec::schema::{CapabilitySchema, Capability};
//!
//! let cap = Capability::new("code_gen", 0.9);
//! assert_eq!(cap.name, "code_gen");
//! assert!((cap.confidence - 0.9).abs() < 1e-10);
//!
//! let schema = CapabilitySchema::default();
//! assert!(schema.capabilities.is_empty());
//! ```

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Capability
// ─────────────────────────────────────────────────────────────────────────────

/// A single capability with confidence, recency, version, and dependency metadata.
///
/// Capabilities represent discrete skills or competencies that an agent possesses.
/// Each one carries a confidence score (0.0–1.0), an optional ISO-date string for
/// when it was last exercised, a semantic version, and a list of other capabilities
/// it depends on.
///
/// # Example
///
/// ```rust
/// use capability_spec::schema::Capability;
///
/// let cap = Capability::new("code_gen", 0.92);
/// assert_eq!(cap.name, "code_gen");
/// assert_eq!(cap.version, "1.0.0"); // default version
///
/// // Serde round-trip
/// let json = serde_json::to_string(&cap).unwrap();
/// let back: Capability = serde_json::from_str(&json).unwrap();
/// assert_eq!(back.name, cap.name);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Human-readable name of this capability (e.g. `"code_gen"`, `"testing"`).
    #[serde(default)]
    pub name: String,

    /// Confidence score in [0.0, 1.0]. Higher means the agent is more certain it
    /// can perform this capability well.
    #[serde(default)]
    pub confidence: f64,

    /// ISO 8601 date string (e.g. `"2025-12-01"`) indicating when this capability
    /// was last exercised. Empty means unknown — treated as stale by the scorer.
    #[serde(default)]
    pub last_used: String,

    /// Human-readable description of what this capability entails.
    #[serde(default)]
    pub description: String,

    /// Names of other capabilities that must be present for this one to function.
    /// Used by the dependency graph for topological ordering.
    #[serde(default)]
    pub requires: Vec<String>,

    /// Semantic version of this capability (default `"1.0.0"`).
    #[serde(default = "default_version")]
    pub version: String,
}

/// Default version string used when omitted from TOML.
fn default_version() -> String {
    "1.0.0".into()
}

impl Capability {
    /// Create a new capability with the given name and confidence.
    ///
    /// # Panics
    ///
    /// Panics if `confidence` is outside `[0.0, 1.0]`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::schema::Capability;
    ///
    /// let cap = Capability::new("testing", 0.75);
    /// assert_eq!(cap.name, "testing");
    /// ```
    pub fn new(name: &str, confidence: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&confidence),
            "confidence must be between 0.0 and 1.0, got {confidence}"
        );
        Self {
            name: name.into(),
            confidence,
            ..Default::default()
        }
    }
}

impl Default for Capability {
    fn default() -> Self {
        Self {
            name: String::new(),
            confidence: 0.0,
            last_used: String::new(),
            description: String::new(),
            requires: Vec::new(),
            version: "1.0.0".into(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AgentInfo
// ─────────────────────────────────────────────────────────────────────────────

/// Agent identity and metadata.
///
/// Describes *who* the agent is within the fleet — its name, type (lighthouse, vessel,
/// scout, etc.), operational status, model, and runtime environment.
///
/// # Example
///
/// ```rust
/// use capability_spec::schema::AgentInfo;
///
/// let info = AgentInfo::default();
/// assert!(info.name.is_empty());
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Display name of the agent (e.g. `"naval-vessel-7"`).
    #[serde(default)]
    pub name: String,

    /// Agent archetype: `lighthouse`, `vessel`, `scout`, `quartermaster`, `barnacle`, or `greenhorn`.
    #[serde(default, rename = "type")]
    pub agent_type: String,

    /// Operational status: `active`, `idle`, `hibernating`, or `decommissioned`.
    #[serde(default)]
    pub status: String,

    /// Free-text role description (e.g. `"Pacific fleet coordinator"`).
    #[serde(default)]
    pub role: String,

    /// Emoji or image URL for UI display.
    #[serde(default)]
    pub avatar: String,

    /// Primary source repository.
    #[serde(default)]
    pub home_repo: String,

    /// Underlying model identifier (e.g. `"gpt-4o"`, `"claude-3-opus"`).
    #[serde(default)]
    pub model: String,

    /// ISO 8601 timestamp of last activity.
    #[serde(default)]
    pub last_active: String,

    /// Arbitrary key-value runtime metadata (OS, arch, versions, etc.).
    #[serde(default)]
    pub runtime: HashMap<String, serde_json::Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// CommunicationConfig
// ─────────────────────────────────────────────────────────────────────────────

/// Communication channels the agent supports.
///
/// Controls which inter-agent messaging protocols are enabled. Bottles are
/// file-based messages; MUD is a shared state space; issues and PR reviews
/// integrate with GitHub-style forges.
///
/// # Example
///
/// ```rust
/// use capability_spec::schema::CommunicationConfig;
///
/// let comm = CommunicationConfig::default();
/// assert!(!comm.bottles); // disabled by default
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunicationConfig {
    /// Enable bottle-based (file) messaging.
    #[serde(default)]
    pub bottles: bool,

    /// Directory path for bottle files.
    #[serde(default)]
    pub bottle_path: String,

    /// Enable MUD (shared state space) integration.
    #[serde(default)]
    pub mud: bool,

    /// MUD home location identifier.
    #[serde(default)]
    pub mud_home: String,

    /// Enable GitHub issue interaction.
    #[serde(default)]
    pub issues: bool,

    /// Enable pull-request review handling.
    #[serde(default)]
    pub pr_reviews: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// ResourceConfig
// ─────────────────────────────────────────────────────────────────────────────

/// Compute and storage resource requirements.
///
/// Declares what the agent needs in terms of hardware — CPU cores, RAM, disk,
/// GPU support, and programming language runtimes.
///
/// # Example
///
/// ```rust
/// use capability_spec::schema::ResourceConfig;
///
/// let res = ResourceConfig::default();
/// assert!(res.cpu_cores.is_none());
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Compute tier label (e.g. `"low"`, `"medium"`, `"high"`).
    #[serde(default)]
    pub compute: String,

    /// Number of CPU cores required.
    pub cpu_cores: Option<f64>,

    /// RAM required in gigabytes.
    pub ram_gb: Option<f64>,

    /// Storage required in gigabytes.
    pub storage_gb: Option<f64>,

    /// Whether CUDA GPU support is required.
    #[serde(default)]
    pub cuda: bool,

    /// Programming language runtimes the agent can use.
    #[serde(default)]
    pub languages: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// ConstraintConfig
// ─────────────────────────────────────────────────────────────────────────────

/// Operational constraints and policy boundaries.
///
/// Defines limits on what the agent is allowed to do — maximum task durations,
/// actions requiring human approval, hard refusals, and daily token budgets.
///
/// # Example
///
/// ```rust
/// use capability_spec::schema::ConstraintConfig;
///
/// let constraints = ConstraintConfig::default();
/// assert!(constraints.requires_approval.is_empty());
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstraintConfig {
    /// Maximum allowed task duration (e.g. `"2h"`, `"30m"`).
    #[serde(default)]
    pub max_task_duration: String,

    /// Action categories that require human approval before execution.
    #[serde(default)]
    pub requires_approval: Vec<String>,

    /// Action categories the agent will never perform (hard refusal).
    #[serde(default)]
    pub refuses: Vec<String>,

    /// Maximum LLM tokens the agent may consume per day.
    pub budget_tokens_per_day: Option<f64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// AssociateConfig
// ─────────────────────────────────────────────────────────────────────────────

/// Organizational relationships between agents.
///
/// Captures the fleet's social graph — who reports to whom, who collaborates,
/// who manages whom, and pairwise trust scores.
///
/// # Example
///
/// ```rust
/// use capability_spec::schema::AssociateConfig;
///
/// let assoc = AssociateConfig::default();
/// assert!(assoc.collaborates.is_empty());
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssociateConfig {
    /// Name of the agent this one reports to (organizational hierarchy).
    #[serde(default)]
    pub reports_to: String,

    /// Agents this one regularly collaborates with.
    #[serde(default)]
    pub collaborates: Vec<String>,

    /// Agents this one manages or supervises.
    #[serde(default)]
    pub manages: Vec<String>,

    /// Pairwise trust scores (0.0–1.0) keyed by agent name.
    #[serde(default)]
    pub trusts: HashMap<String, f64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// CapabilitySchema (root)
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level capability specification document.
///
/// This is the root type that maps 1:1 to a `CAPABILITY.toml` file. It aggregates
/// agent metadata, named capabilities, communication preferences, resource
/// requirements, operational constraints, and organizational relationships.
///
/// # Example
///
/// ```rust
/// use capability_spec::schema::CapabilitySchema;
///
/// let schema = CapabilitySchema::default();
/// assert!(schema.capabilities.is_empty());
/// assert_eq!(schema.version, "1.0.0");
///
/// // Serde round-trip through JSON
/// let json = serde_json::to_string(&schema).unwrap();
/// let back: CapabilitySchema = serde_json::from_str(&json).unwrap();
/// assert_eq!(back.version, schema.version);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySchema {
    /// Agent identity and metadata.
    #[serde(default)]
    pub agent: AgentInfo,

    /// Named capabilities keyed by skill identifier.
    #[serde(default)]
    pub capabilities: HashMap<String, Capability>,

    /// Communication channel configuration.
    #[serde(default)]
    pub communication: CommunicationConfig,

    /// Resource requirements.
    #[serde(default)]
    pub resources: ResourceConfig,

    /// Operational constraints and policies.
    #[serde(default)]
    pub constraints: ConstraintConfig,

    /// Organizational relationships.
    #[serde(default)]
    pub associates: AssociateConfig,

    /// Schema format version (default `"1.0.0"`).
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

/// Default schema format version.
fn default_schema_version() -> String {
    "1.0.0".into()
}

impl fmt::Display for CapabilitySchema {
    /// Pretty-print a capability schema as a human-readable summary.
    ///
    /// Shows agent name/type, capability count, status, and a compact list of
    /// each capability with its confidence and version.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::builder::CapabilitySchemaBuilder;
    ///
    /// let schema = CapabilitySchemaBuilder::new("test-agent")
    ///     .agent_type("vessel")
    ///     .capability("code_gen", 0.9, None, None)
    ///     .build();
    ///
    /// let display = format!("{schema}");
    /// assert!(display.contains("test-agent"));
    /// assert!(display.contains("vessel"));
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CapabilitySchema v{}", self.version)?;
        writeln!(
            f,
            "  Agent: {} ({}) — {}",
            if self.agent.name.is_empty() {
                "<unnamed>"
            } else {
                &self.agent.name
            },
            if self.agent.agent_type.is_empty() {
                "unknown"
            } else {
                &self.agent.agent_type
            },
            if self.agent.status.is_empty() {
                "unknown status"
            } else {
                &self.agent.status
            },
        )?;
        writeln!(f, "  Capabilities ({}):", self.capabilities.len())?;
        for (name, cap) in &self.capabilities {
            writeln!(
                f,
                "    {} confidence={:.2} version={}",
                name, cap.confidence, cap.version
            )?;
        }
        if !self.resources.compute.is_empty() {
            writeln!(f, "  Resources: compute={}", self.resources.compute)?;
        }
        if !self.constraints.refuses.is_empty() {
            writeln!(f, "  Refuses: {}", self.constraints.refuses.join(", "))?;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

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
    fn test_capability_new_panics_on_invalid_confidence() {
        let result = std::panic::catch_unwind(|| Capability::new("bad", 1.5));
        assert!(result.is_err());
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

    #[test]
    fn test_schema_display() {
        let mut schema = CapabilitySchema::default();
        schema.agent.name = "test-agent".into();
        schema.agent.agent_type = "vessel".into();
        schema.agent.status = "active".into();
        schema
            .capabilities
            .insert("code_gen".into(), Capability::new("code_gen", 0.9));
        let display = format!("{schema}");
        assert!(display.contains("test-agent"));
        assert!(display.contains("vessel"));
        assert!(display.contains("code_gen"));
    }
}
