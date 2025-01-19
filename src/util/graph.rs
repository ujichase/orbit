//
//  Copyright (C) 2022-2025  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

/// Basic graph data structure
/// - source: http://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
use std::collections::HashSet;

use super::graphmap::GraphMap;

type NodeIndex = usize;

#[derive(Debug, PartialEq)]
struct NodeData<V> {
    node: V,
    first_outgoing_edge: Option<EdgeIndex>,
    first_incoming_edge: Option<EdgeIndex>,
}

type EdgeIndex = usize;

#[derive(Debug, PartialEq)]
struct EdgeData<E> {
    edge: E,
    source: NodeIndex,
    target: NodeIndex,
    next_outgoing_edge: Option<EdgeIndex>,
    next_incoming_edge: Option<EdgeIndex>,
}

#[derive(Debug, PartialEq)]
pub struct Graph<V, E> {
    vertices: Vec<NodeData<V>>,
    edges: Vec<EdgeData<E>>,
}

impl<V, E> std::fmt::Display for Graph<V, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, data) in self.vertices.iter().enumerate() {
            write!(
                f,
                "i:{} out:{:?} in:{:?}\n",
                i, data.first_outgoing_edge, data.first_incoming_edge
            )?
        }
        write!(f, "\n")?;
        for (i, data) in self.edges.iter().enumerate() {
            write!(
                f,
                "i:{}, t:{} s:{} out:{:?} in:{:?}\n",
                i, data.target, data.source, data.next_outgoing_edge, data.next_incoming_edge
            )?
        }
        Ok(())
    }
}

