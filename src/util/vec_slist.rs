pub trait NearestNodeLocator<S> {
    type Error;

    fn locate_nearest<'a, SI, I>(&mut self, state: &S, rtt_root: SI, rtt_states: I) ->
        Result<&'a SI, Self::Error>
        where I: Iterator<Item = SI>, SI: AsRef<S>;
}

struct PathNode<S> {
    state: S,
    prev: Option<usize>,
}

pub struct RandomTree<S, NNL> {
    locator: NNL,
    nodes: Vec<PathNode<S>>,
}

impl<S, NLL> RandomTree<S, NLL> {
    pub fn new(locator: NLL) -> RandomTree<S, NLL> {
        RandomTree {
            locator,
            nodes: Vec::new(),
        }
    }
}

impl<S, NLL> super::super::RandomTree for RandomTree<S, NLL> where NLL: NearestNodeLocator<S> {
    type State = S;
    type Error = NLL::Error;
    type Node = RandomTreeNode<S, NLL>;

    fn add_root(mut self, state: Self::State) -> Result<Self::Node, Self::Error> {
        self.nodes.clear();
        self.nodes.push(PathNode { state, prev: None, });
        Ok(RandomTreeNode {
            locator: self.locator,
            nodes: self.nodes,
            node: 0,
        })
    }
}

pub struct NonEmptyRandomTree<S, NLL> {
    locator: NLL,
    nodes: Vec<PathNode<S>>,
}

impl<S, NLL> super::super::NonEmptyRandomTree for NonEmptyRandomTree<S, NLL> where NLL: NearestNodeLocator<S> {
    type State = S;
    type Error = NLL::Error;
    type Node = RandomTreeNode<S, NLL>;

    fn nearest_node(mut self, state: &Self::State) -> Result<Self::Node, Self::Error> {
        let nearest_node_index = {
            struct Indexed<'a, S: 'a> {
                state: &'a S,
                index: usize,
            }

            impl<'a, S> AsRef<S> for Indexed<'a, S> {
                fn as_ref(&self) -> &S {
                    self.state
                }
            }

            self.locator.locate_nearest(
                state,
                Indexed { state: &self.nodes[0].state, index: 0, },
                self.nodes[1 ..].iter()
                    .enumerate()
                    .map(|(index, n)| Indexed { state: &n.state, index, }),
            )?.index
        };

        Ok(RandomTreeNode {
            locator: self.locator,
            nodes: self.nodes,
            node: nearest_node_index,
        })
    }
}

pub struct RandomTreeNode<S, NLL> {
    locator: NLL,
    nodes: Vec<PathNode<S>>,
    node: usize,
}

impl<S, NLL> RandomTreeNode<S, NLL> {
    pub fn state(&self) -> &S {
        &self.nodes[self.node].state
    }
}

impl<S, NLL> super::super::RandomTreeNode for RandomTreeNode<S, NLL> where NLL: NearestNodeLocator<S> {
    type State = S;
    type Error = NLL::Error;
    type Tree = NonEmptyRandomTree<S, NLL>;
    type Path = RevPathIterator<S>;

    fn expand(mut self, state: Self::State) -> Result<Self, Self::Error> {
        let next_index = self.nodes.len();
        self.nodes.push(PathNode { state, prev: Some(self.node), });
        self.node = next_index;
        Ok(self)
    }

    fn into_tree(self) -> Self::Tree {
        NonEmptyRandomTree {
            locator: self.locator,
            nodes: self.nodes,
        }
    }

    fn into_path(self) -> Self::Path {
        RevPathIterator {
            nodes: self.nodes,
            node: Some(self.node),
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
