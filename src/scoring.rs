//! Capability scoring with recency-weighted confidence.
//!
//! This module implements a weighted scoring system for ranking capabilities.
//! The core insight: a capability's effective score depends not just on its
//! raw confidence, but also on **how recently** it was exercised. Fresh skills
//! score higher than stale ones.
//!
//! # Scoring Formula
//!
//! For a single capability:
//!
//! ```text
//! score = confidence × recency_weight(days_since_last_use)
//! ```
//!
//! The recency weight follows a step function:
//!
//! | Days since last use | Weight |
//! |---------------------|--------|
//! | 0 (today)           | 1.0    |
//! | 1–2                 | 0.9    |
//! | 3–6                 | 0.7    |
//! | 7–29                | 0.5    |
//! | 30+ or unknown      | 0.3    |
//!
//! The aggregate schema score is the **mean** of all individual capability scores.
//!
//! # Example
//!
//! ```rust
//! use capability_spec::scoring::{score_capability, score_schema, recency_weight};
//! use capability_spec::schema::Capability;
//!
//! // A freshly used high-confidence capability scores near 1.0
//! let mut cap = Capability::new("code_gen", 0.9);
//! cap.last_used = chrono::Local::now().format("%Y-%m-%d").to_string();
//! assert!((score_capability(&cap) - 0.9).abs() < 1e-10);
//!
//! // Recency weights decay over time
//! assert!((recency_weight(0) - 1.0).abs() < 1e-10);
//! assert!((recency_weight(5) - 0.7).abs() < 1e-10);
//! assert!((recency_weight(100) - 0.3).abs() < 1e-10);
//! ```

use chrono::{Local, NaiveDate};

use crate::schema::{Capability, CapabilitySchema};

// ─────────────────────────────────────────────────────────────────────────────
// Recency weight
// ─────────────────────────────────────────────────────────────────────────────