impl<V, E> Graph<V, E> {
    /// Creates an empty `Graph` struct.
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            vertices: Vec::new(),
        }
    }

    /// Creates an empty `Graph` struct with reserved capacities for `nodes` and `edges`.
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(nodes),
            edges: Vec::with_capacity(edges),
        }
    }

    /// Adds a new node to the graph.
    ///
    /// Returns the `NodeIndex` to remember the node.
    pub fn add_node(&mut self, node: V) -> NodeIndex {
        let index = self.vertices.len();
        self.vertices.push(NodeData {
            node: node,
            first_outgoing_edge: None,
            first_incoming_edge: None,
        });
        index
    }

    /// Checks if a given `source` node is in the graph.
    pub fn has_node(&self, source: NodeIndex) -> bool {
        source < self.node_count()
    }

    /// Checks if a given `source` node is connected to the given `target` node.
    pub fn has_edge(&self, source: NodeIndex, target: NodeIndex) -> bool {
        let mut successors = self.successors(source);
        successors.find(|f| f == &target).is_some()
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Accesses the node data label behind the `node` index.
    pub fn get_node(&self, node: NodeIndex) -> Option<&V> {
        Some(&self.vertices.get(node)?.node)
    }

    /// Adds a new edge to the graph from `source` to `target`.
    ///
    /// Returns true if the edge insertion was successful. Returns false if the edge
    /// relationship already exists or the edge is a self-loop.
    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, cost: E) -> EdgeStatus {
        // do not allow duplicate edges
        if self.has_edge(source, target) == true {
            return EdgeStatus::AlreadyExists;
        }
        // do not allow self-loops
        if source == target {
            return EdgeStatus::SelfLoop;
        }

        let edge_index = self.edges.len();
        // enter source -> target data
        {
            let node_data = &mut self.vertices[source];
            self.edges.push(EdgeData {
                source: source,
                edge: cost,
                target: target,
                next_outgoing_edge: node_data.first_outgoing_edge,
                next_incoming_edge: None,
            });
            node_data.first_outgoing_edge = Some(edge_index);
        }
        // enter target <- source data
        let rev_node_data = &mut self.vertices[target];
        let incoming_edge = rev_node_data.first_incoming_edge;
        rev_node_data.first_incoming_edge = Some(edge_index);
        // update the edge data
        self.edges.last_mut().unwrap().next_incoming_edge = incoming_edge;
        EdgeStatus::Success
    }

    /// Returns the number of successors to the `source` node.
    pub fn out_degree(&self, source: NodeIndex) -> usize {
        self.successors(source).count()
    }

    /// Returns the number of predecessors to the `source` node.
    pub fn in_degree(&self, target: NodeIndex) -> usize {
        self.predecessors(target).count()
    }

    /// Creates an iterator over the incoming nodes to the `target` source.
    pub fn predecessors(&self, target: NodeIndex) -> Predecessors<V, E> {
        let first_incoming_edge = self.vertices[target].first_incoming_edge;
        Predecessors {
            graph: self,
            current_edge_index: first_incoming_edge,
        }
    }

    /// Creates an iterator over the outgoing nodes from the `source` node.
    pub fn successors(&self, source: NodeIndex) -> Successors<V, E> {
        let first_outgoing_edge = self.vertices[source].first_outgoing_edge;
        Successors {
            graph: self,
            current_edge_index: first_outgoing_edge,
        }
    }

    pub fn iter(&self) -> IterGraph<V, E> {
        IterGraph {
            graph: self,
            current_node_index: 0,
        }
    }

    /// Checks if the graph has zero nodes.
    pub fn is_empty(&self) -> bool {
        self.node_count() == 0
    }

    /// Checks if the graph contains a cycle.
    pub fn is_cyclic(&self) -> bool {
        if self.is_empty() {
            return false;
        }

        for i in 0..self.node_count() {
            if self.in_degree(i) == 0 {
                return false;
            }
        }
        true
    }

    /// Determines which node has zero outgoing edges as the 'root' node.
    ///
    /// Returns a list of possible roots (potentially zero) as an err if there is
    /// not just a definite single root.
    pub fn find_root(&self) -> Result<NodeIndex, Vec<NodeIndex>> {
        // grab a list of all nodes that have zero outgoing edges
        let roots: Vec<NodeIndex> = self
            .vertices
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if n.first_outgoing_edge.is_none() {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();
        // check how many roots were detected
        match roots.len() {
            1 => Ok(*roots.first().unwrap()),
            _ => Err(roots),
        }
    }

    /// Performs depth-first search algorithm starting from the `target` node.
    pub fn depth_first_search(&self, target: NodeIndex) -> Vec<NodeIndex> {
        let mut traversal = Vec::new();
        // add target to the list
        traversal.push(target);
        // select predecessors
        let mut tunnels: Box<dyn Iterator<Item = usize>> = Box::new(self.predecessors(target));
        while let Some(n) = tunnels.next() {
            // select node
            traversal.push(n);
            // add its predecessors to list to process (in front of current nodes to process)
            tunnels = Box::new(self.predecessors(n).chain(tunnels));
        }
        traversal
    }

    /// Performs topological sort to give in-order nodes to perform given tasks
    /// based upon dependencies.
    pub fn topological_sort(&self) -> Vec<NodeIndex> {
        let mut order = Vec::<NodeIndex>::with_capacity(self.node_count());
        // store the set of remaining nodes with incoming edges
        let mut store: Vec<Option<HashSet<NodeIndex>>> = Vec::with_capacity(self.node_count());
        let mut able_to_complete: Option<NodeIndex> = None;
        for i in 0..self.node_count() {
            let deps = self.predecessors(i).collect::<HashSet<NodeIndex>>();
            // mark first node which has 0 dependencies
            if deps.len() == 0 && able_to_complete.is_none() {
                able_to_complete = Some(i);
            }
            store.push(Some(deps));
        }
        // continue processing tasks able to be completed
        while let Some(current) = able_to_complete {
            // take from store
            store[current] = None;
            able_to_complete = None;
            for (i, task) in store.iter_mut().enumerate() {
                if let Some(deps) = task {
                    // remove current task from this task's dependencies
                    deps.remove(&current);
                    // find node with zero predecessors
                    if deps.len() == 0 && able_to_complete.is_none() {
                        able_to_complete = Some(i);
                    }
                }
            }
            order.push(current);
        }
        order
    }

    /// Performs topological sort on the entire graph and then only selects the
    /// minimal number of affected nodes needed process up to `target`.
    pub fn minimal_topological_sort(&self, target: NodeIndex) -> Vec<NodeIndex> {
        // order the nodes
        let total_order = self.topological_sort();
        // collect number of nodes a part of the partial tree starting from `target`
        let effected_nodes: HashSet<usize> = self.depth_first_search(target).into_iter().collect();
        // filter out all nodes not in the hashset
        total_order
            .into_iter()
            .filter(|f| effected_nodes.contains(f) == true)
            .collect()
    }

    /// Recursively generates the in-order y-down list of nodes to print with their
    /// corresponding twig style and level of indentation.
    fn recurse_treeview(&self, target: NodeIndex, level: Twig) -> Vec<(Twig, NodeIndex)> {
        let mut traversal = Vec::new();
        // add target to the list
        traversal.push((level.clone(), target));
        // select predecessors
        let mut tunnels = self.predecessors(target).peekable();
        while let Some(n) = tunnels.next() {
            // remember the order and parent branch type
            let twig_type = match tunnels.peek() {
                Some(_) => Twig::MidBranch(Some(Box::new(level.clone()))),
                None => Twig::EndLeaf(Some(Box::new(level.clone()))),
            };
            traversal.append(&mut self.recurse_treeview(n, twig_type));
        }
        traversal
    }

    /// Creates the in-order y-down list of nodes to display with their
    /// corresponding indentation depth and twig style.
    pub fn treeview(&self, target: NodeIndex) -> Vec<(Twig, NodeIndex)> {
        self.recurse_treeview(target, Twig::EndLeaf(None))
    }

    /// Removes duplicate branches from the treeview and replaces them with labels.
    pub fn compress_treeview(&self, _tree: &Vec<(Twig, NodeIndex)>) -> Vec<(Twig, NodeIndex)> {
        todo!()
    }
}

