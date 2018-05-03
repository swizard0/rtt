use std::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NodeRef(usize);

struct PathNode<S> {
    state: S,
    prev: Option<usize>,
}

pub struct EmptyRandomTree<S> {
    _marker: PhantomData<S>,
}

impl<S> EmptyRandomTree<S> {
    pub fn new() -> EmptyRandomTree<S> {
        EmptyRandomTree {
            _marker: PhantomData,
        }
    }

    pub fn add_root(self, state: S) -> RandomTree<S> {
        RandomTree {
            nodes: vec![PathNode { state, prev: None, }],
        }
    }
}

pub struct RandomTree<S> {
    nodes: Vec<PathNode<S>>,
}

impl<S> RandomTree<S> {
    pub fn root(&self) -> NodeRef {
        NodeRef(0)
    }

    pub fn expand(&mut self, NodeRef(node_index): NodeRef, state: S) -> NodeRef {
        let next_index = self.nodes.len();
        self.nodes.push(PathNode { state, prev: Some(node_index), });
        NodeRef(next_index)
    }

    pub fn into_path(self, NodeRef(node_index): NodeRef) -> RevPathIterator<S> {
        RevPathIterator {
            nodes: self.nodes,
            node: Some(node_index),
        }
    }

    pub fn get_state(&self, NodeRef(node_index): NodeRef) -> &S {
        &self.nodes[node_index].state
    }

    pub fn states(&self) -> RandomTreeStatesIter<S> {
        RandomTreeStatesIter {
            nodes: &self.nodes,
            index: 0,
        }
    }
}

pub struct RandomTreeStatesIter<'a, S: 'a> {
    nodes: &'a [PathNode<S>],
    index: usize,
}

impl<'a, S> Iterator for RandomTreeStatesIter<'a, S> {
    type Item = (NodeRef, &'a S);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.nodes.len() {
            None
        } else {
            let item = (
                NodeRef(self.index),
                &self.nodes[self.index].state
            );
            self.index += 1;
            Some(item)
        }
    }
}

pub struct RevPathIterator<S> {
    nodes: Vec<PathNode<S>>,
    node: Option<usize>,
}

impl<S> Iterator for RevPathIterator<S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node_index) = self.node {
            let node = self.nodes.swap_remove(node_index);
            self.node = node.prev;
            Some(node.state)
        } else {
            None
        }
    }
}
