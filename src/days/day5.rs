use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    ops::{Range, RangeTo},
};

use itertools::multiunzip;
use nom::{
    character::complete::{newline, u32},
    combinator::eof,
    complete::take,
    sequence::{separated_pair, tuple},
    IResult, Parser,
};
use nom_supreme::{
    error::ErrorTree, multi::collect_separated_terminated, tag::complete::tag, ParserExt,
};

use crate::util::prelude::*;

#[derive(Debug, Clone)]
struct DirectedGraph<N> {
    adjacency: Vec<usize>,
    adjacency_offsets: Vec<usize>,
    nodes: Vec<N>,
}

impl<K> Default for DirectedGraph<K> {
    fn default() -> Self {
        Self {
            adjacency: Default::default(),
            adjacency_offsets: Default::default(),
            nodes: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct DirectedGraphBuilder<N> {
    edges: Vec<Vec<usize>>,
    node_positions: HashMap<N, usize>,
}

impl<N> Default for DirectedGraphBuilder<N> {
    fn default() -> Self {
        Self {
            edges: Default::default(),
            node_positions: Default::default(),
        }
    }
}

impl<N> DirectedGraph<N>
where
    N: Hash + Eq,
{
    fn into_builder(mut self) -> DirectedGraphBuilder<N> {
        DirectedGraphBuilder {
            node_positions: std::mem::take(&mut self.nodes)
                .into_iter()
                .enumerate()
                .map(|(i, n)| (n, i))
                .collect(),
            edges: {
                let mut edges_rev = self
                    .adjacency_offsets
                    .iter()
                    .rev()
                    .map(|ix| self.adjacency.split_off(*ix))
                    .collect_vec();
                edges_rev.reverse();
                edges_rev
            },
        }
    }
}

impl<N> DirectedGraphBuilder<N> {
    fn build(self) -> DirectedGraph<N> {
        let mut running_offset: usize = 0;
        DirectedGraph {
            adjacency_offsets: self
                .edges
                .iter()
                .map(|children| {
                    let old_offset = running_offset;
                    running_offset += children.len();
                    old_offset
                })
                .collect(),
            adjacency: self.edges.into_iter().flatten().collect(),

            nodes: self
                .node_positions
                .into_iter()
                .sorted_by_key(|(_, i)| *i)
                .map(|(node, _)| node)
                .collect(),
        }
    }
}

impl<N> Extend<(N, N)> for DirectedGraphBuilder<N>
where
    N: Hash + Eq,
{
    fn extend<T: IntoIterator<Item = (N, N)>>(&mut self, iter: T) {
        for (start, end) in iter {
            let mut get_position = |node: N| {
                use std::collections::hash_map::Entry::*;
                match self.node_positions.entry(node) {
                    Occupied(occupied_entry) => *occupied_entry.get(),
                    Vacant(vacant_entry) => {
                        self.edges.push(Vec::new());
                        *vacant_entry.insert(self.edges.len() - 1)
                    }
                }
            };
            let ix_start = get_position(start);
            let ix_end = get_position(end);
            self.edges[ix_start].push(ix_end);
        }
    }
}

impl<N> DirectedGraph<N> {
    fn children(&self, node_ix: usize) -> &[usize] {
        let start = self.adjacency_offsets[node_ix];
        let end = self
            .adjacency_offsets
            .get(node_ix + 1)
            .copied()
            .unwrap_or(self.adjacency.len());
        &self.adjacency[start..end]
    }

    pub fn topological_sort(
        &self,
    ) -> Result<impl Iterator<Item = &N> + Debug, impl Iterator<Item = &N> + Debug> {
        #[derive(Debug, Clone, Copy)]
        enum VisitState {
            Unseen,
            InProgress,
            Visited,
        }

        #[derive(Debug)]
        enum NodeIter<'a> {
            AllNodes(Range<usize>),
            Children {
                node_ix: usize,
                children: std::slice::Iter<'a, usize>,
            },
        }

        impl Iterator for NodeIter<'_> {
            type Item = usize;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::AllNodes(range) => range.next(),
                    Self::Children { children, .. } => children.next().copied(),
                }
            }
        }