use std::hash::Hash;

impl<K, V, E> GraphMap<K, V, E>
where
    K: Eq + Hash + Clone,
{
    /// Creates an iterator over the outgoing nodes from the `source` node.
    pub fn successors(&self, source: NodeIndex) -> SuccessorsGraphMap<K, V, E> {
        let first_outgoing_edge = self.get_graph().vertices[source].first_outgoing_edge;
        SuccessorsGraphMap {
            graph: self,
            current_edge_index: first_outgoing_edge,
        }
    }

    pub fn predecessors(&self, source: NodeIndex) -> PredecessorsGraphMap<K, V, E> {
        let first_incoming_edge = self.get_graph().vertices[source].first_incoming_edge;
        PredecessorsGraphMap {
            graph: self,
            current_edge_index: first_incoming_edge,
        }
    }
}

pub struct SuccessorsGraphMap<'graph, K: Eq + Hash + Clone, V, E> {
    graph: &'graph GraphMap<K, V, E>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, K: Eq + Hash + Clone, V, E> Iterator for SuccessorsGraphMap<'graph, K, V, E> {
    type Item = (&'graph K, &'graph V, &'graph E);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.get_graph().edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some((
                    self.graph.get_key_by_index(edge.target).unwrap(),
                    self.graph.get_node_by_index(edge.target).unwrap().as_ref(),
                    &edge.edge,
                ))
            }
        }
    }
}

