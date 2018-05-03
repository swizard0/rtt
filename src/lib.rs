// pub mod util;

// pub trait RandomTree {
//     type State;
//     type Error;
//     type Node: RandomTreeNode<State = Self::State, Error = Self::Error>;

//     fn add_root(self, state: Self::State) -> Result<Self::Node, Self::Error>;
// }

// pub trait NonEmptyRandomTree {
//     type State;
//     type Error;
//     type Node: RandomTreeNode<State = Self::State, Error = Self::Error>;

//     fn nearest_node(self, state: &Self::State) -> Result<Self::Node, Self::Error>;
// }

// pub trait RandomTreeNode: Sized {
//     type State;
//     type Error;
//     type Tree: NonEmptyRandomTree<State = Self::State, Error = Self::Error>;
//     type Path;

//     fn expand(self, state: Self::State) -> Result<Self, Self::Error>;
//     fn into_tree(self) -> Self::Tree;
//     fn into_path(self) -> Self::Path;
// }


// -----

// Planner

pub struct Planner<RT> {
    rtt: RT,
}

impl<RT> Planner<RT> {
    pub fn new(rtt: RT) -> Planner<RT> {
        Planner { rtt, }
    }
}

pub trait TransAddRoot<RT> {
    type RttNodeFocus;
    type Error;

    fn add_root(self, rtt: RT) -> Result<Self::RttNodeFocus, Self::Error>;
}

impl<RT, F, NF, E> TransAddRoot<RT> for F where F: FnOnce(RT) -> Result<NF, E> {
    type RttNodeFocus = NF;
    type Error = E;

    fn add_root(self, rtt: RT) -> Result<Self::RttNodeFocus, Self::Error> {
        (self)(rtt)
    }
}

impl<RT> Planner<RT> {
    pub fn add_root<TR>(self, trans: TR) ->
        Result<PlannerNodeExpanded<TR::RttNodeFocus>, TR::Error>
        where TR: TransAddRoot<RT>
    {
        Ok(PlannerNodeExpanded {
            rtt_node: trans.add_root(self.rtt)?,
        })
    }
}

// PlannerNodeExpanded

pub struct PlannerNodeExpanded<RN> {
    rtt_node: RN,
}

pub trait TransIntoPath<RN> {
    type RttPath;
    type Error;

    fn into_path(self, rtt_node: RN) -> Result<Self::RttPath, Self::Error>;
}

impl<RN, F, P, E> TransIntoPath<RN> for F where F: FnOnce(RN) -> Result<P, E> {
    type RttPath = P;
    type Error = E;

    fn into_path(self, rtt_node: RN) -> Result<Self::RttPath, Self::Error> {
        (self)(rtt_node)
    }
}

pub trait TransPrepareSample<RN> {
    type Rtt;
    type Error;

    fn prepare_sample(self, rtt_node: RN) -> Result<Self::Rtt, Self::Error>;
}

impl<RN, F, RT, E> TransPrepareSample<RN> for F where F: FnOnce(RN) -> Result<RT, E> {
    type Rtt = RT;
    type Error = E;

    fn prepare_sample(self, rtt_node: RN) -> Result<Self::Rtt, Self::Error> {
        (self)(rtt_node)
    }
}

impl<RN> PlannerNodeExpanded<RN> {
    pub fn rtt_node(&self) -> &RN {
        &self.rtt_node
    }

    pub fn into_path<TR>(self, trans: TR) -> Result<TR::RttPath, TR::Error>
        where TR: TransIntoPath<RN>
    {
        trans.into_path(self.rtt_node)
    }

    pub fn prepare_sample<TR>(self, trans: TR) -> Result<PlannerReadyToSample<TR::Rtt>, TR::Error>
        where TR: TransPrepareSample<RN>
    {
        Ok(PlannerReadyToSample {
            rtt: trans.prepare_sample(self.rtt_node)?,
        })
    }
}

// PlannerReadyToSample

pub struct PlannerReadyToSample<RT> {
    rtt: RT,
}

pub trait TransSample<RT> {
    type RttWithSample;
    type Error;

    fn sample(self, rtt: RT) -> Result<Self::RttWithSample, Self::Error>;
}

impl<RT, F, TS, E> TransSample<RT> for F where F: FnOnce(RT) -> Result<TS, E> {
    type RttWithSample = TS;
    type Error = E;

    fn sample(self, rtt: RT) -> Result<Self::RttWithSample, Self::Error> {
        (self)(rtt)
    }
}

impl<RT> PlannerReadyToSample<RT> {
    pub fn rtt(&self) -> &RT {
        &self.rtt
    }

    pub fn sample<TR>(self, trans: TR) ->
        Result<PlannerSamplePicked<TR::RttWithSample>, TR::Error>
        where TR: TransSample<RT>
    {
        Ok(PlannerSamplePicked {
            rtts: trans.sample(self.rtt)?,
        })
    }
}

