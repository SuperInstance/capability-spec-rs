//! Dependency graph with topological sort (Kahn's algorithm).
//!
//! This module provides a directed graph for modelling capability dependencies.
//! Each node is a capability name, and edges represent "depends-on" relationships.
//! The primary use-case is ordering capabilities so that prerequisites are always
//! resolved before dependents.
//!
//! # Algorithms
//!
//! - **Topological sort** — [Kahn's algorithm](https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm)
//!   produces a linear ordering where every node appears after all nodes it depends on.
//!   Returns `None` if the graph contains a cycle.
//!
//! - **Cycle detection** — Checks whether adding a hypothetical edge would close a
//!   loop, by doing a reverse reachability walk from the target.
//!
//! - **Reachability** — Computes the transitive closure of all dependencies reachable
//!   from a given node.
//!
//! - **Lowest common ancestor** — Finds the deepest shared dependency between two nodes.
//!
//! # Example
//!
//! ```rust
//! use capability_spec::graph::DependencyGraph;
//! use std::collections::HashSet;
//!
//! let mut g = DependencyGraph::new();
//! g.add_edge("deploy", "review");   // deploy depends on review
//! g.add_edge("review", "code_gen"); // review depends on code_gen
//! g.add_edge("review", "testing");  // review also depends on testing
//!
//! // Topological order: code_gen and testing first, then review, then deploy
//! let sorted = g.topological_sort().unwrap();
//! let pos = |name: &str| sorted.iter().position(|x| x == name).unwrap();
//! assert!(pos("code_gen") < pos("review"));
//! assert!(pos("testing") < pos("review"));
//! assert!(pos("review") < pos("deploy"));
//!
//! // Reachability from deploy includes everything
//! let reachable = g.reachability("deploy");
//! assert_eq!(reachable.len(), 3);
//!
//! // LCA of code_gen and testing (both are deps of review)
//! let lca = g.lowest_common_ancestor("code_gen", "testing");
//! assert_eq!(lca, None); // they share no common dependency
//! ```

use std::collections::{HashMap, HashSet, VecDeque};

// ─────────────────────────────────────────────────────────────────────────────
// DependencyGraph
// ─────────────────────────────────────────────────────────────────────────────

