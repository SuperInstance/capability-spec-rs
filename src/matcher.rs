//! Capability matching and compatibility scoring between agents.
//!
//! When a fleet orchestrator needs to find the right agent for a task, it must
//! compare capability profiles. This module provides [`CapabilityMatcher`] — a
//! struct that takes two agents' [`CapabilitySchema`]
//! instances and computes overlap, compatibility scores, and gap analysis.
//!
//! # Matching Strategy
//!
//! Two agents are "compatible" on a given capability if they both declare it.
//! The **compatibility score** uses the minimum confidence of the two (a chain is
//! only as strong as its weakest link), weighted by recency.
//!
//! # Example
//!
//! ```rust
//! use capability_spec::matcher::CapabilityMatcher;
//! use capability_spec::builder::CapabilitySchemaBuilder;
//!
//! let agent_a = CapabilitySchemaBuilder::new("vessel-7")
//!     .agent_type("vessel")
//!     .capability("code_gen", 0.9, None, None)
//!     .capability("review", 0.8, None, None)
//!     .build();
//!
//! let agent_b = CapabilitySchemaBuilder::new("scout-3")
//!     .agent_type("scout")
//!     .capability("code_gen", 0.7, None, None)
//!     .capability("search", 0.95, None, None)
//!     .build();
//!
//! let matcher = CapabilityMatcher::new(&agent_a, &agent_b);
//!
//! // Both can generate code
//! assert_eq!(matcher.shared_capabilities(), vec!["code_gen"]);
//!
//! // What agent_a has that agent_b doesn't
//! assert!(matcher.gaps_for_b().contains(&"review".to_string()));
//!
//! // Overall compatibility (0.0–1.0)
//! assert!(matcher.compatibility_score() > 0.0);
//! ```

use std::collections::HashSet;

use crate::scoring::{compatibility_score, match_capabilities};
use crate::schema::CapabilitySchema;

// ─────────────────────────────────────────────────────────────────────────────
// CapabilityMatcher
// ─────────────────────────────────────────────────────────────────────────────

/// Compares two agents' capability profiles for compatibility and overlap.
///
/// Borrowing references to both schemas, this struct provides lazy-computed
/// analysis of shared capabilities, gaps, and compatibility scores.
///
/// # Example
///
/// ```rust
/// use capability_spec::matcher::CapabilityMatcher;
/// use capability_spec::builder::CapabilitySchemaBuilder;
///
/// let a = CapabilitySchemaBuilder::new("a")
///     .capability("x", 0.9, None, None)
///     .capability("y", 0.8, None, None)
///     .build();
///
/// let b = CapabilitySchemaBuilder::new("b")
///     .capability("x", 0.7, None, None)
///     .capability("z", 0.9, None, None)
///     .build();
///
/// let matcher = CapabilityMatcher::new(&a, &b);
/// assert_eq!(matcher.shared_capabilities().len(), 1);
/// ```
pub struct CapabilityMatcher<'a> {
    /// The first agent's capability schema.
    schema_a: &'a CapabilitySchema,
    /// The second agent's capability schema.
    schema_b: &'a CapabilitySchema,
}

impl<'a> CapabilityMatcher<'a> {
    /// Create a new matcher comparing two agents.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::matcher::CapabilityMatcher;
    /// use capability_spec::schema::CapabilitySchema;
    ///
    /// let a = CapabilitySchema::default();
    /// let b = CapabilitySchema::default();
    /// let matcher = CapabilityMatcher::new(&a, &b);
    /// ```
    pub fn new(schema_a: &'a CapabilitySchema, schema_b: &'a CapabilitySchema) -> Self {
        Self { schema_a, schema_b }
    }

    /// Return the sorted list of capability names present in **both** agents.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::matcher::CapabilityMatcher;
    /// use capability_spec::builder::CapabilitySchemaBuilder;
    ///
    /// let a = CapabilitySchemaBuilder::new("a")
    ///     .capability("x", 0.9, None, None)
    ///     .build();
    /// let b = CapabilitySchemaBuilder::new("b")
    ///     .capability("x", 0.7, None, None)
    ///     .build();
    ///
    /// let matcher = CapabilityMatcher::new(&a, &b);
    /// assert_eq!(matcher.shared_capabilities(), vec!["x"]);
    /// ```
    pub fn shared_capabilities(&self) -> Vec<String> {
        match_capabilities(self.schema_a, self.schema_b)
    }

    /// Return capabilities that agent A has but agent B does not.
    ///
    /// These are "gaps" from B's perspective — capabilities B would need to
    /// acquire or delegate to match A's profile.
    pub fn gaps_for_b(&self) -> Vec<String> {
        let b_keys: HashSet<&str> = self.schema_b.capabilities.keys().map(|s| s.as_str()).collect();
        let mut gaps: Vec<String> = self
            .schema_a
            .capabilities
            .keys()
            .filter(|k| !b_keys.contains(k.as_str()))
            .cloned()
            .collect();
        gaps.sort();
        gaps
    }