/// Compute a recency weight from the number of days since last use.
///
/// Uses a step function that penalizes stale capabilities:
///
/// - `< 1 day` → 1.0 (just used, full weight)
/// - `1–2 days` → 0.9
/// - `3–6 days` → 0.7
/// - `7–29 days` → 0.5
/// - `30+ days` → 0.3 (stale, heavily discounted)
///
/// # Example
///
/// ```rust
/// use capability_spec::scoring::recency_weight;
///
/// assert_eq!(recency_weight(0), 1.0);
/// assert_eq!(recency_weight(1), 0.9);
/// assert_eq!(recency_weight(5), 0.7);
/// assert_eq!(recency_weight(15), 0.5);
/// assert_eq!(recency_weight(100), 0.3);
/// ```
pub fn recency_weight(days: i64) -> f64 {
    if days < 1 {
        1.0 // Used today or in the future — full weight.
    } else if days < 3 {
        0.9 // Used yesterday or day before — slight decay.
    } else if days < 7 {
        0.7 // Used within the past week — moderate decay.
    } else if days < 30 {
        0.5 // Used within the past month — significant decay.
    } else {
        0.3 // Stale — heavily discounted.
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Date parsing
// ─────────────────────────────────────────────────────────────────────────────

/// Compute the number of days between today and an ISO 8601 date string.
///
/// Handles:
/// - Full ISO dates: `"2025-12-01"`, `"2025-12-01T10:30:00Z"`
/// - Date-only: `"2025-12-01"`
/// - Empty string: returns 365 (treated as unknown/very stale)
///
/// Returns 0 if the date is today or in the future.
fn days_since(date_str: &str) -> i64 {
    // Empty string means unknown → treat as very stale.
    if date_str.is_empty() {
        return 365;
    }

    // Try to parse just the date portion (first 10 chars of ISO format).
    let date_part = if date_str.len() >= 10 {
        &date_str[..10]
    } else {
        date_str
    };

    match NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
        Ok(date) => {
            let today = Local::now().date_naive();
            let diff = today - date; // chrono::Duration
            diff.num_days().max(0) // Clamp to 0 if date is in the future
        }
        // If we can't parse the date, treat as unknown.
        Err(_) => 365,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Scoring
// ─────────────────────────────────────────────────────────────────────────────

/// Score a single capability as `confidence × recency_weight(days_since_last_used)`.
///
/// # Example
///
/// ```rust
/// use capability_spec::scoring::score_capability;
/// use capability_spec::schema::Capability;
///
/// let cap = Capability::new("test", 0.8);
/// let score = score_capability(&cap);
/// assert!(score > 0.0 && score <= 1.0);
/// ```
pub fn score_capability(cap: &Capability) -> f64 {
    cap.confidence * recency_weight(days_since(&cap.last_used))
}

/// Compute the mean score across all capabilities in a schema.
///
/// Returns 0.0 for schemas with no capabilities.
///
/// # Example
///
/// ```rust
/// use capability_spec::scoring::score_schema;
/// use capability_spec::schema::CapabilitySchema;
///
/// let schema = CapabilitySchema::default();
/// assert_eq!(score_schema(&schema), 0.0);
/// ```
pub fn score_schema(schema: &CapabilitySchema) -> f64 {
    if schema.capabilities.is_empty() {
        return 0.0;
    }
    // Sum all individual capability scores.
    let total: f64 = schema.capabilities.values().map(score_capability).sum();
    // Normalize by count to get the mean.
    total / schema.capabilities.len() as f64
}

/// Find the names of capabilities shared between two schemas.
///
/// Returns a sorted vector of capability names present in both `a` and `b`.
///
/// # Example
///
/// ```rust
/// use capability_spec::scoring::match_capabilities;
/// use capability_spec::schema::{CapabilitySchema, Capability};
///
/// let mut a = CapabilitySchema::default();
/// let mut b = CapabilitySchema::default();
/// a.capabilities.insert("x".into(), Capability::new("x", 0.5));
/// a.capabilities.insert("y".into(), Capability::new("y", 0.5));
/// b.capabilities.insert("x".into(), Capability::new("x", 0.8));
/// b.capabilities.insert("z".into(), Capability::new("z", 0.8));
///
/// let matched = match_capabilities(&a, &b);
/// assert_eq!(matched, vec!["x"]);
/// ```
pub fn match_capabilities(a: &CapabilitySchema, b: &CapabilitySchema) -> Vec<String> {
    let mut shared: Vec<String> = a
        .capabilities
        .keys()
        .filter(|k| b.capabilities.contains_key(*k))
        .cloned()
        .collect();
    shared.sort();
    shared
}

/// Compute a compatibility score between two schemas based on overlapping capabilities.
///
/// For each shared capability, computes `min(score_a, score_b)` — the joint confidence
/// is limited by the weaker agent. Returns the mean of these min-scores, or 0.0 if
/// there are no shared capabilities.
///
/// # Example
///
/// ```rust
/// use capability_spec::scoring::compatibility_score;
/// use capability_spec::schema::{CapabilitySchema, Capability};
///
/// let mut a = CapabilitySchema::default();
/// let mut b = CapabilitySchema::default();
/// a.capabilities.insert("x".into(), Capability::new("x", 0.9));
/// b.capabilities.insert("x".into(), Capability::new("x", 0.6));
///
/// let compat = compatibility_score(&a, &b);
/// // min(0.9, 0.6) * recency_weight = 0.6 (both have default last_used)
/// assert!(compat > 0.0);
/// ```
pub fn compatibility_score(a: &CapabilitySchema, b: &CapabilitySchema) -> f64 {
    let shared = match_capabilities(a, b);
    if shared.is_empty() {
        return 0.0;
    }

    let total: f64 = shared
        .iter()
        .map(|name| {
            let cap_a = &a.capabilities[name];
            let cap_b = &b.capabilities[name];
            // Joint score is limited by the weaker agent.
            let min_confidence = cap_a.confidence.min(cap_b.confidence);
            // Use the more recent of the two.
            let days_a = days_since(&cap_a.last_used);
            let days_b = days_since(&cap_b.last_used);
            let best_days = days_a.min(days_b);
            min_confidence * recency_weight(best_days)
        })
        .sum();

    total / shared.len() as f64
}

/// Rank capabilities by their individual scores, returning names in descending order.
///
/// # Example
///
/// ```rust
/// use capability_spec::scoring::rank_capabilities;
/// use capability_spec::schema::{CapabilitySchema, Capability};
///
/// let mut schema = CapabilitySchema::default();
/// schema.capabilities.insert("weak".into(), Capability::new("weak", 0.3));
/// schema.capabilities.insert("strong".into(), Capability::new("strong", 0.9));
///
/// let ranked = rank_capabilities(&schema);
/// assert_eq!(ranked[0], "strong");
/// assert_eq!(ranked[1], "weak");
/// ```
pub fn rank_capabilities(schema: &CapabilitySchema) -> Vec<String> {
    let mut scored: Vec<(String, f64)> = schema
        .capabilities
        .iter()
        .map(|(name, cap)| (name.clone(), score_capability(cap)))
        .collect();

    // Sort by score descending (highest first).
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    scored.into_iter().map(|(name, _)| name).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recency_weight() {
        assert!((recency_weight(-1) - 1.0).abs() < 1e-10); // future date → full weight
        assert!((recency_weight(0) - 1.0).abs() < 1e-10);
        assert!((recency_weight(1) - 0.9).abs() < 1e-10);
        assert!((recency_weight(2) - 0.9).abs() < 1e-10);
        assert!((recency_weight(3) - 0.7).abs() < 1e-10);
        assert!((recency_weight(6) - 0.7).abs() < 1e-10);
        assert!((recency_weight(7) - 0.5).abs() < 1e-10);
        assert!((recency_weight(29) - 0.5).abs() < 1e-10);
        assert!((recency_weight(30) - 0.3).abs() < 1e-10);
        assert!((recency_weight(100) - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_score_capability() {
        let cap = Capability::new("test", 0.8);
        // Default last_used is empty → 365 days → weight 0.3
        let score = score_capability(&cap);
        assert!((score - 0.24).abs() < 1e-10); // 0.8 * 0.3
    }

    #[test]
    fn test_score_capability_recent() {
        let mut cap = Capability::new("test", 0.8);
        cap.last_used = chrono::Local::now().format("%Y-%m-%d").to_string();
        let score = score_capability(&cap);
        assert!((score - 0.8).abs() < 1e-10); // 0.8 * 1.0
    }

    #[test]
    fn test_days_since_empty() {
        assert_eq!(days_since(""), 365);
    }

    #[test]
    fn test_days_since_invalid() {
        assert_eq!(days_since("not-a-date"), 365);
    }

    #[test]
    fn test_days_since_today() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(days_since(&today), 0);
    }

    #[test]
    fn test_days_since_with_timestamp() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let with_time = format!("{today}T10:30:00Z");
        assert_eq!(days_since(&with_time), 0);
    }

    #[test]
    fn test_score_schema_empty() {
        let schema = CapabilitySchema::default();
        assert!((score_schema(&schema) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_match_capabilities() {
        let mut a = CapabilitySchema::default();
        let mut b = CapabilitySchema::default();
        a.capabilities.insert("x".into(), Capability::new("x", 0.5));
        a.capabilities.insert("y".into(), Capability::new("y", 0.5));
        b.capabilities.insert("x".into(), Capability::new("x", 0.8));
        b.capabilities.insert("z".into(), Capability::new("z", 0.8));
        let matched = match_capabilities(&a, &b);
        assert_eq!(matched, vec!["x"]);
    }

    #[test]
    fn test_match_capabilities_empty() {
        let a = CapabilitySchema::default();
        let b = CapabilitySchema::default();
        assert!(match_capabilities(&a, &b).is_empty());
    }

    #[test]
    fn test_compatibility_score() {
        let mut a = CapabilitySchema::default();
        let mut b = CapabilitySchema::default();
        a.capabilities.insert("x".into(), Capability::new("x", 0.9));
        b.capabilities.insert("x".into(), Capability::new("x", 0.6));
        let compat = compatibility_score(&a, &b);
        // min(0.9, 0.6) * 0.3 (both empty last_used) = 0.18
        assert!((compat - 0.18).abs() < 1e-10);
    }

    #[test]
    fn test_compatibility_no_overlap() {
        let mut a = CapabilitySchema::default();
        let mut b = CapabilitySchema::default();
        a.capabilities.insert("x".into(), Capability::new("x", 0.9));
        b.capabilities.insert("y".into(), Capability::new("y", 0.9));
        assert!((compatibility_score(&a, &b) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_rank_capabilities() {
        let mut schema = CapabilitySchema::default();
        schema.capabilities.insert("weak".into(), Capability::new("weak", 0.3));
        schema.capabilities.insert("strong".into(), Capability::new("strong", 0.9));
        schema.capabilities.insert("medium".into(), Capability::new("medium", 0.6));
        let ranked = rank_capabilities(&schema);
        assert_eq!(ranked[0], "strong");
        assert_eq!(ranked[1], "medium");
        assert_eq!(ranked[2], "weak");
    }
}
