//! TOML parser for capability specifications.

use thiserror::Error;

use crate::schema::CapabilitySchema;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Parse a CAPABILITY.toml string into a CapabilitySchema.
pub fn parse_capability_toml(text: &str) -> Result<CapabilitySchema, ParseError> {
    let schema: CapabilitySchema = toml::from_str(text)?;
    Ok(schema)
}

/// Parse a CAPABILITY.toml file from disk.
pub fn parse_capability_file(path: &std::path::Path) -> Result<CapabilitySchema, ParseError> {
    let text = std::fs::read_to_string(path)?;
    parse_capability_toml(&text)
}

/// Validate a parsed schema.
pub fn validate(schema: &CapabilitySchema) -> Result<(), ParseError> {
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
            "Invalid agent type: {}",
            schema.agent.agent_type
        )));
    }
    let valid_statuses = ["active", "idle", "hibernating", "decommissioned", ""];
    if !valid_statuses.contains(&schema.agent.status.as_str()) {
        return Err(ParseError::Validation(format!(
            "Invalid status: {}",
            schema.agent.status
        )));
    }
    for (name, cap) in &schema.capabilities {
        if !(0.0..=1.0).contains(&cap.confidence) {
            return Err(ParseError::Validation(format!(
                "Capability '{}' confidence must be 0-1",
                name
            )));
        }
    }
    Ok(())
}

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
    fn test_validate_bad_confidence() {
        let toml = r#"
[capabilities.test]
confidence = 2.0
"#;
        let schema = parse_capability_toml(toml).unwrap();
        assert!(validate(&schema).is_err());
    }
}
