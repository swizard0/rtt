pub mod util;

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

pub trait TransClosestToSample<RT> {
    type RttClosestNodeToSample;
    type Error;

    fn closest_to_sample(self, rtt: RT) -> Result<Self::RttClosestNodeToSample, Self::Error>;
}

impl<RT, F, RNS, E> TransClosestToSample<RT> for F where F: FnOnce(RT) -> Result<RNS, E> {
    type RttClosestNodeToSample = RNS;
    type Error = E;

    fn closest_to_sample(self, rtt: RT) -> Result<Self::RttClosestNodeToSample, Self::Error> {
        (self)(rtt)
    }
}

impl<RT> PlannerReadyToSample<RT> {
    pub fn rtt(&self) -> &RT {
        &self.rtt
    }

    pub fn closest_to_sample<TR>(self, trans: TR) ->
        Result<PlannerNearestNodeFound<TR::RttClosestNodeToSample>, TR::Error>
        where TR: TransClosestToSample<RT>
    {
        Ok(PlannerNearestNodeFound {
            rtts_node: trans.closest_to_sample(self.rtt)?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton() {

        #[derive(PartialEq, Debug)]
        enum Step {
            EmptyTree,
            NodesCount(usize),
            SamplePrepared(usize),
            SampleNearest(usize),
            Path,
        }

        let planner = Planner::new(Step::EmptyTree);

        let mut planner_node = planner.add_root(|s| {
            assert_eq!(s, Step::EmptyTree);
            Ok::<_, ()>(Step::NodesCount(1))
        }).unwrap();

        loop {
            match planner_node.rtt_node() {
                &Step::NodesCount(10) =>
                    break,
                &Step::NodesCount(..) =>
                    (),
                other =>
                    panic!("Invalid state on limit step: {:?}", other),
            }

            let mut planner_sample = planner_node.prepare_sample(|s| {
                if let Step::NodesCount(count) = s {
                    Ok::<_, ()>(Step::SamplePrepared(count))
                } else {
                    panic!("Invalid state on prepare sample step: {:?}", s)
                }
            }).unwrap();

            loop {
                let next_sample = if let &Step::SamplePrepared(count) = planner_sample.rtt() {
                    count + 1
                } else {
                    panic!("Invalid state on sample picking step");
                };

                let planner_nearest = planner_sample.closest_to_sample(|_| {
                    Ok::<_, ()>(Step::SampleNearest(next_sample))
                }).unwrap();

                let value = if let &Step::SampleNearest(value) = planner_nearest.rtts_node() {
                    value
                } else {
                    panic!("Invalid on transition step: {:?}", planner_nearest.rtts_node())
                };

                if value % 2 != 0 {
                    planner_sample = planner_nearest.no_transition(|_| {
                        Ok::<_, ()>(Step::SamplePrepared(value))
                    }).unwrap();
                } else {
                    planner_node = planner_nearest.transition(|_| {
                        Ok::<_, ()>(Step::NodesCount(value))
                    }).unwrap();
                    break;
                }
            }
        }

        let path = planner_node.into_path(|s| {
            assert_eq!(s, Step::NodesCount(10));
            Ok::<_, ()>(Step::Path)
        }).unwrap();
        assert_eq!(path, Step::Path);
    }
}