    /// Return capabilities that agent B has but agent A does not.
    ///
    /// These are "gaps" from A's perspective.
    pub fn gaps_for_a(&self) -> Vec<String> {
        let a_keys: HashSet<&str> = self.schema_a.capabilities.keys().map(|s| s.as_str()).collect();
        let mut gaps: Vec<String> = self
            .schema_b
            .capabilities
            .keys()
            .filter(|k| !a_keys.contains(k.as_str()))
            .cloned()
            .collect();
        gaps.sort();
        gaps
    }

    /// Compute the overall compatibility score (0.0–1.0).
    ///
    /// For each shared capability, takes the minimum confidence of the two agents
    /// (weakest-link principle), weighted by recency. Returns the mean of these
    /// scores, or 0.0 if there are no shared capabilities.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::matcher::CapabilityMatcher;
    /// use capability_spec::builder::CapabilitySchemaBuilder;
    ///
    /// let a = CapabilitySchemaBuilder::new("a")
    ///     .capability("x", 0.9, None, None)
    ///     .build();
    /// let b = CapabilitySchemaBuilder::new("b")
    ///     .capability("x", 0.7, None, None)
    ///     .build();
    ///
    /// let matcher = CapabilityMatcher::new(&a, &b);
    /// let score = matcher.compatibility_score();
    /// assert!(score > 0.0 && score <= 1.0);
    /// ```
    pub fn compatibility_score(&self) -> f64 {
        compatibility_score(self.schema_a, self.schema_b)
    }

    /// Return the fraction of A's capabilities that B also has.
    ///
    /// A value of 1.0 means B covers all of A's capabilities.
    /// Returns 0.0 if A has no capabilities.
    pub fn coverage_of_a(&self) -> f64 {
        if self.schema_a.capabilities.is_empty() {
            return 0.0;
        }
        let shared = self.shared_capabilities().len() as f64;
        shared / self.schema_a.capabilities.len() as f64
    }

    /// Return the fraction of B's capabilities that A also has.
    ///
    /// A value of 1.0 means A covers all of B's capabilities.
    /// Returns 0.0 if B has no capabilities.
    pub fn coverage_of_b(&self) -> f64 {
        if self.schema_b.capabilities.is_empty() {
            return 0.0;
        }
        let shared = self.shared_capabilities().len() as f64;
        shared / self.schema_b.capabilities.len() as f64
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::CapabilitySchemaBuilder;

    fn make_a() -> CapabilitySchema {
        CapabilitySchemaBuilder::new("agent-a")
            .agent_type("vessel")
            .capability("code_gen", 0.9, None, Some("Code generation"))
            .capability("review", 0.8, None, Some("Code review"))
            .build()
    }

    fn make_b() -> CapabilitySchema {
        CapabilitySchemaBuilder::new("agent-b")
            .agent_type("scout")
            .capability("code_gen", 0.7, None, Some("Code gen"))
            .capability("search", 0.95, None, Some("Web search"))
            .build()
    }

    #[test]
    fn test_shared_capabilities() {
        let a = make_a();
        let b = make_b();
        let matcher = CapabilityMatcher::new(&a, &b);
        assert_eq!(matcher.shared_capabilities(), vec!["code_gen"]);
    }

    #[test]
    fn test_gaps_for_b() {
        let a = make_a();
        let b = make_b();
        let matcher = CapabilityMatcher::new(&a, &b);
        assert_eq!(matcher.gaps_for_b(), vec!["review"]);
    }

    #[test]
    fn test_gaps_for_a() {
        let a = make_a();
        let b = make_b();
        let matcher = CapabilityMatcher::new(&a, &b);
        assert_eq!(matcher.gaps_for_a(), vec!["search"]);
    }

    #[test]
    fn test_compatibility_score() {
        let a = make_a();
        let b = make_b();
        let matcher = CapabilityMatcher::new(&a, &b);
        let score = matcher.compatibility_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_coverage() {
        let a = make_a();
        let b = make_b();
        let matcher = CapabilityMatcher::new(&a, &b);
        // A has 2 caps, B shares 1 → 50% coverage
        assert!((matcher.coverage_of_a() - 0.5).abs() < 1e-10);
        // B has 2 caps, A shares 1 → 50% coverage
        assert!((matcher.coverage_of_b() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_no_overlap() {
        let a = CapabilitySchemaBuilder::new("a")
            .capability("x", 0.9, None, None)
            .build();
        let b = CapabilitySchemaBuilder::new("b")
            .capability("y", 0.9, None, None)
            .build();
        let matcher = CapabilityMatcher::new(&a, &b);
        assert!(matcher.shared_capabilities().is_empty());
        assert!((matcher.compatibility_score() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty_schemas() {
        let a = CapabilitySchema::default();
        let b = CapabilitySchema::default();
        let matcher = CapabilityMatcher::new(&a, &b);
        assert!(matcher.shared_capabilities().is_empty());
        assert!((matcher.compatibility_score() - 0.0).abs() < 1e-10);
    }
}