/// A directed graph for dependency resolution.
///
/// Nodes are capability names (strings). An edge `A → B` means *A depends on B*,
/// i.e. B must be resolved before A. Internally we store `edges[from] = {to, …}`
/// where `to` is a dependency of `from`.
///
/// # Example
///
/// ```rust
/// use capability_spec::graph::DependencyGraph;
///
/// let mut g = DependencyGraph::new();
/// g.add_edge("deploy", "review");
/// g.add_edge("review", "code_gen");
///
/// assert_eq!(g.nodes().len(), 3);
/// assert!(g.dependencies("deploy").contains("review"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    /// All known nodes in the graph.
    nodes: HashSet<String>,

    /// Adjacency list: `edges[from]` is the set of nodes that `from` depends on.
    edges: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    /// Create an empty dependency graph.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::graph::DependencyGraph;
    ///
    /// let g = DependencyGraph::new();
    /// assert!(g.nodes().is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node (capability name) to the graph without any edges.
    ///
    /// If the node already exists this is a no-op.
    pub fn add_node(&mut self, name: &str) {
        self.nodes.insert(name.into());
        self.edges.entry(name.into()).or_default();
    }

    /// Add a dependency edge: `from` depends on `to`.
    ///
    /// Both nodes are implicitly added if they don't already exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::graph::DependencyGraph;
    ///
    /// let mut g = DependencyGraph::new();
    /// g.add_edge("deploy", "review"); // deploy depends on review
    /// assert!(g.dependencies("deploy").contains("review"));
    /// ```
    pub fn add_edge(&mut self, from: &str, to: &str) {
        // Ensure both endpoints exist in the node set.
        self.add_node(from);
        self.add_node(to);
        self.edges.get_mut(from).unwrap().insert(to.into());
    }

    /// Return the set of all nodes in the graph.
    pub fn nodes(&self) -> &HashSet<String> {
        &self.nodes
    }

    /// Return the direct dependencies of `node`.
    ///
    /// If `node` is not in the graph, returns an empty set.
    pub fn dependencies(&self, node: &str) -> HashSet<&str> {
        self.edges
            .get(node)
            .map(|s| s.iter().map(|x| x.as_str()).collect())
            .unwrap_or_default()
    }

    /// Topological sort using **Kahn's algorithm**.
    ///
    /// Produces a linear ordering such that for every edge `A → B` (A depends on B),
    /// B appears *before* A in the result. This is the canonical order for resolving
    /// dependencies: install/execute items from left to right.
    ///
    /// # Algorithm
    ///
    /// 1. Compute **in-degree** for each node (number of nodes that depend on it).
    ///    Wait — actually, in our representation `edges[from]` = set of `from`'s
    ///    dependencies. So the *in-degree* for topological sort is the number of
    ///    **dependents**, i.e. how many nodes list this node in their `edges`.
    ///    But we compute it as: `in_degree[node] = edges[node].len()` — the number
    ///    of *unresolved dependencies* this node has. Nodes with 0 in-degree (no
    ///    dependencies) go first.
    ///
    /// 2. Initialize a queue with all nodes whose in-degree is 0 (no dependencies).
    ///
    /// 3. Repeatedly pop a node from the queue, append it to the result, and
    ///    decrement the in-degree of any node that depends on it. If a dependent's
    ///    in-degree drops to 0, enqueue it.
    ///
    /// 4. If the result contains all nodes, return it. Otherwise there's a cycle.
    ///
    /// # Cycle detection
    ///
    /// Returns `None` if the graph contains a cycle — not all nodes can be ordered.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::graph::DependencyGraph;
    ///
    /// let mut g = DependencyGraph::new();
    /// g.add_edge("c", "a"); // c depends on a
    /// g.add_edge("c", "b"); // c depends on b
    /// g.add_edge("b", "a"); // b depends on a
    ///
    /// let sorted = g.topological_sort().unwrap();
    /// // a must come before b and c; b must come before c
    /// let pos = |name: &str| sorted.iter().position(|x| x == name).unwrap();
    /// assert!(pos("a") < pos("b"));
    /// assert!(pos("b") < pos("c"));
    /// ```
    pub fn topological_sort(&self) -> Option<Vec<String>> {
        // ── Step 1: Build reverse adjacency (dependents) and compute in-degrees ──
        //
        // `edges[from]` = {deps of from}, so the *reverse* map tells us:
        //   "who depends on this node?"  →  we need this to decrement in-degrees.
        //
        // In-degree here = number of unresolved dependencies a node has.
        // A node with in-degree 0 has no dependencies → goes first.

        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        // adj[x] = list of nodes that depend on x (reverse edges)

        // Initialize in-degree: each node starts with its number of dependencies.
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for node in &self.nodes {
            in_degree.insert(node.as_str(), 0);
            adj.insert(node.as_str(), Vec::new());
        }

        for (node, deps) in &self.edges {
            // in_degree[node] = how many things node depends on
            in_degree.insert(node.as_str(), deps.len());

            // Reverse edge: each dep has `node` as a dependent
            for dep in deps {
                if let Some(list) = adj.get_mut(dep.as_str()) {
                    list.push(node.as_str());
                }
            }
        }

        // ── Step 2: Seed the queue with nodes that have no dependencies ──

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&n, _)| n)
            .collect();

        // ── Step 3: Process the queue ──

        let mut result = Vec::with_capacity(self.nodes.len());
        while let Some(node) = queue.pop_front() {
            // This node is fully resolved — add to output.
            result.push(node.to_string());

            // For every node that depends on this one, decrement its in-degree.
            if let Some(dependents) = adj.get(node) {
                for &dependent in dependents {
                    let deg = in_degree.get_mut(dependent).unwrap();
                    *deg -= 1;
                    // If all dependencies are now resolved, enqueue it.
                    if *deg == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }

        // ── Step 4: Check for cycles ──
        // If we couldn't visit all nodes, there's a cycle.

        if result.len() == self.nodes.len() {
            Some(result)
        } else {
            None
        }
    }

    /// Check whether adding a dependency edge `from → to` would create a cycle.
    ///
    /// Does a reverse walk from `to` through the existing edges: if we can reach
    /// `from`, then adding `to → from` (via the new edge) would close a loop.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::graph::DependencyGraph;
    ///
    /// let mut g = DependencyGraph::new();
    /// g.add_edge("a", "b");
    /// g.add_edge("b", "c");
    ///
    /// assert!(g.would_create_cycle("c", "a"));  // c→a would close the loop
    /// assert!(!g.would_create_cycle("a", "d")); // d is new — no cycle
    /// ```
    pub fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        // Walk from `to` through its dependencies. If we can reach `from`,
        // then the new edge from→to would create: from→to→…→from (cycle).
        let mut visited = HashSet::new();
        let mut stack = vec![to];
        while let Some(node) = stack.pop() {
            if node == from {
                return true; // Found a path back to `from` → cycle.
            }
            if visited.insert(node.to_string()) {
                // Expand: follow all dependencies of this node.
                if let Some(deps) = self.edges.get(node) {
                    for dep in deps {
                        stack.push(dep.as_str());
                    }
                }
            }
        }
        false
    }

    /// Compute all nodes transitively reachable from `start`.
    ///
    /// Follows dependency edges to find everything that `start` depends on,
    /// directly or indirectly. The `start` node itself is **not** included
    /// in the result.
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::graph::DependencyGraph;
    ///
    /// let mut g = DependencyGraph::new();
    /// g.add_edge("c", "b");
    /// g.add_edge("b", "a");
    ///
    /// let reachable = g.reachability("c");
    /// assert!(reachable.contains("a"));
    /// assert!(reachable.contains("b"));
    /// assert!(!reachable.contains("c")); // start not included
    /// ```
    pub fn reachability(&self, start: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut stack = vec![start];

        while let Some(node) = stack.pop() {
            if let Some(deps) = self.edges.get(node) {
                for dep in deps {
                    // Only visit each node once to avoid infinite loops in cycles.
                    if visited.insert(dep.clone()) {
                        stack.push(dep.as_str());
                    }
                }
            }
        }

        visited
    }

    /// Find the **lowest common ancestor** (LCA) of two nodes.
    ///
    /// The LCA is the deepest shared dependency — the most specific node that
    /// both `a` and `b` transitively depend on. If they share no common
    /// dependency, returns `None`.
    ///
    /// # Algorithm
    ///
    /// 1. Compute the full transitive reachability set for `a`.
    /// 2. Compute the full transitive reachability set for `b`.
    /// 3. Intersect the two sets to find common dependencies.
    /// 4. Filter out any common node that is an ancestor of another common node
    ///    (i.e. keep only the "deepest" common ancestors).
    ///
    /// # Example
    ///
    /// ```rust
    /// use capability_spec::graph::DependencyGraph;
    ///
    /// let mut g = DependencyGraph::new();
    /// g.add_edge("deploy", "review");
    /// g.add_edge("deploy", "testing");
    /// g.add_edge("review", "code_gen");
    /// g.add_edge("testing", "code_gen");
    ///
    /// // Both review and testing transitively depend on code_gen.
    /// let lca = g.lowest_common_ancestor("review", "testing");
    /// assert_eq!(lca, Some("code_gen".to_string()));
    /// ```
    pub fn lowest_common_ancestor(&self, a: &str, b: &str) -> Option<String> {
        // Step 1–2: Compute transitive deps for both nodes.
        let reach_a = self.reachability(a);
        let reach_b = self.reachability(b);

        // Step 3: Find common dependencies.
        let common: HashSet<_> = reach_a.intersection(&reach_b).cloned().collect();
        if common.is_empty() {
            return None;
        }

        // Step 4: Find the "lowest" (most specific) common ancestor.
        // A node X is the LCA if no other common node depends on X.
        // In other words, X is LCA if it's not in the reachability set
        // of any other common node.
        for candidate in &common {
            let mut is_lca = true;
            for other in &common {
                if candidate != other {
                    // Check if candidate is reachable from other.
                    let reachable_from_other = self.reachability(other);
                    if reachable_from_other.contains(candidate.as_str()) {
                        // candidate is an ancestor of other → not the lowest.
                        is_lca = false;
                        break;
                    }
                }
            }
            if is_lca {
                return Some(candidate.clone());
            }
        }

        // Fallback: return any common node (shouldn't normally reach here).
        common.into_iter().next()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort_simple() {
        let mut g = DependencyGraph::new();
        g.add_edge("a", "b"); // a depends on b
        g.add_edge("b", "c"); // b depends on c
        let sorted = g.topological_sort().unwrap();
        // c (no deps) → b (dep: c) → a (dep: b)
        assert!(sorted.iter().position(|x| x == "c") < sorted.iter().position(|x| x == "b"));
        assert!(sorted.iter().position(|x| x == "b") < sorted.iter().position(|x| x == "a"));
    }

    #[test]
    fn test_cycle_detection() {
        let mut g = DependencyGraph::new();
        g.add_edge("a", "b");
        g.add_edge("b", "a");
        assert!(g.topological_sort().is_none());
    }

    #[test]
    fn test_would_create_cycle() {
        let mut g = DependencyGraph::new();
        g.add_edge("a", "b");
        g.add_edge("b", "c");
        assert!(g.would_create_cycle("c", "a"));
        assert!(!g.would_create_cycle("a", "d"));
    }

    #[test]
    fn test_empty_graph() {
        let g = DependencyGraph::new();
        let sorted = g.topological_sort().unwrap();
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_reachability() {
        let mut g = DependencyGraph::new();
        g.add_edge("c", "b");
        g.add_edge("b", "a");

        let reachable = g.reachability("c");
        assert!(reachable.contains("a"));
        assert!(reachable.contains("b"));
        assert!(!reachable.contains("c")); // start not included
    }

    #[test]
    fn test_reachability_branching() {
        let mut g = DependencyGraph::new();
        g.add_edge("deploy", "review");
        g.add_edge("deploy", "testing");
        g.add_edge("review", "code_gen");
        g.add_edge("testing", "code_gen");

        let reachable = g.reachability("deploy");
        assert_eq!(reachable.len(), 3); // review, testing, code_gen
    }

    #[test]
    fn test_reachability_empty() {
        let g = DependencyGraph::new();
        assert!(g.reachability("nonexistent").is_empty());
    }

    #[test]
    fn test_lowest_common_ancestor() {
        let mut g = DependencyGraph::new();
        g.add_edge("deploy", "review");
        g.add_edge("deploy", "testing");
        g.add_edge("review", "code_gen");
        g.add_edge("testing", "code_gen");

        let lca = g.lowest_common_ancestor("review", "testing");
        assert_eq!(lca, Some("code_gen".to_string()));
    }

    #[test]
    fn test_lowest_common_ancestor_no_common() {
        let mut g = DependencyGraph::new();
        g.add_edge("a", "x");
        g.add_edge("b", "y");

        assert_eq!(g.lowest_common_ancestor("a", "b"), None);
    }

    #[test]
    fn test_add_node_idempotent() {
        let mut g = DependencyGraph::new();
        g.add_node("a");
        g.add_node("a");
        assert_eq!(g.nodes().len(), 1);
    }

    #[test]
    fn test_dependencies_unknown_node() {
        let g = DependencyGraph::new();
        assert!(g.dependencies("nonexistent").is_empty());
    }
}