// PlannerSamplePicked

pub struct PlannerSamplePicked<RTS> {
    rtts: RTS,
}

pub trait TransNearestNode<RTS> {
    type RttNodeFocusWithSample;
    type Error;

    fn nearest_node(self, rtts: RTS) -> Result<Self::RttNodeFocusWithSample, Self::Error>;
}

impl<RTS, F, RNS, E> TransNearestNode<RTS> for F where F: FnOnce(RTS) -> Result<RNS, E> {
    type RttNodeFocusWithSample = RNS;
    type Error = E;

    fn nearest_node(self, rtts: RTS) -> Result<Self::RttNodeFocusWithSample, Self::Error> {
        (self)(rtts)
    }
}

impl<RTS> PlannerSamplePicked<RTS> {
    pub fn rtts(&self) -> &RTS {
        &self.rtts
    }

    pub fn nearest_node<TR>(self, trans: TR) ->
        Result<PlannerNearestNodeFound<TR::RttNodeFocusWithSample>, TR::Error>
        where TR: TransNearestNode<RTS>
    {
        Ok(PlannerNearestNodeFound {
            rtts_node: trans.nearest_node(self.rtts)?,
        })
    }
}

// PlannerNearestNodeFound

pub struct PlannerNearestNodeFound<RNS> {
    rtts_node: RNS,
}

pub trait TransNoTransition<RNS> {
    type Rtt;
    type Error;

    fn no_transition(self, rtts_node: RNS) -> Result<Self::Rtt, Self::Error>;
}

impl<RNS, F, RT, E> TransNoTransition<RNS> for F where F: FnOnce(RNS) -> Result<RT, E> {
    type Rtt = RT;
    type Error = E;

    fn no_transition(self, rtts_node: RNS) -> Result<Self::Rtt, Self::Error> {
        (self)(rtts_node)
    }
}

pub trait TransTransition<RNS> {
    type RttNodeFocus;
    type Error;

    fn transition(self, rtts_node: RNS) -> Result<Self::RttNodeFocus, Self::Error>;
}

impl<RNS, F, RN, E> TransTransition<RNS> for F where F: FnOnce(RNS) -> Result<RN, E> {
    type RttNodeFocus = RN;
    type Error = E;

    fn transition(self, rtts_node: RNS) -> Result<Self::RttNodeFocus, Self::Error> {
        (self)(rtts_node)
    }
}

impl<RNS> PlannerNearestNodeFound<RNS> {
    pub fn rtts_node(&self) -> &RNS {
        &self.rtts_node
    }

    pub fn no_transition<TR>(self, trans: TR) -> Result<PlannerReadyToSample<TR::Rtt>, TR::Error>
        where TR: TransNoTransition<RNS>
    {
        Ok(PlannerReadyToSample {
            rtt: trans.no_transition(self.rtts_node)?,
        })
    }

    pub fn transition<TR>(self, trans: TR) -> Result<PlannerNodeExpanded<TR::RttNodeFocus>, TR::Error>
        where TR: TransTransition<RNS>
    {
        Ok(PlannerNodeExpanded {
            rtt_node: trans.transition(self.rtts_node)?,
        })
    }
}


// -----

// impl<RN, S> PlannerNearestNodeFound<RN, S> where RN: RandomTreeNode<State = S> {
//     pub fn no_transition(self) -> PlannerReadyToSample<RN::Tree> {
//         PlannerReadyToSample {
//             rtt: self.rtt_node.into_tree(),
//         }
//     }

//     pub fn start_transition(self) -> PlannerTransStateWait<RN, RN::State> {
//         PlannerTransStateWait {
//             rtt_node: self.rtt_node,
//             final_state: self.sample_state,
//         }
//     }
// }

// pub struct PlannerTransStateWait<RN, S> {
//     rtt_node: RN,
//     final_state: S,
// }

// impl<RN, S> PlannerTransStateWait<RN, S> {
//     pub fn rtt_node(&self) -> &RN {
//         &self.rtt_node
//     }

//     pub fn final_state(&self) -> &S {
//         &self.final_state
//     }
// }

// impl<RN, S> PlannerTransStateWait<RN, S> where RN: RandomTreeNode<State = S> {
//     pub fn finish(self) -> Result<PlannerNodeExpanded<RN>, RN::Error> {
//         Ok(PlannerNodeExpanded {
//             rtt_node: self.rtt_node.expand(self.final_state)?,
//         })
//     }

//     pub fn intermediate_trans(self, trans_state: RN::State) -> Result<PlannerTransStateWait<RN, S>, RN::Error> {
//         Ok(PlannerTransStateWait {
//             rtt_node: self.rtt_node.expand(trans_state)?,
//             final_state: self.final_state,
//         })
//     }
// }
