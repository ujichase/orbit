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

use std::{collections::HashMap, hash::Hash, iter::FromIterator};

use super::graph::{EdgeStatus, Graph, PredecessorsGraphMap, SuccessorsGraphMap};

pub struct GraphMap<K: Eq + Hash + Clone, V, E> {
    graph: Graph<K, E>,
    map: HashMap<K, Node<V>>,
}

#[derive(Clone)]
pub struct Node<V>(V, usize);

impl<V> Node<V> {
    pub fn index(&self) -> usize {
        self.1
    }

    pub fn as_ref(&self) -> &V {
        &self.0
    }

    pub fn as_ref_mut(&mut self) -> &mut V {
        &mut self.0
    }

    pub fn take(self) -> V {
        self.0
    }
}

impl<K: Eq + Hash + Clone, V, E> GraphMap<K, V, E> {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, key: K, value: V) -> usize {
        let iden = self.graph.add_node(key.clone());
        self.map.insert(key, Node(value, iden));
        iden
    }

    pub fn has_node_by_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Creates an edge between `source` and `target`.
    ///
    /// Returns `true` if the edge insertion was successful. Returns `false` if
    /// either endpoint does not exist in the map, the edge already exists,
    /// or the edge is a self-loop.
    pub fn add_edge_by_key(&mut self, source: &K, target: &K, cost: E) -> EdgeStatus {
        let source = match self.map.get(source) {
            Some(i) => i.index(),
            None => return EdgeStatus::MissingSource,
        };
        let target = match self.map.get(target) {
            Some(i) => i.index(),
            None => return EdgeStatus::MissingTarget,
        };
        self.graph.add_edge(source, target, cost)
    }

    pub fn add_edge_by_index(&mut self, source: usize, target: usize, cost: E) -> EdgeStatus {
        self.graph.add_edge(source, target, cost)
    }

    pub fn get_node_by_key(&self, key: &K) -> Option<&Node<V>> {
        self.map.get(key)
    }

    pub fn get_node_by_index(&self, index: usize) -> Option<&Node<V>> {
        self.map.get(self.graph.get_node(index)?)
    }

    pub fn get_node_by_key_mut(&mut self, key: &K) -> Option<&mut Node<V>> {
        self.map.get_mut(key)
    }

    pub fn get_key_by_index(&self, index: usize) -> Option<&K> {
        Some(self.graph.get_node(index)?)
    }

    pub fn get_map(&self) -> &HashMap<K, Node<V>> {
        &self.map
    }

    pub fn get_map_mut(&mut self) -> &mut HashMap<K, Node<V>> {
        &mut self.map
    }

    pub fn find_root(&self) -> Result<&Node<V>, Vec<usize>> {
        match self.graph.find_root() {
            Ok(n) => Ok(self.map.get(self.graph.get_node(n).unwrap()).unwrap()),
            Err(e) => Err(e),
        }
    }

    pub fn get_graph(&self) -> &Graph<K, E> {
        &self.graph
    }

    pub fn iter(&self) -> IterGraphMap<K, V, E> {
        IterGraphMap {
            graph: &self,
            current_node_index: 0,
        }
    }
}

type NodeIndex = usize;
type EdgeIndex = usize;

pub struct IterGraphMap<'graph, K: Eq + Hash + Clone, V, E> {
    graph: &'graph GraphMap<K, V, E>,
    current_node_index: NodeIndex,
}

impl<'graph, K: Eq + Hash + Clone, V, E> Iterator for IterGraphMap<'graph, K, V, E> {
    type Item = (&'graph K, &'graph V, SuccessorsGraphMap<'graph, K, V, E>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_node_index >= self.graph.get_graph().node_count() {
            None
        } else {
            let key = self
                .graph
                .get_key_by_index(self.current_node_index)
                .unwrap();
            let value = &self.graph.get_node_by_key(&key).unwrap();

            self.current_node_index += 1;
            Some((key, value.as_ref(), self.graph.successors(value.index())))
        }
    }
}

impl<'graph, K: 'graph + Eq + Hash + Clone, V: 'graph, E: 'graph>
    FromIterator<(&'graph K, &'graph V, SuccessorsGraphMap<'graph, K, V, E>)>
    for GraphMap<&'graph K, &'graph V, &'graph E>
{
    fn from_iter<
        T: IntoIterator<Item = (&'graph K, &'graph V, SuccessorsGraphMap<'graph, K, V, E>)>,
    >(
        iter: T,
    ) -> Self {
        let mut graph: GraphMap<&K, &V, &E> = GraphMap::new();

        let mut iter = iter.into_iter();

        while let Some((key, value, mut outgoing_neighbors)) = iter.next() {
            // add missing node
            if graph.has_node_by_key(&key) == false {
                graph.add_node(key, value);
            }
            // add node's missing neighbors
            while let Some((n_key, n_value, edge)) = outgoing_neighbors.next() {
                if graph.has_node_by_key(&n_key) == false {
                    graph.add_node(n_key, n_value);
                }
                // add edge connection between node and neighbor
                graph.add_edge_by_key(&key, &n_key, edge);
            }
        }
        graph
    }
}

impl<'graph, K: 'graph + Eq + Hash + Clone, V: 'graph, E: 'graph>
    FromIterator<(&'graph K, &'graph V, PredecessorsGraphMap<'graph, K, V, E>)>
    for GraphMap<&'graph K, &'graph V, &'graph E>
{
    fn from_iter<
        T: IntoIterator<Item = (&'graph K, &'graph V, PredecessorsGraphMap<'graph, K, V, E>)>,
    >(
        iter: T,
    ) -> Self {
        let mut graph: GraphMap<&K, &V, &E> = GraphMap::new();

        let mut iter = iter.into_iter();

        while let Some((key, value, mut incoming_neighbors)) = iter.next() {
            // add missing node
            if graph.has_node_by_key(&key) == false {
                graph.add_node(key, value);
            }
            // add node's missing neighbors
            while let Some((n_key, n_value, edge)) = incoming_neighbors.next() {
                if graph.has_node_by_key(&n_key) == false {
                    graph.add_node(n_key, n_value);
                }
                // add edge connection between node and neighbor
                graph.add_edge_by_key(&n_key, &key, edge);
            }
        }
        graph
    }
}
