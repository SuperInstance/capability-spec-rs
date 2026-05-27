//! Dependency graph using Kahn's algorithm for topological sort.

use std::collections::{HashMap, HashSet, VecDeque};

/// A directed graph for dependency resolution.
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    nodes: HashSet<String>,
    edges: HashMap<String, HashSet<String>>, // node -> set of dependencies
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, name: &str) {
        self.nodes.insert(name.into());
        self.edges.entry(name.into()).or_default();
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.add_node(from);
        self.add_node(to);
        self.edges.get_mut(from).unwrap().insert(to.into());
    }

    pub fn nodes(&self) -> &HashSet<String> {
        &self.nodes
    }

    pub fn dependencies(&self, node: &str) -> HashSet<&str> {
        self.edges.get(node).map(|s| s.iter().map(|x| x.as_str()).collect()).unwrap_or_default()
    }

    /// Topological sort using Kahn's algorithm. Returns None if cycle detected.
    pub fn topological_sort(&self) -> Option<Vec<String>> {
        let mut in_degree: HashMap<&str, usize> = self.nodes.iter().map(|n| (n.as_str(), 0)).collect();
        let mut adj: HashMap<&str, Vec<&str>> = self.nodes.iter().map(|n| (n.as_str(), Vec::new())).collect();

        for (node, deps) in &self.edges {
            for dep in deps {
                *in_degree.entry(node.as_str()).or_insert(0) += 0; // ensure exists
                adj.entry(dep.as_str()).or_default().push(node.as_str());
                // edge: node depends on dep, so dep -> node in adj
            }
        }

        // Recompute in_degree properly
        in_degree.clear();
        for n in &self.nodes {
            in_degree.insert(n.as_str(), 0);
        }
        for (node, deps) in &self.edges {
            in_degree.insert(node.as_str(), deps.len());
        }

        let mut queue: VecDeque<&str> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&n, _)| n)
            .collect();

        let mut result = Vec::new();
        while let Some(node) = queue.pop_front() {
            result.push(node.to_string());
            if let Some(dependents) = adj.get(node) {
                for &dep in dependents {
                    let deg = in_degree.get_mut(dep).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }

        if result.len() == self.nodes.len() {
            Some(result)
        } else {
            None // cycle detected
        }
    }

    /// Check if adding a dependency would create a cycle.
    pub fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![to];
        while let Some(node) = stack.pop() {
            if node == from { return true; }
            if visited.insert(node.to_string()) {
                if let Some(deps) = self.edges.get(node) {
                    for dep in deps {
                        stack.push(dep.as_str());
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort_simple() {
        let mut g = DependencyGraph::new();
        g.add_edge("a", "b"); // a depends on b
        g.add_edge("b", "c"); // b depends on c
        let sorted = g.topological_sort().unwrap();
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
}
