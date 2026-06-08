//! # capability-spec
//!
//! **Capability specification parser with semver, dependency graphs, and weighted scoring.**
//!
//! ## What Is a Capability Specification?
//!
//! A *capability specification* (`CAPABILITY.toml`) is a declarative manifest that describes
//! what a software agent **can do**, how it communicates, what resources it needs, and how it
//! fits into a larger fleet. Think of it as a résumé for autonomous agents — a machine-readable
//! contract that lets fleet orchestrators discover, compare, and dispatch work to the right agent.
//!
//! The spec covers:
//!
//! - **Agent metadata** — name, type, status, role, model, runtime info
//! - **Capabilities** — named skills with confidence scores, recency, versioning, and dependencies
//! - **Communication** — bottles, MUD, issues, PR reviews
//! - **Resources** — compute tier, CPU, RAM, storage, CUDA, languages
//! - **Constraints** — max task duration, approval requirements, refusals, budgets
//! - **Associates** — reporting structure, collaboration, trust scores
//!
//! ## The Fleet Agent Model
//!
//! In the SuperInstance ecosystem, agents are organized into a naval hierarchy:
//!
//! | Agent Type | Role |
//! |------------|------|
//! | **lighthouse** | Fleet coordinator — routes tasks, monitors health |
//! | **vessel** | General-purpose worker — handles assigned missions |
//! | **scout** | Lightweight explorer — probes, searches, gathers intel |
//! | **quartermaster** | Resource manager — allocates compute, storage, budgets |
//! | **barnacle** | Persistent attachment — stays with a single repo/project |
//! | **greenhorn** | New agent in training — limited capabilities, learning |
//!
//! ## Quick Start
//!
//! ### Parse a `CAPABILITY.toml`
//!
//! ```rust
//! use capability_spec::parser::{parse_capability_toml, validate};
//!
//! let schema = parse_capability_toml(r#"
//! version = "1.0.0"
//!
//! [agent]
//! name = "naval-vessel-7"
//! type = "vessel"
//! status = "active"
//!
//! [capabilities.code_gen]
//! confidence = 0.92
//! last_used = "2025-12-01"
//! description = "Generate production-grade code from prompts"
//! version = "2.1.0"
//!
//! [capabilities.code_review]
//! confidence = 0.85
//! last_used = "2025-12-03"
//! description = "Review code for quality and correctness"
//! requires = ["code_gen"]
//! version = "1.3.0"
//!
//! [communication]
//! bottles = true
//! issues = true
//! pr_reviews = true
//!
//! [resources]
//! compute = "high"
//! cpu_cores = 8.0
//! ram_gb = 32.0
//! languages = ["rust", "python", "typescript"]
//!
//! [constraints]
//! max_task_duration = "2h"
//! requires_approval = ["production_deploy"]
//! refuses = ["destructive_ops"]
//! "#).unwrap();
//!
//! // Validate the parsed schema
//! assert!(validate(&schema).is_ok());
//! assert_eq!(schema.agent.name, "naval-vessel-7");
//! assert_eq!(schema.capabilities.len(), 2);
//! ```
//!
//! ### Semantic Versioning
//!
//! ```rust
//! use capability_spec::semver::SemVer;
//!
//! let v = SemVer::parse("2.1.0").unwrap();
//! assert_eq!(v.to_string(), "2.1.0");
//!
//! // Compatibility: same major, minor >= target
//! assert!(SemVer::new(1, 5, 0).is_compatible(&SemVer::new(1, 2, 0)));
//! assert!(!SemVer::new(2, 0, 0).is_compatible(&SemVer::new(1, 0, 0)));
//!
//! // Breaking changes: major version differs
//! assert!(SemVer::new(2, 0, 0).is_breaking(&SemVer::new(1, 9, 9)));
//! ```
//!
//! ### Dependency Graph
//!
//! ```rust
//! use capability_spec::graph::DependencyGraph;
//! use std::collections::HashSet;
//!
//! let mut g = DependencyGraph::new();
//! g.add_edge("deploy", "review");  // deploy depends on review
//! g.add_edge("review", "code_gen"); // review depends on code_gen
//!
//! // Topological sort: code_gen → review → deploy
//! let sorted = g.topological_sort().unwrap();
//! assert_eq!(sorted, vec!["code_gen", "review", "deploy"]);
//!
//! // Reachability: what does deploy transitively depend on?
//! let reachable = g.reachability("deploy");
//! assert!(reachable.contains("code_gen"));
//!
//! // Lowest common ancestor
//! let lca = g.lowest_common_ancestor("review", "deploy");
//! assert_eq!(lca, Some("code_gen".to_string()));
//! ```
//!
//! ### Scoring
//!
//! ```rust
//! use capability_spec::scoring::score_schema;
//! use capability_spec::builder::CapabilitySchemaBuilder;
//!
//! let schema = CapabilitySchemaBuilder::new("my-agent")
//!     .agent_type("vessel")
//!     .capability("code_gen", 0.9, Some("2025-12-01"), Some("Code generation"))
//!     .capability("review", 0.8, Some("2025-12-03"), Some("Code review"))
//!     .build();
//!
//! let score = score_schema(&schema);
//! assert!(score > 0.0 && score <= 1.0);
//! ```
//!
//! ### Capability Matching
//!
//! ```rust
//! use capability_spec::matcher::CapabilityMatcher;
//! use capability_spec::builder::CapabilitySchemaBuilder;
//!
//! let agent_a = CapabilitySchemaBuilder::new("agent-a")
//!     .capability("code_gen", 0.9, None, None)
//!     .capability("review", 0.8, None, None)
//!     .build();
//!
//! let agent_b = CapabilitySchemaBuilder::new("agent-b")
//!     .capability("code_gen", 0.7, None, None)
//!     .capability("testing", 0.9, None, None)
//!     .build();
//!
//! let matcher = CapabilityMatcher::new(&agent_a, &agent_b);
//! assert_eq!(matcher.shared_capabilities(), vec!["code_gen"]);
//! assert!(matcher.compatibility_score() > 0.0);
//! ```
//!
//! ## Architecture
//!
//! The crate is organized into focused modules:
//!
//! ```text
//! ┌─────────────┐
//! │   parser     │  TOML → CapabilitySchema
//! └──────┬───────┘
//!        │
//! ┌──────▼───────┐
//! │   schema     │  Core data types (CapabilitySchema, Capability, AgentInfo, …)
//! └──────┬───────┘
//!        │
//! ┌──────┼───────────────────────────────┐
//! │      │                               │
//! │  ┌───▼────┐  ┌──────────┐  ┌────────▼───┐
//! │  │  graph  │  │  semver  │  │  scoring   │
//! │  │(deps)   │  │(versions)│  │(weights)   │
//! │  └─────────┘  └──────────┘  └────────────┘
//! │
//! │  ┌──────────┐  ┌──────────┐
//! │  │  matcher  │  │  builder │
//! │  │(compare)  │  │(fluent)  │
//! │  └──────────┘  └──────────┘
//! ```
//!
//! - **`schema`** — Core data types with full serde support
//! - **`parser`** — TOML parsing and validation
//! - **`semver`** — Semantic versioning with ordering and compatibility
//! - **`graph`** — Dependency resolution with topological sort (Kahn's algorithm)
//! - **`scoring`** — Weighted scoring with recency decay
//! - **`matcher`** — Compare two agents' capabilities
//! - **`builder`** — Fluent API for programmatic schema construction
//!
//! ## Example `CAPABILITY.toml`
//!
//! ```toml
//! version = "1.0.0"
//!
//! [agent]
//! name = "lighthouse-alpha"
//! type = "lighthouse"
//! status = "active"
//! role = "Fleet coordinator for the Pacific region"
//! avatar = "🗼"
//! home_repo = "github.com/org/fleet-lighthouse"
//! model = "gpt-4o"
//! last_active = "2025-12-15T08:30:00Z"
//!
//! [agent.runtime]
//! os = "linux"
//! arch = "x86_64"
//! rust_version = "1.75"
//!
//! [capabilities.fleet_coordination]
//! confidence = 0.95
//! last_used = "2025-12-15"
//! description = "Route and dispatch tasks across fleet agents"
//! version = "3.0.0"
//! requires = []
//!
//! [capabilities.health_monitoring]
//! confidence = 0.90
//! last_used = "2025-12-15"
//! description = "Monitor agent health and trigger alerts"
//! version = "2.1.0"
//! requires = ["fleet_coordination"]
//!
//! [communication]
//! bottles = true
//! bottle_path = "/tmp/bottles"
//! mud = true
//! mud_home = "pacific-fleet"
//! issues = true
//! pr_reviews = true
//!
//! [resources]
//! compute = "high"
//! cpu_cores = 16.0
//! ram_gb = 64.0
//! storage_gb = 500.0
//! cuda = false
//! languages = ["rust", "typescript", "python"]
//!
//! [constraints]
//! max_task_duration = "4h"
//! requires_approval = ["fleet_reconfig", "agent_decommission"]
//! refuses = ["destructive_ops", "data_exfiltration"]
//! budget_tokens_per_day = 500000.0
//!
//! [associates]
//! reports_to = "admiral"
//! collaborates = ["vessel-7", "scout-3"]
//! manages = ["vessel-7", "vessel-12", "scout-3"]
//!
//! [associates.trusts]
//! vessel-7 = 0.95
//! scout-3 = 0.80
//! ```

pub mod builder;
pub mod graph;
pub mod matcher;
pub mod parser;
pub mod schema;
pub mod scoring;
pub mod semver;

// Re-export the core types for convenience.
pub use builder::CapabilitySchemaBuilder;
pub use matcher::CapabilityMatcher;
pub use schema::{Capability, CapabilitySchema};
