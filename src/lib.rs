pub mod util;

pub trait RandomTree {
    type State;
    type Error;
    type Node: RandomTreeNode<State = Self::State, Error = Self::Error>;

    fn add_root(self, state: Self::State) -> Result<Self::Node, Self::Error>;
}

pub trait NonEmptyRandomTree {
    type State;
    type Error;
    type Node: RandomTreeNode<State = Self::State, Error = Self::Error>;

    fn nearest_node(self, state: &Self::State) -> Result<Self::Node, Self::Error>;
}

pub trait RandomTreeNode: Sized {
    type State;
    type Error;
    type Tree: NonEmptyRandomTree<State = Self::State, Error = Self::Error>;
    type Path;

    fn expand(self, state: Self::State) -> Result<Self, Self::Error>;
    fn into_tree(self) -> Self::Tree;
    fn into_path(self) -> Self::Path;
}

pub struct Planner<RT> {
    rtt: RT,
}

impl<RT> Planner<RT> {
    pub fn new(rtt: RT) -> Planner<RT> {
        Planner { rtt, }
    }
}

impl<RT> Planner<RT> where RT: RandomTree {
    pub fn init(self, init_state: RT::State) -> Result<PlannerNodeExpanded<RT::Node>, RT::Error> {
        let rtt_node = self.rtt.add_root(init_state)?;
        Ok(PlannerNodeExpanded { rtt_node, })
    }
}

pub struct PlannerNodeExpanded<RN> {
    rtt_node: RN,
}

impl<RN> PlannerNodeExpanded<RN> {
    pub fn rtt_node(&self) -> &RN {
        &self.rtt_node
    }
}

impl<RN> PlannerNodeExpanded<RN> where RN: RandomTreeNode {
    pub fn prepare_sample(self) -> PlannerReadyToSample<RN::Tree> {
        PlannerReadyToSample {
            rtt: self.rtt_node.into_tree(),
        }
    }

    pub fn into_path(self) -> RN::Path {
        self.rtt_node.into_path()
    }
}

pub struct PlannerReadyToSample<RT> {
    rtt: RT,
}

impl<RT> PlannerReadyToSample<RT> {
    pub fn rtt(&self) -> &RT {
        &self.rtt
    }
}

impl<RT> PlannerReadyToSample<RT> where RT: NonEmptyRandomTree {
    pub fn sample(self, sample_state: RT::State) ->
        Result<PlannerNearestNodeFound<RT::Node, RT::State>, RT::Error>
    {
        let rtt_node = self.rtt.nearest_node(&sample_state)?;
        Ok(PlannerNearestNodeFound { rtt_node, sample_state, })
    }
}

pub struct PlannerNearestNodeFound<RN, S> {
    rtt_node: RN,
    sample_state: S,
}

impl<RN, S> PlannerNearestNodeFound<RN, S> {
    pub fn rtt_node(&self) -> &RN {
        &self.rtt_node
    }

    pub fn sample_state(&self) -> &S {
        &self.sample_state
    }
}

impl<RN, S> PlannerNearestNodeFound<RN, S> where RN: RandomTreeNode<State = S> {
    pub fn no_transition(self) -> PlannerReadyToSample<RN::Tree> {
        PlannerReadyToSample {
            rtt: self.rtt_node.into_tree(),
        }
    }

    pub fn start_transition(self) -> PlannerTransStateWait<RN, RN::State> {
        PlannerTransStateWait {
            rtt_node: self.rtt_node,
            final_state: self.sample_state,
        }
    }
}

pub struct PlannerTransStateWait<RN, S> {
    rtt_node: RN,
    final_state: S,
}

impl<RN, S> PlannerTransStateWait<RN, S> {
    pub fn rtt_node(&self) -> &RN {
        &self.rtt_node
    }

    pub fn final_state(&self) -> &S {
        &self.final_state
    }
}

impl<RN, S> PlannerTransStateWait<RN, S> where RN: RandomTreeNode<State = S> {
    pub fn finish(self) -> Result<PlannerNodeExpanded<RN>, RN::Error> {
        Ok(PlannerNodeExpanded {
            rtt_node: self.rtt_node.expand(self.final_state)?,
        })
    }

    pub fn intermediate_trans(self, trans_state: RN::State) -> Result<PlannerTransStateWait<RN, S>, RN::Error> {
        Ok(PlannerTransStateWait {
            rtt_node: self.rtt_node.expand(trans_state)?,
            final_state: self.final_state,
        })
    }
}
