use super::seen_cache::SeenCache;

pub trait ContextManager {
    type State;
    type Error;
    type Dist: PartialOrd;

    fn metric_distance(&mut self, node: &Self::State, probe: &Self::State) ->
        Result<Self::Dist, Self::Error>;

    fn generate_trans<T, E>(&self, probe: Self::State, node_trans: T) ->
        Result<Option<Self::State>, CMError<Self::Error, E>>
        where T: TransChecker<Self::State, E>;

    fn generate_expand<E, EE>(&mut self, probe: Self::State, node_expander: E) ->
        Result<(), CMError<Self::Error, EE>>
        where E: Expander<Self::State, EE>
    {
        node_expander.expand(Some(Ok(probe)).into_iter())
            .map_err(CMError::RandomTreeProc)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum CMError<CME, RTPE> {
    ContextManager(CME),
    RandomTreeProc(RTPE),
}

pub trait Expander<S, E> {
    fn current(&self) -> &S;
    fn expand<I>(self, states: I) -> Result<(), E> where I: Iterator<Item = Result<S, E>>;
}

pub trait TransChecker<S, E> {
    fn current(&self) -> &S;
    fn already_visited(&self, state: &S) -> Result<bool, E>;
}

struct PathNode<S> {
    state: S,
    prev: Option<usize>,
}

pub struct RandomTree<CM, SC, S> {
    ctx_manager: CM,
    seen_cache: SC,
    nodes: Vec<PathNode<S>>,
}

impl<CM, SC, S> RandomTree<CM, SC, S>
    where CM: ContextManager<State = S>,
          SC: SeenCache<State = S>,
{
    pub fn new(ctx_manager: CM, seen_cache: SC) -> RandomTree<CM, SC, S> {
        RandomTree {
            ctx_manager,
            seen_cache,
            nodes: Vec::new(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Error<CME, SCE> {
    ContextManager(CME),
    SeenCache(SCE),
}

impl<CM, SC, S> super::super::RandomTree for RandomTree<CM, SC, S>
    where CM: ContextManager<State = S>,
          SC: SeenCache<State = S>,
{
    type State = S;
    type Error = Error<CM::Error, SC::Error>;
    type Node = RandomTreeNode<CM, SC, S>;

    fn make_root(mut self, state: Self::State) -> Result<Self::Node, Self::Error> {
        self.nodes.clear();
        self.nodes.push(PathNode { state, prev: None, });
        Ok(RandomTreeNode {
            ctx_manager: self.ctx_manager,
            seen_cache: self.seen_cache,
            nodes: self.nodes,
            node: 0,
        })
    }

    fn nearest_node(mut self, state: &Self::State) -> Result<Self::Node, Self::Error> {
        let distance_to_root = self.ctx_manager
            .metric_distance(&self.nodes[0].state, state)
            .map_err(Error::ContextManager)?;
        let mut nearest = (distance_to_root, 0);
        for index in 1 .. self.nodes.len() {
            let distance = self.ctx_manager
                .metric_distance(&self.nodes[index].state, state)
                .map_err(Error::ContextManager)?;
            if distance < nearest.0 {
                nearest = (distance, index);
            }
        }
        Ok(RandomTreeNode {
            ctx_manager: self.ctx_manager,
            seen_cache: self.seen_cache,
            nodes: self.nodes,
            node: nearest.1,
        })
    }
}

pub struct RandomTreeNode<CM, SC, S> {
    ctx_manager: CM,
    seen_cache: SC,
    nodes: Vec<PathNode<S>>,
    node: usize,
}

impl<CM, SC, S> RandomTreeNode<CM, SC, S> {
    pub fn state(&self) -> &S {
        &self.nodes[self.node].state
    }
}

impl<CM, SC, S> super::super::RandomTreeNode for RandomTreeNode<CM, SC, S>
    where CM: ContextManager<State = S>,
          SC: SeenCache<State = S>,
{
    type State = S;
    type Error = Error<CM::Error, SC::Error>;
    type Tree = RandomTree<CM, SC, S>;
    type Path = RevPathIterator<S>;

    fn expand(mut self, state: Self::State) -> Result<Self, Self::Error> {
        struct NodesExpander<'a, SC: 'a, S: 'a> {
            seen_cache: &'a mut SC,
            nodes: &'a mut Vec<PathNode<S>>,
            node: &'a mut usize,
        }

        impl<'a, SC, S, E> Expander<S, E> for NodesExpander<'a, SC, S>
            where SC: SeenCache<State = S, Error = E>
        {
            fn current(&self) -> &S {
                &self.nodes[*self.node].state
            }

            fn expand<I>(self, states: I) -> Result<(), E>
                where I: Iterator<Item = Result<S, E>>
            {
                for maybe_state in states {
                    let state = maybe_state?;
                    self.seen_cache.remember(&state)?;
                    let next_index = self.nodes.len();
                    self.nodes.push(PathNode { state, prev: Some(*self.node), });
                    *self.node = next_index;
                }
                Ok(())
            }
        }

        let result = self.ctx_manager
            .generate_expand(state, NodesExpander {
                seen_cache: &mut self.seen_cache,
                nodes: &mut self.nodes,
                node: &mut self.node,
            });
        match result {
            Ok(()) =>
                Ok(self),
            Err(CMError::ContextManager(err)) =>
                Err(Error::ContextManager(err)),
            Err(CMError::RandomTreeProc(err)) =>
                Err(Error::SeenCache(err)),
        }
    }

    fn transition(&self, random_state: Self::State) -> Result<Option<Self::State>, Self::Error> {
        struct NodesChecker<'a, SC: 'a, S: 'a> {
            seen_cache: &'a SC,
            nodes: &'a Vec<PathNode<S>>,
            node: usize,
        }

        impl<'a, SC, S, E> TransChecker<S, E> for NodesChecker<'a, SC, S>
            where SC: SeenCache<State = S, Error = E>
        {
            fn current(&self) -> &S {
                &self.nodes[self.node].state
            }

            fn already_visited(&self, state: &S) -> Result<bool, E> {
                let iter = self.nodes
                    .iter()
                    .map(|n| Ok(&n.state));
                self.seen_cache.already_seen(state, iter)
            }
        }

        let result = self.ctx_manager
            .generate_trans(random_state, NodesChecker {
                seen_cache: &self.seen_cache,
                nodes: &self.nodes,
                node: self.node,
            });
        match result {
            Ok(value) =>
                Ok(value),
            Err(CMError::ContextManager(err)) =>
                Err(Error::ContextManager(err)),
            Err(CMError::RandomTreeProc(err)) =>
                Err(Error::SeenCache(err)),
        }
    }

    fn into_tree(self) -> Self::Tree {
        RandomTree {
            ctx_manager: self.ctx_manager,
            seen_cache: self.seen_cache,
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
