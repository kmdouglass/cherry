/// A dependency graph for Views.
///
/// This module is modeled after the graph implementation presented at https://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/.
/// In effect, each node is the start of a linked list of edges, where each edge
/// points to the next edge in the list. All edges in any linked list share the
/// same source node.

type NodeIndex = usize;

type EdgeIndex = usize;

#[derive(Debug)]
struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Debug)]
struct Node {
    first_outgoing_edge: Option<EdgeIndex>,
}

#[derive(Debug)]
struct Edge {
    target: NodeIndex,
    next_outgoing_edge: Option<EdgeIndex>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(Node {
            first_outgoing_edge: None,
        });
        index
    }

    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex) {
        // TODO Add duplicate check if necessary
        let edge_index = self.edges.len();
        let node_data = &mut self.nodes[source];
        self.edges.push(Edge {
            target: target,
            next_outgoing_edge: node_data.first_outgoing_edge,
        });
        node_data.first_outgoing_edge = Some(edge_index);
    }

    /// Determines whether the graph is cyclic.
    fn is_cyclic(&self) -> bool {
        let mut visited = vec![false; self.nodes.len()];
        let mut rec_stack = vec![false; self.nodes.len()];

        for i in 0..self.nodes.len() {
            if !visited[i] {
                if self.is_cyclic_util(i, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }
        false
    }

    /// Used in a depth-first search to detect cycles in the graph.
    fn is_cyclic_util(
        &self,
        node_index: usize,
        visited: &mut Vec<bool>,
        rec_stack: &mut Vec<bool>,
    ) -> bool {
        if !visited[node_index] {
            visited[node_index] = true;
            rec_stack[node_index] = true;

            let mut edge_index = self.nodes[node_index].first_outgoing_edge;
            while let Some(e_idx) = edge_index {
                let target = self.edges[e_idx].target;
                if !visited[target] && self.is_cyclic_util(target, visited, rec_stack) {
                    return true;
                } else if rec_stack[target] {
                    return true;
                }
                edge_index = self.edges[e_idx].next_outgoing_edge;
            }
        }
        rec_stack[node_index] = false;
        false
    }

    /// Returns a topological sorting of the graph.
    fn sort(&self) -> Vec<NodeIndex> {
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = Vec::new();

        for i in 0..self.nodes.len() {
            if !visited[i] {
                self.topological_sort_util(i, &mut visited, &mut stack);
            }
        }

        stack.reverse();
        stack
    }

    /// Used in a depth-first search for topological sorting of the graph.
    fn topological_sort_util(
        &self,
        node_index: usize,
        visited: &mut Vec<bool>,
        stack: &mut Vec<usize>,
    ) {
        visited[node_index] = true;

        let mut edge_index = self.nodes[node_index].first_outgoing_edge;
        while let Some(e_idx) = edge_index {
            let target = self.edges[e_idx].target;
            if !visited[target] {
                self.topological_sort_util(target, visited, stack);
            }
            edge_index = self.edges[e_idx].next_outgoing_edge;
        }

        stack.push(node_index); // Add this node to the stack after visiting all
                                // edges
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_graph() {
        let mut graph = Graph::new();
        let v0 = graph.add_node();
        let v1 = graph.add_node();
        let v2 = graph.add_node();
        let v3 = graph.add_node();

        graph.add_edge(v1, v0);
        graph.add_edge(v1, v2);
        graph.add_edge(v2, v0);

        assert_eq!(graph.is_cyclic(), false);
    }

    #[test]
    fn test_graph_cyclic() {
        let mut graph = Graph::new();
        let v0 = graph.add_node();
        let v1 = graph.add_node();

        graph.add_edge(v0, v1);
        graph.add_edge(v1, v0);

        assert_eq!(graph.is_cyclic(), true);
    }

    #[test]
    fn test_graph_cyclic_node_cycles_upon_itself() {
        let mut graph = Graph::new();
        let v0 = graph.add_node();

        graph.add_edge(v0, v0);

        assert_eq!(graph.is_cyclic(), true);
    }

    #[test]
    fn test_sort() {
        let mut graph = Graph::new();
        let v0 = graph.add_node();
        let v1 = graph.add_node();
        let v2 = graph.add_node();
        let v3 = graph.add_node();

        graph.add_edge(v1, v0);
        graph.add_edge(v1, v2);
        graph.add_edge(v2, v0);

        let sorted = graph.sort();
        assert_eq!(sorted, vec![v3, v1, v2, v0]);
    }
}
