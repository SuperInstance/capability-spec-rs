//! TOML parser and validator for capability specifications.
//!
//! This module handles converting raw `CAPABILITY.toml` text (or files) into
//! validated [`CapabilitySchema`] structs.
//! Parsing is done by the `toml` crate; validation enforces domain-specific
//! invariants like confidence ranges and allowed agent types.
//!
//! # Errors
//!
//! All errors are surfaced through [`ParseError`], which wraps:
//! - TOML syntax errors
//! - I/O errors (when reading from disk)
//! - Validation errors (invalid agent type, confidence out of range, etc.)
//!
//! # Example
//!
//! ```rust
//! use capability_spec::parser::{parse_capability_toml, validate};
//!
//! let schema = parse_capability_toml(r#"
//! version = "1.0.0"
//!
//! [agent]
//! name = "scout-3"
//! type = "scout"
//! status = "active"
//!
//! [capabilities.search]
//! confidence = 0.85
//! description = "Web search and retrieval"
//! "#).unwrap();
//!
//! assert!(validate(&schema).is_ok());
//! ```

use thiserror::Error;

use crate::schema::CapabilitySchema;

// ─────────────────────────────────────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during parsing or validation.
///
/// Each variant preserves the original error source for chaining, so callers
/// can inspect root causes when needed.
#[derive(Debug, Error)]
pub enum ParseError {
    /// The TOML text could not be deserialized into a [`CapabilitySchema`].
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    /// A file could not be read from disk.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The schema was syntactically valid but failed domain-specific checks.
    #[error("Validation error: {0}")]
    Validation(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsing
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a `CAPABILITY.toml` string into a [`CapabilitySchema`].
///
/// This is a thin wrapper around `toml::from_str` that produces our
/// domain-specific error type. The result is **not** validated — call
/// [`validate`] separately if you want to enforce invariants.
///
/// # Example
///
/// ```rust
/// use capability_spec::parser::parse_capability_toml;
///
/// let schema = parse_capability_toml(r#"
/// version = "1.0.0"
/// [agent]
/// name = "test"
/// "#).unwrap();
///
/// assert_eq!(schema.agent.name, "test");
/// ```
pub fn parse_capability_toml(text: &str) -> Result<CapabilitySchema, ParseError> {
    let schema: CapabilitySchema = toml::from_str(text)?;
    Ok(schema)
}

/// Parse a `CAPABILITY.toml` file from disk.
///
/// Reads the file contents and delegates to [`parse_capability_toml`].
///
/// # Example
///
/// ```rust,no_run
/// use capability_spec::parser::parse_capability_file;
/// use std::path::Path;
///
/// let schema = parse_capability_file(Path::new("CAPABILITY.toml")).unwrap();
/// ```
pub fn parse_capability_file(path: &std::path::Path) -> Result<CapabilitySchema, ParseError> {
    let text = std::fs::read_to_string(path)?;
    parse_capability_toml(&text)
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation
// ─────────────────────────────────────────────────────────────────────────────

/// Validate a parsed [`CapabilitySchema`] against domain invariants.
///
/// Checks performed:
///
/// 1. **Agent type** must be one of: `lighthouse`, `vessel`, `scout`,
///    `quartermaster`, `barnacle`, `greenhorn`, or empty (untyped).
/// 2. **Agent status** must be one of: `active`, `idle`, `hibernating`,
///    `decommissioned`, or empty.
/// 3. **Every capability's confidence** must be in `[0.0, 1.0]`.
///
/// # Example
///
/// ```rust
/// use capability_spec::parser::{parse_capability_toml, validate};
///
/// let schema = parse_capability_toml(r#"
/// [agent]
/// name = "test"
/// type = "vessel"
/// status = "active"
/// "#).unwrap();
///
/// assert!(validate(&schema).is_ok());
/// ```
pub fn validate(schema: &CapabilitySchema) -> Result<(), ParseError> {
    // Step 1: Validate agent type against the allowed set.
    let valid_types = [
        "lighthouse",
        "vessel",
        "scout",
        "quartermaster",
        "barnacle",
        "greenhorn",
        "",
    ];
    if !valid_types.contains(&schema.agent.agent_type.as_str()) {
        return Err(ParseError::Validation(format!(
            "Invalid agent type: '{}'. Must be one of: {}",
            schema.agent.agent_type,
            valid_types
                .iter()
                .filter(|t| !t.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    // Step 2: Validate agent status.
    let valid_statuses = ["active", "idle", "hibernating", "decommissioned", ""];
    if !valid_statuses.contains(&schema.agent.status.as_str()) {
        return Err(ParseError::Validation(format!(
            "Invalid status: '{}'. Must be one of: {}",
            schema.agent.status,
            valid_statuses
                .iter()
                .filter(|t| !t.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    // Step 3: Validate every capability's confidence is in [0, 1].
    for (name, cap) in &schema.capabilities {
        if !(0.0..=1.0).contains(&cap.confidence) {
            return Err(ParseError::Validation(format!(
                "Capability '{}' has confidence {}, which is outside [0.0, 1.0]",
                name, cap.confidence
            )));
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal() {
        let toml = r#"
version = "1.0.0"
"#;
        let schema = parse_capability_toml(toml).unwrap();
        assert_eq!(schema.version, "1.0.0");
    }

    #[test]
    fn test_parse_with_agent() {
        let toml = r#"
[agent]
name = "test-agent"
type = "vessel"
status = "active"
"#;
        let schema = parse_capability_toml(toml).unwrap();
        assert_eq!(schema.agent.name, "test-agent");
        assert_eq!(schema.agent.agent_type, "vessel");
    }

    #[test]
    fn test_parse_with_capabilities() {
        let toml = r#"
[capabilities.code_gen]
confidence = 0.9
description = "Code generation"
"#;
        let schema = parse_capability_toml(toml).unwrap();
        assert!(schema.capabilities.contains_key("code_gen"));
        assert!((schema.capabilities["code_gen"].confidence - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_validate_ok() {
        let toml = r#"
[agent]
name = "test"
type = "vessel"
status = "active"
"#;
        let schema = parse_capability_toml(toml).unwrap();
        assert!(validate(&schema).is_ok());
    }

    #[test]
    fn test_validate_bad_agent_type() {
        let toml = r#"
[agent]
name = "test"
type = "pirate"
status = "active"
"#;
        let schema = parse_capability_toml(toml).unwrap();
        let err = validate(&schema).unwrap_err();
        assert!(matches!(err, ParseError::Validation(_)));
        assert!(err.to_string().contains("pirate"));
    }

    #[test]
    fn test_validate_bad_status() {
        let toml = r#"
[agent]
name = "test"
type = "vessel"
status = "sailing"
"#;
        let schema = parse_capability_toml(toml).unwrap();
        let err = validate(&schema).unwrap_err();
        assert!(err.to_string().contains("sailing"));
    }

    #[test]
    fn test_validate_bad_confidence() {
        let toml = r#"
[capabilities.test]
confidence = 2.0
"#;
        let schema = parse_capability_toml(toml).unwrap();
        assert!(validate(&schema).is_err());
    }

    #[test]
    fn test_parse_invalid_toml() {
        let result = parse_capability_toml("not valid toml [[[[");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_all_agent_types() {
        for agent_type in &[
            "lighthouse",
            "vessel",
            "scout",
            "quartermaster",
            "barnacle",
            "greenhorn",
        ] {
            let toml = format!(
                r#"
[agent]
name = "test"
type = "{agent_type}"
status = "active"
"#
            );
            let schema = parse_capability_toml(&toml).unwrap();
            assert!(validate(&schema).is_ok(), "Failed for type: {agent_type}");
        }
    }
}
