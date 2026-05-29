# capability-spec-rs

Capability specification parser with semver validation, dependency graphs (Kahn's algorithm topological sort), and weighted scoring — all from a single `CAPABILITY.toml`.

## What This Gives You

- **`CapabilitySchema`** — Full `CAPABILITY.toml` schema with serde support
- **TOML parsing + validation** — Parse specs, validate confidence ranges, check required fields
- **`SemVer`** — Semantic versioning with comparison, compatibility checks, and ordering
- **`DependencyGraph`** — Directed graph with Kahn's algorithm topological sort for capability ordering
- **Capability scoring** — Weighted scores with recency decay for capability ranking

## Quick Start

### Parse and validate a spec

```rust
use capability_spec::parser::{parse_capability_toml, validate};

let schema = parse_capability_toml(r#"
version = "1.0.0"

[agent]
name = "my-agent"
type = "vessel"
status = "active"

[capabilities.code_gen]
confidence = 0.9
last_used = "2024-01-15"
description = "Generate code from prompts"

[capabilities.review]
confidence = 0.8
requires = ["code_gen"]
"#).unwrap();

assert_eq!(schema.agent.name, "my-agent");
assert_eq!(schema.capabilities.len(), 2);
assert!(validate(&schema).is_ok());
```

### Semantic versioning

```rust
use capability_spec::semver::SemVer;

let v = SemVer::parse("1.2.3").unwrap();
assert!(v > SemVer::new(1, 2, 0));
assert!(SemVer::new(1, 5, 0).is_compatible(&SemVer::new(1, 2, 0)));
assert!(!SemVer::new(2, 0, 0).is_compatible(&SemVer::new(1, 2, 0)));
```

### Dependency graph with topological sort

```rust
use capability_spec::graph::DependencyGraph;

let mut g = DependencyGraph::new();
g.add_edge("code_gen", "review");  // review depends on code_gen
g.add_edge("review", "deploy");    // deploy depends on review
assert_eq!(g.topological_sort().unwrap(), vec!["deploy", "review", "code_gen"]);
```

## API Reference

### `parser` module

| Function | Description |
|----------|-------------|
| `parse_capability_toml(toml)` | Parse TOML string into `CapabilitySchema` |
| `validate(schema)` | Validate schema (confidence ranges, required fields) |

### `SemVer`

| Method | Description |
|--------|-------------|
| `new(major, minor, patch)` | Construct version |
| `parse(s)` | Parse from "X.Y.Z" string |
| `is_compatible(other)` | Same major version? |
| Comparison traits | `Ord`, `PartialOrd`, `Eq`, `PartialEq` |

### `DependencyGraph`

| Method | Description |
|--------|-------------|
| `new()` | Empty graph |
| `add_edge(from, to)` | Add dependency edge |
| `topological_sort()` | Kahn's algorithm — returns sorted or cycle error |

### `scoring` module

Weighted capability scoring with recency weights — recently-used capabilities score higher.

## How It Fits

- **[cocapn-health-rs](https://github.com/SuperInstance/cocapn-health-rs)** — Health checks use capability specs to understand what each agent can do
- **[categorical-agents](https://github.com/SuperInstance/categorical-agents)** — Category theory objects map to capabilities defined in specs
- **[conservation-protocol](https://github.com/SuperInstance/conservation-protocol)** — Agent identity derived from capability structure
- **[co-captain-git-agent](https://github.com/SuperInstance/co-captain-git-agent)** — Fleet dispatch routes tasks to agents matching capability specs

## Testing

22 tests covering TOML parsing, validation (valid/invalid), semver comparison/compatibility, dependency graph topological sort, cycle detection, and scoring.

```bash
cargo test
```

## Installation

```toml
[dependencies]
capability-spec = { git = "https://github.com/SuperInstance/capability-spec-rs" }
```

```bash
git clone https://github.com/SuperInstance/capability-spec-rs.git
cd capability-spec-rs
cargo build
```

## License

MIT

Part of the [SuperInstance OpenConstruct](https://github.com/SuperInstance) ecosystem.
