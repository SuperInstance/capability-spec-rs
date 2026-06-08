//! Fluent builder for constructing [`CapabilitySchema`] programmatically.
//!
//! Instead of assembling a schema field-by-field or parsing TOML, use
//! [`CapabilitySchemaBuilder`] for a clean, chainable API. This is especially
//! useful in tests and when generating specs dynamically.
//!
//! # Example
//!
//! ```rust
//! use capability_spec::builder::CapabilitySchemaBuilder;
//!
//! let schema = CapabilitySchemaBuilder::new("vessel-7")
//!     .agent_type("vessel")
//!     .status("active")
//!     .role("Code generation and review specialist")
//!     .capability("code_gen", 0.92, Some("2025-12-01"), Some("Generate code from prompts"))
//!     .capability("review", 0.85, Some("2025-12-03"), Some("Review PRs"))
//!     .resource_compute("high")
//!     .resource_cpu(8.0)
//!     .resource_ram(32.0)
//!     .language("rust")
//!     .language("python")
//!     .constraint_max_duration("2h")
//!     .require_approval("production_deploy")
//!     .refuse("destructive_ops")
//!     .build();
//!
//! assert_eq!(schema.agent.name, "vessel-7");
//! assert_eq!(schema.capabilities.len(), 2);
//! assert_eq!(schema.resources.languages.len(), 2);
//! ```

use crate::schema::{Capability, CapabilitySchema};

// ─────────────────────────────────────────────────────────────────────────────
// CapabilitySchemaBuilder
// ─────────────────────────────────────────────────────────────────────────────

/// Fluent builder for [`CapabilitySchema`].
///
/// Start with [`new`](Self::new) providing the agent name, then chain method
/// calls to configure capabilities, resources, constraints, and more. Call
/// [`build`](Self::build) to produce the final schema.
///
/// All methods take `&mut self` and return `&mut Self` for chaining.
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
/// assert_eq!(schema.agent.name, "test-agent");
/// assert_eq!(schema.agent.agent_type, "vessel");
/// ```
pub struct CapabilitySchemaBuilder {
    schema: CapabilitySchema,
}

impl CapabilitySchemaBuilder {
    /// Create a new builder with the given agent name and sensible defaults.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::builder::CapabilitySchemaBuilder;
    ///
    /// let builder = CapabilitySchemaBuilder::new("my-agent");
    /// ```
    pub fn new(agent_name: &str) -> Self {
        let mut schema = CapabilitySchema::default();
        schema.agent.name = agent_name.to_string();
        Self { schema }
    }

    // ── Agent metadata ──────────────────────────────────────────────────

    /// Set the agent type (e.g. `"vessel"`, `"lighthouse"`).
    pub fn agent_type(mut self, agent_type: &str) -> Self {
        self.schema.agent.agent_type = agent_type.to_string();
        self
    }

    /// Set the agent status (e.g. `"active"`, `"idle"`).
    pub fn status(mut self, status: &str) -> Self {
        self.schema.agent.status = status.to_string();
        self
    }

    /// Set the agent's role description.
    pub fn role(mut self, role: &str) -> Self {
        self.schema.agent.role = role.to_string();
        self
    }

    /// Set the agent's avatar (emoji or URL).
    pub fn avatar(mut self, avatar: &str) -> Self {
        self.schema.agent.avatar = avatar.to_string();
        self
    }

    /// Set the agent's underlying model identifier.
    pub fn model(mut self, model: &str) -> Self {
        self.schema.agent.model = model.to_string();
        self
    }

    // ── Capabilities ────────────────────────────────────────────────────

