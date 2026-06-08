# capability-spec-rs

**Production-grade capability specification parser with semver, dependency graphs, and weighted scoring.**

A capability specification (`CAPABILITY.toml`) is a declarative manifest that describes what a software agent **can do**, how it communicates, what resources it needs, and how it fits into a larger fleet. Think of it as a résumé for autonomous agents — a machine-readable contract for fleet orchestrators.

## What This Gives You

- **`CapabilitySchema`** — Full `CAPABILITY.tomL` schema with serde support
- **TOML parsing + validation** — Parse specs, validate confidence ranges, agent types, statuses
- **`SemVer`** — Semantic versioning with comparison, compatibility, and breaking-change detection
- **`DependencyGraph`** — Directed graph with Kahn's algorithm topological sort, reachability, and LCA
- **Capability scoring** — Weighted scores with recency decay for ranking
- **`CapabilityMatcher`** — Compare two agents' capabilities, compute compatibility and coverage
- **`CapabilitySchemaBuilder`** — Fluent API for programmatic schema construction

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
last_used = "2025-01-15"
description = "Generate code from prompts"
version = "2.1.0"

[capabilities.review]
confidence = 0.8
requires = ["code_gen"]
"#).unwrap();

assert_eq!(schema.agent.name, "my-agent");
assert_eq!(schema.capabilities.len(), 2);
assert!(validate(&schema).is_ok());
```

### Build a schema programmatically

```rust
use capability_spec::builder::CapabilitySchemaBuilder;

let schema = CapabilitySchemaBuilder::new("vessel-7")
    .agent_type("vessel")
    .status("active")
    .capability("code_gen", 0.92, Some("2025-12-01"), Some("Generate code"))
    .capability("review", 0.85, Some("2025-12-03"), Some("Review PRs"))
    .resource_compute("high")
    .resource_cpu(8.0)
    .language("rust")
    .constraint_max_duration("2h")
    .refuse("destructive_ops")
    .build();
```

### Semantic versioning

```rust
use capability_spec::semver::SemVer;

let v = SemVer::parse("1.2.3").unwrap();
assert!(v > SemVer::new(1, 2, 0));
assert!(SemVer::new(1, 5, 0).is_compatible(&SemVer::new(1, 2, 0)));
assert!(!SemVer::new(2, 0, 0).is_compatible(&SemVer::new(1, 2, 0)));
assert!(SemVer::new(2, 0, 0).is_breaking(&SemVer::new(1, 9, 9)));
```

### Dependency graph with topological sort

```rust
use capability_spec::graph::DependencyGraph;

let mut g = DependencyGraph::new();
g.add_edge("deploy", "review");   // deploy depends on review
g.add_edge("review", "code_gen"); // review depends on code_gen

let sorted = g.topological_sort().unwrap();
// code_gen first (no deps), then review, then deploy
assert_eq!(sorted, vec!["code_gen", "review", "deploy"]);

// Reachability: what does deploy transitively depend on?
let reachable = g.reachability("deploy");
assert!(reachable.contains("code_gen"));
assert!(reachable.contains("review"));
```

### Capability matching

```rust
use capability_spec::matcher::CapabilityMatcher;
use capability_spec::builder::CapabilitySchemaBuilder;

let a = CapabilitySchemaBuilder::new("agent-a")
    .capability("code_gen", 0.9, None, None)
    .capability("review", 0.8, None, None)
    .build();

let b = CapabilitySchemaBuilder::new("agent-b")
    .capability("code_gen", 0.7, None, None)
    .capability("search", 0.95, None, None)
    .build();

let matcher = CapabilityMatcher::new(&a, &b);
assert_eq!(matcher.shared_capabilities(), vec!["code_gen"]);
assert_eq!(matcher.gaps_for_b(), vec!["review"]);
assert!(matcher.compatibility_score() > 0.0);
```

### Scoring

```rust
use capability_spec::scoring::{score_schema, rank_capabilities};

let score = score_schema(&schema);
assert!(score > 0.0 && score <= 1.0);

let ranked = rank_capabilities(&schema);
// Capabilities sorted by effective score (confidence × recency)
```

## Agent Types

In the SuperInstance fleet, agents follow a naval hierarchy:

| Type | Role |
|------|------|
| **lighthouse** | Fleet coordinator — routes tasks, monitors health |
| **vessel** | General-purpose worker — handles assigned missions |
| **scout** | Lightweight explorer — probes, searches, gathers intel |
| **quartermaster** | Resource manager — allocates compute, storage, budgets |
| **barnacle** | Persistent attachment — stays with a single repo/project |
| **greenhorn** | New agent in training — limited capabilities, learning |

## Example CAPABILITY.toml

```toml
version = "1.0.0"