pub struct PredecessorsGraphMap<'graph, K: Eq + Hash + Clone, V, E> {
    graph: &'graph GraphMap<K, V, E>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, K: Eq + Hash + Clone, V, E> Iterator for PredecessorsGraphMap<'graph, K, V, E> {
    type Item = (&'graph K, &'graph V, &'graph E);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.get_graph().edges[edge_num];
                self.current_edge_index = edge.next_incoming_edge;
                Some((
                    self.graph.get_key_by_index(edge.source).unwrap(),
                    self.graph.get_node_by_index(edge.source).unwrap().as_ref(),
                    &edge.edge,
                ))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum EdgeStatus {
    MissingSource,
    MissingTarget,
    SelfLoop,
    AlreadyExists,
    Success,
}

impl EdgeStatus {
    pub fn is_ok(&self) -> bool {
        match self {
            Self::Success => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Twig {
    EndLeaf(Option<Box<Twig>>),
    MidBranch(Option<Box<Twig>>),
}

impl Twig {
    /// Accesses what type of node was the parent to the current `self`.
    pub fn get_upper(&self) -> Option<&Twig> {
        match self {
            Self::EndLeaf(e) => e.as_deref(),
            Self::MidBranch(e) => e.as_deref(),
        }
    }
}

impl std::fmt::Display for Twig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // determine the spacing leading up the node in the tree
        let space: String = {
            let mut space = String::new();
            let mut x = self;
            while let Some(n) = x.get_upper() {
                match n {
                    Self::EndLeaf(q) => {
                        if q.is_some() {
                            space.push_str("   ")
                        }
                    }
                    Self::MidBranch(q) => {
                        if q.is_some() {
                            space.push_str("  │")
                        }
                    }
                }
                x = n;
            }
            // reverse the characters around because twig chains need to be rewinded through recursion
            space.chars().rev().collect()
        };

        match self {
            Self::EndLeaf(m) => {
                if m.is_none() {
                    write!(f, "")
                } else {
                    write!(f, "{}└─ ", space)
                }
            }
            Self::MidBranch(_) => write!(f, "{}├─ ", space),
        }
    }
}

pub struct IterGraph<'graph, V, E> {
    graph: &'graph Graph<V, E>,
    current_node_index: NodeIndex,
}

impl<'graph, V, E> Iterator for IterGraph<'graph, V, E> {
    type Item = &'graph V;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_node_index >= self.graph.vertices.len() {
            None
        } else {
            let n = Some(&self.graph.vertices[self.current_node_index].node);
            self.current_node_index += 1;
            n
        }
    }
}

pub struct Predecessors<'graph, V, E> {
    graph: &'graph Graph<V, E>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, V, E> Iterator for Predecessors<'graph, V, E> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num];
                self.current_edge_index = edge.next_incoming_edge;
                Some(edge.source)
            }
        }
    }
}