    /// Add a capability to the schema.
    ///
    /// - `name` — Capability identifier (e.g. `"code_gen"`)
    /// - `confidence` — Confidence score in [0.0, 1.0]
    /// - `last_used` — Optional ISO date string (e.g. `Some("2025-12-01")`)
    /// - `description` — Optional human-readable description
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::builder::CapabilitySchemaBuilder;
    ///
    /// let schema = CapabilitySchemaBuilder::new("a")
    ///     .capability("code_gen", 0.9, Some("2025-12-01"), Some("Code generation"))
    ///     .build();
    ///
    /// assert_eq!(schema.capabilities["code_gen"].confidence, 0.9);
    /// ```
    pub fn capability(
        mut self,
        name: &str,
        confidence: f64,
        last_used: Option<&str>,
        description: Option<&str>,
    ) -> Self {
        let mut cap = Capability::new(name, confidence);
        if let Some(lu) = last_used {
            cap.last_used = lu.to_string();
        }
        if let Some(desc) = description {
            cap.description = desc.to_string();
        }
        self.schema.capabilities.insert(name.to_string(), cap);
        self
    }

    /// Add a capability with explicit version and dependencies.
    ///
    /// A more detailed variant of [`capability`](Self::capability) that also
    /// accepts a version string and a list of required capability names.
    pub fn capability_with_deps(
        mut self,
        name: &str,
        confidence: f64,
        last_used: Option<&str>,
        description: Option<&str>,
        version: &str,
        requires: &[&str],
    ) -> Self {
        let mut cap = Capability::new(name, confidence);
        if let Some(lu) = last_used {
            cap.last_used = lu.to_string();
        }
        if let Some(desc) = description {
            cap.description = desc.to_string();
        }
        cap.version = version.to_string();
        cap.requires = requires.iter().map(|s| s.to_string()).collect();
        self.schema.capabilities.insert(name.to_string(), cap);
        self
    }

    // ── Communication ───────────────────────────────────────────────────

    /// Enable bottles (file-based messaging).
    pub fn enable_bottles(mut self, path: &str) -> Self {
        self.schema.communication.bottles = true;
        self.schema.communication.bottle_path = path.to_string();
        self
    }

    /// Enable MUD (shared state space).
    pub fn enable_mud(mut self, home: &str) -> Self {
        self.schema.communication.mud = true;
        self.schema.communication.mud_home = home.to_string();
        self
    }

    /// Enable issue handling.
    pub fn enable_issues(mut self) -> Self {
        self.schema.communication.issues = true;
        self
    }

    /// Enable PR review handling.
    pub fn enable_pr_reviews(mut self) -> Self {
        self.schema.communication.pr_reviews = true;
        self
    }

    // ── Resources ───────────────────────────────────────────────────────

    /// Set the compute tier label.
    pub fn resource_compute(mut self, tier: &str) -> Self {
        self.schema.resources.compute = tier.to_string();
        self
    }

    /// Set CPU core count.
    pub fn resource_cpu(mut self, cores: f64) -> Self {
        self.schema.resources.cpu_cores = Some(cores);
        self
    }

    /// Set RAM in GB.
    pub fn resource_ram(mut self, gb: f64) -> Self {
        self.schema.resources.ram_gb = Some(gb);
        self
    }

    /// Set storage in GB.
    pub fn resource_storage(mut self, gb: f64) -> Self {
        self.schema.resources.storage_gb = Some(gb);
        self
    }

    /// Enable CUDA requirement.
    pub fn resource_cuda(mut self) -> Self {
        self.schema.resources.cuda = true;
        self
    }

    /// Add a programming language to the supported list.
    pub fn language(mut self, lang: &str) -> Self {
        self.schema.resources.languages.push(lang.to_string());
        self
    }

    // ── Constraints ─────────────────────────────────────────────────────

    /// Set the maximum task duration.
    pub fn constraint_max_duration(mut self, duration: &str) -> Self {
        self.schema.constraints.max_task_duration = duration.to_string();
        self
    }

    /// Add an action that requires human approval.
    pub fn require_approval(mut self, action: &str) -> Self {
        self.schema.constraints.requires_approval.push(action.to_string());
        self
    }

    /// Add an action the agent refuses to perform.
    pub fn refuse(mut self, action: &str) -> Self {
        self.schema.constraints.refuses.push(action.to_string());
        self
    }

    /// Set the daily token budget.
    pub fn budget(mut self, tokens: f64) -> Self {
        self.schema.constraints.budget_tokens_per_day = Some(tokens);
        self
    }

    // ── Associates ──────────────────────────────────────────────────────

