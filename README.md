# capability-spec-rs

Rust port of [capability-spec](https://github.com/SuperInstance/capability-spec) — capability specification parser with semver, dependency graphs, and scoring.

## Features

- **`CapabilitySchema`** — Full CAPABILITY.toml schema with serde support
- **`parser`** — TOML parsing and validation
- **`SemVer`** — Semantic versioning with comparison and compatibility
- **`DependencyGraph`** — Directed graph with Kahn's algorithm topological sort
- **`scoring`** — Capability scoring with recency weights

## Usage

```rust
use capability_spec::parser::parse_capability_toml;
use capability_spec::semver::SemVer;
use capability_spec::graph::DependencyGraph;

let schema = parse_capability_toml(r#"
[agent]
name = "my-agent"
type = "vessel"

[capabilities.code_gen]
confidence = 0.9
"#).unwrap();

let v = SemVer::parse("1.2.3").unwrap();
assert!(v > SemVer::new(1, 2, 0));

let mut g = DependencyGraph::new();
g.add_edge("a", "b");
assert_eq!(g.topological_sort().unwrap().len(), 2);
```

## License

MIT