pub struct Successors<'graph, V, E> {
    graph: &'graph Graph<V, E>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph, V, E> Iterator for Successors<'graph, V, E> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some(edge.target)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Transforms a tree into a string for easier verification.
    fn tree_to_string(t: &Vec<(Twig, usize)>) -> String {
        let mut display = String::new();
        for node in t {
            display.push_str(&format!("{}{}\n", node.0, node.1));
        }
        display
    }

    #[test]
    fn treeview() {
        let mut g = binary_tree();
        g.add_edge(4, 2, ());
        let tree = g.treeview(0);
        assert_eq!(
            tree_to_string(&tree),
            "\
0
├─ 4
│  ├─ 6
│  └─ 5
└─ 1
   ├─ 3
   └─ 2
      └─ 4
         ├─ 6
         └─ 5
"
        );
    }

    #[test]
    fn treeview_hard() {
        let mut graph = Graph::<(), ()>::new();
        let z = graph.add_node(());
        let y = graph.add_node(());
        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        let e = graph.add_node(());
        let f = graph.add_node(());
        let g = graph.add_node(());
        let h = graph.add_node(());

        graph.add_edge(y, z, ());

        graph.add_edge(a, y, ());
        graph.add_edge(b, y, ());
        graph.add_edge(d, y, ());
        graph.add_edge(c, y, ());

        graph.add_edge(e, c, ());
        graph.add_edge(f, c, ());

        graph.add_edge(g, d, ());
        graph.add_edge(g, b, ());

        graph.add_edge(h, g, ());

        let tree = graph.treeview(z);
        assert_eq!(
            tree_to_string(&tree),
            "\
0
└─ 1
   ├─ 4
   │  ├─ 7
   │  └─ 6
   ├─ 5
   │  └─ 8
   │     └─ 9
   ├─ 3
   │  └─ 8
   │     └─ 9
   └─ 2
"
        );
    }
    /* --ascii version
    0
    \─ 1
       +─ 4
       |  +─ 7
       |  \- 6
       +─ 5
       |  \─ 8
       |     \─ 9
       +─ 3
       |  \─ 8
       |     \─ 9
       \─ 2
    */

    /// Creates basic graph illustrated in this blog post:
    /// - source: http://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
    fn basic_graph() -> Graph<(), ()> {
        let mut g = Graph::new();
        let n0 = g.add_node(());
        let n1 = g.add_node(());
        let n2 = g.add_node(());
        let n3 = g.add_node(());
        g.add_edge(n0, n1, ());
        g.add_edge(n1, n2, ());
        g.add_edge(n0, n3, ());
        g.add_edge(n3, n2, ());
        g
    }

    /// Creates an example binary tree with height = 2. Edges are directed up indicating
    /// the child is a dependency to the parent.
    ///
    ///         n0
    ///        /  \
    ///     n1     n4
    ///    /  \   /  \
    /// n2   n3 n5   n6
    fn binary_tree() -> Graph<(), ()> {
        // create binary tree
        let mut g = Graph::new();
        let n0 = g.add_node(());
        let n1 = g.add_node(());
        let n2 = g.add_node(());
        let n3 = g.add_node(());
        let n4 = g.add_node(());
        let n5 = g.add_node(());
        let n6 = g.add_node(());
        // level 1
        g.add_edge(n1, n0, ()); // n1 -> n0
        g.add_edge(n4, n0, ()); // n4 -> n0
                                // level 2 - L
        g.add_edge(n2, n1, ()); // n2 -> n1
        g.add_edge(n3, n1, ()); // n3 -> n1
                                // level 2 - R
        g.add_edge(n5, n4, ()); // n5 -> n4
        g.add_edge(n6, n4, ()); // n6 -> n4
        g
    }

    #[test]
    fn is_cyclic() {
        let mut g = basic_graph();
        assert_eq!(g.is_cyclic(), false);
        // add in edge to make cycle
        g.add_edge(1, 0, ());
        assert_eq!(g.is_cyclic(), true);

        let mut g = basic_graph();
        g.add_edge(2, 0, ());
        assert_eq!(g.is_cyclic(), true);

        let g = binary_tree();
        assert_eq!(g.is_cyclic(), false);
    }

    #[test]
    fn topological_sort() {
        let mut g = basic_graph();
        assert_eq!(g.topological_sort(), vec![0, 1, 3, 2]);

        let n0 = g.add_node(());
        g.add_edge(n0, 0, ());
        assert_eq!(g.topological_sort(), vec![4, 0, 1, 3, 2]);

        let g = binary_tree();
        assert_eq!(g.topological_sort(), vec![2, 3, 1, 5, 6, 4, 0]);
    }

    #[test]
    fn add_node_and_has_node() {
        let mut g: Graph<(), ()> = Graph::new();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.has_node(0), false);
        assert_eq!(g.has_node(1), false);
        g.add_node(());
        assert_eq!(g.node_count(), 1);
        assert_eq!(g.has_node(0), true);
        assert_eq!(g.has_node(1), false);
    }

    #[test]
    fn add_edge() {
        let mut g = Graph::new();
        assert_eq!(g.edge_count(), 0);
        let n0 = g.add_node(());
        let n1 = g.add_node(());
        assert_eq!(g.edge_count(), 0);
        assert_eq!(g.add_edge(n0, n1, ()).is_ok(), true);
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.add_edge(n1, n0, ()).is_ok(), true);
        assert_eq!(g.edge_count(), 2);
        // do not allow duplicate edges
        assert_eq!(g.add_edge(n1, n0, ()).is_ok(), false);
        assert_eq!(g.edge_count(), 2);
        // do not allow self-loops
        assert_eq!(g.add_edge(n0, n0, ()).is_ok(), false);
    }

    #[test]
    fn find_root() {
        let mut g = binary_tree();
        assert_eq!(g.find_root(), Ok(0));

        // add a second possible root to the tree
        let n7 = g.add_node(());
        g.add_edge(4, n7, ());
        assert_eq!(g.find_root(), Err(vec![0, 7]));

        // add edges between the two known roots: n0 and n7
        g.add_edge(0, n7, ());
        g.add_edge(n7, 0, ());
        assert_eq!(g.find_root(), Err(vec![]));
    }

    #[test]
    fn out_degree() {
        let g = basic_graph();
        assert_eq!(g.out_degree(0), 2);
        assert_eq!(g.out_degree(1), 1);
        assert_eq!(g.out_degree(2), 0);
        assert_eq!(g.out_degree(3), 1);
    }

    #[test]
    fn in_degree() {
        let g = basic_graph();
        assert_eq!(g.in_degree(0), 0);
        assert_eq!(g.in_degree(1), 1);
        assert_eq!(g.in_degree(2), 2);
        assert_eq!(g.in_degree(3), 1);
    }

    #[test]
    fn successors() {
        let g = basic_graph();
        assert_eq!(g.successors(0).collect::<Vec<NodeIndex>>(), vec![3, 1]);
        assert_eq!(g.successors(1).collect::<Vec<NodeIndex>>(), vec![2]);
        assert_eq!(
            g.successors(2).collect::<Vec<NodeIndex>>(),
            Vec::<usize>::new()
        );
        assert_eq!(g.successors(3).collect::<Vec<NodeIndex>>(), vec![2]);
    }

    #[test]
    fn predecessors() {
        let g = basic_graph();
        assert_eq!(
            g.predecessors(0).collect::<Vec<usize>>(),
            Vec::<usize>::new()
        );
        assert_eq!(g.predecessors(1).collect::<Vec<usize>>(), vec![0]);
        assert_eq!(g.predecessors(2).collect::<Vec<usize>>(), vec![3, 1]);
        assert_eq!(g.predecessors(3).collect::<Vec<usize>>(), vec![0]);
    }

    #[test]
    fn min_y_sort() {
        let g = binary_tree();
        assert_eq!(g.minimal_topological_sort(0), vec![2, 3, 1, 5, 6, 4, 0]);

        assert_eq!(g.minimal_topological_sort(1), vec![2, 3, 1]);

        assert_eq!(g.minimal_topological_sort(4), vec![5, 6, 4]);
    }

    #[test]
    fn get_node() {
        let mut g = Graph::<&str, usize>::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        let c = g.add_node("c");

        g.add_edge(a, b, 3);
        g.add_edge(a, c, 10);
        g.add_edge(b, c, 8);

        assert_eq!(g.get_node(a).unwrap(), &"a");
        assert_eq!(g.get_node(b).unwrap(), &"b");

        assert_eq!(g.get_node(100), None);
    }

    #[test]
    fn has_edge() {
        let mut g = Graph::new();
        let n0 = g.add_node(());
        let n1 = g.add_node(());
        let n2 = g.add_node(());
        assert_eq!(g.has_edge(n0, n1), false);
        g.add_edge(n0, n1, ());
        assert_eq!(g.has_edge(n0, n1), true);
        assert_eq!(g.has_edge(n1, n0), false);

        assert_eq!(g.has_edge(n1, n2), false);
        g.add_edge(n1, n2, ());
        assert_eq!(g.has_edge(n1, n2), true);
    }

    #[test]
    fn dfs() {
        let g = binary_tree();
        // from y node
        assert_eq!(g.depth_first_search(0), vec![0, 4, 6, 5, 1, 3, 2]);

        // from intermediate node
        assert_eq!(g.depth_first_search(1), vec![1, 3, 2]);

        // from intermediate node
        assert_eq!(g.depth_first_search(4), vec![4, 6, 5]);

        // from leaf node
        assert_eq!(g.depth_first_search(6), vec![6]);
    }
}