[agent]
name = "lighthouse-alpha"
type = "lighthouse"
status = "active"
role = "Fleet coordinator for the Pacific region"
avatar = "🗼"
home_repo = "github.com/org/fleet-lighthouse"
model = "gpt-4o"
last_active = "2025-12-15T08:30:00Z"

[agent.runtime]
os = "linux"
arch = "x86_64"

[capabilities.fleet_coordination]
confidence = 0.95
last_used = "2025-12-15"
description = "Route and dispatch tasks across fleet agents"
version = "3.0.0"

[capabilities.health_monitoring]
confidence = 0.90
last_used = "2025-12-15"
description = "Monitor agent health and trigger alerts"
version = "2.1.0"
requires = ["fleet_coordination"]

[communication]
bottles = true
bottle_path = "/tmp/bottles"
mud = true
mud_home = "pacific-fleet"
issues = true
pr_reviews = true

[resources]
compute = "high"
cpu_cores = 16.0
ram_gb = 64.0
storage_gb = 500.0
cuda = false
languages = ["rust", "typescript", "python"]

[constraints]
max_task_duration = "4h"
requires_approval = ["fleet_reconfig", "agent_decommission"]
refuses = ["destructive_ops", "data_exfiltration"]
budget_tokens_per_day = 500000.0

[associates]
reports_to = "admiral"
collaborates = ["vessel-7", "scout-3"]
manages = ["vessel-7", "vessel-12", "scout-3"]

[associates.trusts]
vessel-7 = 0.95
scout-3 = 0.80
```

## API Reference

### Modules

| Module | Description |
|--------|-------------|
| `schema` | Core data types: `CapabilitySchema`, `Capability`, `AgentInfo`, etc. |
| `parser` | TOML parsing and validation |
| `semver` | Semantic versioning with ordering and compatibility |
| `graph` | Dependency resolution with topological sort |
| `scoring` | Weighted scoring with recency decay |
| `matcher` | Compare two agents' capabilities |
| `builder` | Fluent API for schema construction |

### Key Methods

**`parser`**
- `parse_capability_toml(text)` → Parse TOML string
- `parse_capability_file(path)` → Parse from disk
- `validate(schema)` → Check invariants

**`SemVer`**
- `new(major, minor, patch)` / `parse(s)`
- `is_compatible(other)` / `is_breaking(other)`
- Full `Ord + Eq + Display`

**`DependencyGraph`**
- `add_edge(from, to)` / `topological_sort()`
- `reachability(node)` / `lowest_common_ancestor(a, b)`
- `would_create_cycle(from, to)`

**`scoring`**
- `score_capability(cap)` / `score_schema(schema)`
- `match_capabilities(a, b)` / `rank_capabilities(schema)`

**`CapabilityMatcher`**
- `shared_capabilities()` / `gaps_for_a()` / `gaps_for_b()`
- `compatibility_score()` / `coverage_of_a()` / `coverage_of_b()`

**`CapabilitySchemaBuilder`**
- `new(name)` → Start building
- `.agent_type()` / `.capability()` / `.resource_*()` / `.build()`

## Fleet Integration

- **[cocapn-health-rs](https://github.com/SuperInstance/cocapn-health-rs)** — Health checks use capability specs to understand what each agent can do
- **[categorical-agents](https://github.com/SuperInstance/categorical-agents)** — Category theory objects map to capabilities defined in specs
- **[conservation-protocol](https://github.com/SuperInstance/conservation-protocol)** — Agent identity derived from capability structure
- **[co-captain-git-agent](https://github.com/SuperInstance/co-captain-git-agent)** — Fleet dispatch routes tasks to agents matching capability specs

## Testing

```bash
cargo test          # 40+ tests
cargo doc --no-deps # Generate docs
cargo clippy -- -D warnings
```

## Installation

```toml
[dependencies]
capability-spec = { git = "https://github.com/SuperInstance/capability-spec-rs" }
```

## License

MIT

Part of the [SuperInstance OpenConstruct](https://github.com/SuperInstance) ecosystem.
