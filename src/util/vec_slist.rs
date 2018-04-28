pub trait ContextManager {
    type State;
    type Error;
    type Dist: PartialOrd;

    fn metric_distance(&mut self, node: &Self::State, probe: &Self::State) ->
        Result<Self::Dist, Self::Error>;
    fn states_eq(&self, state_a: &Self::State, state_b: &Self::State) -> bool;

    fn generate_expand<E>(&mut self, probe: Self::State, node_expander: E) ->
        Result<(), Self::Error>
        where E: Expander<Self::State, Self::Error>
    {
        node_expander.expand(Some(Ok(probe)).into_iter())
    }
}

pub trait Expander<S, E> {
    fn current(&self) -> &S;
    fn expand<I>(self, states: I) -> Result<(), E> where I: Iterator<Item = Result<S, E>>;
}

struct PathNode<S> {
    state: S,
    prev: Option<usize>,
}

pub struct RandomTree<CM, S> {
    ctx_manager: CM,
    nodes: Vec<PathNode<S>>,
}

impl<CM, S> RandomTree<CM, S> where CM: ContextManager<State = S> {
    pub fn new(ctx_manager: CM) -> RandomTree<CM, S> {
        RandomTree {
            ctx_manager,
            nodes: Vec::new(),
        }
    }
}

impl<CM, S> super::super::RandomTree for RandomTree<CM, S> where CM: ContextManager<State = S> {
    type State = S;
    type Error = CM::Error;
    type Node = RandomTreeNode<CM, S>;

    fn make_root(mut self, state: Self::State) -> Result<Self::Node, Self::Error> {
        self.nodes.clear();
        self.nodes.push(PathNode { state, prev: None, });
        Ok(RandomTreeNode {
            ctx_manager: self.ctx_manager,
            nodes: self.nodes,
            node: 0,
        })
    }

    fn nearest_node(mut self, state: &Self::State) -> Result<Self::Node, Self::Error> {
        let distance_to_root =
            self.ctx_manager.metric_distance(&self.nodes[0].state, state)?;
        let mut nearest = (distance_to_root, 0);
        for index in 1 .. self.nodes.len() {
            let distance =
                self.ctx_manager.metric_distance(&self.nodes[index].state, state)?;
            if distance < nearest.0 {
                nearest = (distance, index);
            }
        }
        Ok(RandomTreeNode {
            ctx_manager: self.ctx_manager,
            nodes: self.nodes,
            node: nearest.1,
        })
    }
}

pub struct RandomTreeNode<CM, S> {
    ctx_manager: CM,
    nodes: Vec<PathNode<S>>,
    node: usize,
}

impl<CM, S> RandomTreeNode<CM, S> {
    pub fn state(&self) -> &S {
        &self.nodes[self.node].state
    }
}

impl<CM, S> super::super::RandomTreeNode for RandomTreeNode<CM, S> where CM: ContextManager<State = S> {
    type State = S;
    type Error = CM::Error;
    type Tree = RandomTree<CM, S>;
    type Path = Vec<S>;

    fn expand(mut self, state: Self::State) -> Result<Self, Self::Error> {
        struct NodesExpander<'a, S: 'a> {
            nodes: &'a mut Vec<PathNode<S>>,
            node: &'a mut usize,
        }

        impl<'a, S, E> Expander<S, E> for NodesExpander<'a, S> {
            fn current(&self) -> &S {
                &self.nodes[*self.node].state
            }

            fn expand<I>(self, states: I) -> Result<(), E>
                where I: Iterator<Item = Result<S, E>>
            {
                for maybe_state in states {
                    let state = maybe_state?;
                    let next_index = self.nodes.len();
                    self.nodes.push(PathNode { state, prev: Some(*self.node), });
                    *self.node = next_index;
                }
                Ok(())
            }
        }

        self.ctx_manager
            .generate_expand(state, NodesExpander { nodes: &mut self.nodes, node: &mut self.node, })?;
        Ok(self)
    }

    fn transition(&self, random_state: Self::State) -> Result<Option<Self::State>, Self::Error> {
        if self.nodes.iter().any(|n| self.ctx_manager.states_eq(&n.state, &random_state)) {
            return Ok(None);
        }


        unimplemented!()
    }

    fn into_tree(self) -> Self::Tree {
        RandomTree {
            ctx_manager: self.ctx_manager,
            nodes: self.nodes,
        }
    }

    fn into_path(mut self) -> Self::Path {
        let mut path = Vec::new();
        let mut maybe_index = Some(self.node);
        while let Some(node_index) = maybe_index {
            let node = self.nodes.swap_remove(node_index);
            path.push(node.state);
            maybe_index = node.prev;
        }
        path.reverse();
        path
    }
}
