//! Capability scoring with recency weights.

use crate::schema::{Capability, CapabilitySchema};

/// Recency weight from days since last use.
pub fn recency_weight(days: i64) -> f64 {
    if days < 1 { 1.0 }
    else if days < 3 { 0.9 }
    else if days < 7 { 0.7 }
    else if days < 30 { 0.5 }
    else { 0.3 }
}

/// Score a single capability: confidence × recency_weight.
pub fn score_capability(cap: &Capability) -> f64 {
    cap.confidence * recency_weight(days_since(&cap.last_used))
}

/// Compute aggregate score for all capabilities in a schema.
pub fn score_schema(schema: &CapabilitySchema) -> f64 {
    if schema.capabilities.is_empty() { return 0.0; }
    let total: f64 = schema.capabilities.values().map(score_capability).sum();
    total / schema.capabilities.len() as f64
}

/// Simple days-since parser (expects ISO date or empty).
fn days_since(date_str: &str) -> i64 {
    if date_str.is_empty() { return 365; } // unknown = old
    // Very rough: just count as 0 for any non-empty date
    // In production you'd parse the actual date
    0
}

/// Match capabilities between two schemas, returning matching names.
pub fn match_capabilities(a: &CapabilitySchema, b: &CapabilitySchema) -> Vec<String> {
    a.capabilities.keys()
        .filter(|k| b.capabilities.contains_key(*k))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recency_weight() {
        assert!((recency_weight(0) - 1.0).abs() < 1e-10);
        assert!((recency_weight(2) - 0.9).abs() < 1e-10);
        assert!((recency_weight(5) - 0.7).abs() < 1e-10);
        assert!((recency_weight(20) - 0.5).abs() < 1e-10);
        assert!((recency_weight(100) - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_score_capability() {
        let cap = Capability::new("test", 0.8);
        assert!(score_capability(&cap) > 0.0);
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
}