        impl NodeIter<'_> {
            fn node_ix(&self) -> Option<usize> {
                match self {
                    Self::Children { node_ix, .. } => Some(*node_ix),
                    _ => None,
                }
            }
        }

        let mut state = vec![VisitState::Unseen; self.nodes.len()];
        let mut stack = vec![NodeIter::AllNodes(0..self.nodes.len())];
        let mut reverse_order_indices: Vec<usize> = Vec::new();

        while let Some(node_iter) = stack.last_mut() {
            match node_iter.next() {
                Some(child_ix) => match state[child_ix] {
                    VisitState::Unseen => {
                        state[child_ix] = VisitState::InProgress;
                        stack.push(NodeIter::Children {
                            node_ix: child_ix,
                            children: self.children(child_ix).iter(),
                        });
                    }
                    VisitState::InProgress => {
                        return Err(stack
                            .into_iter()
                            .flat_map(|node_iter| node_iter.node_ix())
                            .skip_while(move |ix| *ix != child_ix)
                            .map(|i| &self.nodes[i]))
                    }
                    VisitState::Visited => {}
                },
                None => {
                    if let Some(node_ix) = node_iter.node_ix() {
                        state[node_ix] = VisitState::Visited;
                        reverse_order_indices.push(node_ix);
                    }
                    stack.pop();
                }
            }
        }
        Ok(reverse_order_indices
            .into_iter()
            .rev()
            .map(|ix| &self.nodes[ix]))
    }
}

impl<N> DirectedGraphBuilder<N>
where
    N: Eq + Hash,
{
    fn subgraph(&self, nodes: impl IntoIterator<Item = N>) -> DirectedGraphBuilder<N> {
        let (node_positions, mut edges, old_to_new): (_, Vec<_>, HashMap<_, _>) =
            multiunzip(nodes.into_iter().enumerate().map(|(i, node)| {
                let node_ix = *self.node_positions.get(&node).expect("Node not in graph");
                let edges = self.edges[node_ix].clone();

                ((node, i), edges, (node_ix, i))
            }));
        for node_edges in &mut edges {
            *node_edges = node_edges
                .iter()
                .flat_map(|old_ix| old_to_new.get(old_ix).copied())
                .collect();
        }

        DirectedGraphBuilder {
            node_positions,
            edges,
        }
    }
}

impl<N> DirectedGraph<N>
where
    for<'a> &'a N: PartialEq,
{
    fn is_sub_topological_order(&self, nodes: impl Iterator<Item = N>) -> bool {
        let mut visited = vec![false; self.nodes.len()];
        for node in nodes {
            let Some(node_ix) = self.nodes.iter().position(|n| n == &node) else {
                return false;
            };
            if self
                .children(node_ix)
                .iter()
                .any(|child_ix| visited[*child_ix])
            {
                return false;
            }
            visited[node_ix] = true;
        }
        true
    }
}

impl<N> FromIterator<(N, N)> for DirectedGraph<N>
where
    N: Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = (N, N)>>(iter: T) -> Self {
        let mut result = DirectedGraphBuilder::default();
        result.extend(iter);
        result.build()
    }
}

fn parse_directed_graph(input: &str) -> IResult<&str, DirectedGraphBuilder<u32>, ErrorTree<&str>> {
    collect_separated_terminated(
        separated_pair(u32, tag("|"), u32).context("edge"),
        newline,
        newline.precedes(newline),
    )
    .context("graph")
    .parse(input)
}

fn parse_input(input: &str) -> Result<(DirectedGraphBuilder<u32>, Vec<Vec<u32>>), ErrorTree<&str>> {
    nom_supreme::final_parser::final_parser(tuple((
        parse_directed_graph,
        collect_separated_terminated(
            collect_separated_terminated(u32, tag(","), newline).context("nodes list"),
            tag(""),
            eof,
        )
        .context("test cases"),
    )))(input)
}

pub fn level1(input: &str) -> u32 {
    let (builder, orders) = parse_input(input).expect("parse");
    orders
        .into_iter()
        .filter(|order| {
            let graph = builder.subgraph(order.clone()).build();
            graph.is_sub_topological_order(order.iter().copied())
        })
        .map(|order| order[order.len() / 2])
        .sum()
}

pub fn level2(input: &str) -> u32 {
    let (builder, orders) = parse_input(input).expect("parse");
    orders
        .into_iter()
        .flat_map(|order| {
            let graph = builder.subgraph(order.clone()).build();
            if graph.is_sub_topological_order(order.iter().copied()) {
                None
            } else {
                Some(
                    graph
                        .topological_sort()
                        .expect("subgraph is DAG")
                        .copied()
                        .collect_vec(),
                )
            }
        })
        .map(|order| order[order.len() / 2])
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day5.txt");
        assert_eq!(level1(test_input), 143)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day5.txt");
        assert_eq!(level2(test_input), 123)
    }
}
