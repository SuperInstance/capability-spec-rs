# capability-spec-rs

Rust port of [capability-spec](https://github.com/SuperInstance/capability-spec) — capability specification parser with semver, dependency graphs, and scoring.

## Features

- **`CapabilitySchema`** — Full CAPABILITY.toml schema with serde support
- **`parser`** — TOML parsing and validation
- **`SemVer`** — Semantic versioning with comparison and compatibility
- **`DependencyGraph`** — Directed graph with Kahn's algorithm topological sort
- **`scoring`** — Capability scoring with recency weights

## Usage

### Parse a capability spec

```rust
use capability_spec::parser::parse_capability_toml;

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
```

### Validate a schema

```rust
use capability_spec::parser::{parse_capability_toml, validate};

let schema = parse_capability_toml(r#"
version = "1.0.0"

[agent]
name = "test"
type = "vessel"

[capabilities.bad]
confidence = 2.0
"#).unwrap();

assert!(validate(&schema).is_err());
```

### Semantic versioning

```rust
use capability_spec::semver::SemVer;

let v = SemVer::parse("1.2.3").unwrap();
assert!(v > SemVer::new(1, 2, 0));
assert!(SemVer::new(1, 5, 0).is_compatible(&SemVer::new(1, 2, 0)));
assert!(!SemVer::new(2, 0, 0).is_compatible(&SemVer::new(1, 2, 0)));
```

### Dependency graph

```rust
use capability_spec::graph::DependencyGraph;

let mut g = DependencyGraph::new();
g.add_edge("a", "b");
g.add_edge("b", "c");
assert_eq!(g.topological_sort().unwrap(), vec!["c", "b", "a"]);
assert!(g.would_create_cycle("c", "a"));
```

### Capability scoring

```rust
use capability_spec::parser::parse_capability_toml;
use capability_spec::scoring::score_schema;

let schema = parse_capability_toml(r#"
version = "1.0.0"

[agent]
name = "scorer"

[capabilities.old]
confidence = 1.0
last_used = ""

[capabilities.new]
confidence = 1.0
last_used = "2024-01-01"
"#).unwrap();

let score = score_schema(&schema);
assert!(score > 0.0 && score < 1.0);
```

## License

MIT

Part of the [SuperInstance OpenConstruct](https://github.com/SuperInstance/OpenConstruct) ecosystem.