    /// Set who this agent reports to.
    pub fn reports_to(mut self, agent: &str) -> Self {
        self.schema.associates.reports_to = agent.to_string();
        self
    }

    /// Add a collaborator.
    pub fn collaborates_with(mut self, agent: &str) -> Self {
        self.schema.associates.collaborates.push(agent.to_string());
        self
    }

    /// Add a managed agent.
    pub fn manages(mut self, agent: &str) -> Self {
        self.schema.associates.manages.push(agent.to_string());
        self
    }

    /// Add a trust score for another agent.
    pub fn trusts(mut self, agent: &str, score: f64) -> Self {
        self.schema.associates.trusts.insert(agent.to_string(), score);
        self
    }

    // ── Build ───────────────────────────────────────────────────────────

    /// Consume the builder and produce the final [`CapabilitySchema`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::builder::CapabilitySchemaBuilder;
    ///
    /// let schema = CapabilitySchemaBuilder::new("done")
    ///     .agent_type("vessel")
    ///     .build();
    ///
    /// assert_eq!(schema.agent.name, "done");
    /// ```
    pub fn build(self) -> CapabilitySchema {
        self.schema
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_minimal() {
        let schema = CapabilitySchemaBuilder::new("test").build();
        assert_eq!(schema.agent.name, "test");
        assert!(schema.capabilities.is_empty());
    }

    #[test]
    fn test_builder_full() {
        let schema = CapabilitySchemaBuilder::new("vessel-7")
            .agent_type("vessel")
            .status("active")
            .role("Code specialist")
            .avatar("🚢")
            .model("gpt-4o")
            .capability("code_gen", 0.92, Some("2025-12-01"), Some("Generate code"))
            .capability_with_deps(
                "deploy",
                0.7,
                None,
                Some("Deploy to prod"),
                "1.0.0",
                &["code_gen"],
            )
            .enable_bottles("/tmp/bottles")
            .enable_mud("pacific")
            .enable_issues()
            .enable_pr_reviews()
            .resource_compute("high")
            .resource_cpu(8.0)
            .resource_ram(32.0)
            .resource_storage(100.0)
            .resource_cuda()
            .language("rust")
            .language("python")
            .constraint_max_duration("2h")
            .require_approval("deploy")
            .refuse("destruct")
            .budget(500_000.0)
            .reports_to("lighthouse")
            .collaborates_with("scout-3")
            .manages("barnacle-1")
            .trusts("scout-3", 0.9)
            .build();

        assert_eq!(schema.agent.name, "vessel-7");
        assert_eq!(schema.agent.agent_type, "vessel");
        assert_eq!(schema.agent.status, "active");
        assert_eq!(schema.agent.role, "Code specialist");
        assert_eq!(schema.agent.avatar, "🚢");
        assert_eq!(schema.agent.model, "gpt-4o");
        assert_eq!(schema.capabilities.len(), 2);
        assert_eq!(schema.capabilities["deploy"].requires, vec!["code_gen"]);
        assert!(schema.communication.bottles);
        assert!(schema.communication.mud);
        assert!(schema.communication.issues);
        assert!(schema.communication.pr_reviews);
        assert_eq!(schema.resources.compute, "high");
        assert_eq!(schema.resources.cpu_cores, Some(8.0));
        assert!(schema.resources.cuda);
        assert_eq!(schema.resources.languages, vec!["rust", "python"]);
        assert_eq!(schema.constraints.max_task_duration, "2h");
        assert_eq!(schema.constraints.budget_tokens_per_day, Some(500_000.0));
        assert_eq!(schema.associates.reports_to, "lighthouse");
        assert_eq!(schema.associates.trusts["scout-3"], 0.9);
    }

    #[test]
    fn test_builder_chaining() {
        // Verify the builder returns &mut Self for chaining
        let schema = CapabilitySchemaBuilder::new("chain")
            .agent_type("scout")
            .capability("a", 0.5, None, None)
            .capability("b", 0.6, None, None)
            .build();

        assert_eq!(schema.capabilities.len(), 2);
    }
}
