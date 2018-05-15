pub mod util;

// PlannerInit

pub struct PlannerInit<ERT> {
    empty_rtt: ERT,
}

impl<ERT> PlannerInit<ERT> {
    pub fn new(empty_rtt: ERT) -> PlannerInit<ERT> {
        PlannerInit { empty_rtt, }
    }
}

pub trait TransAddRoot<ERT> {
    type NonEmptyRtt;
    type Error;

    fn add_root(self, empty_rtt: ERT) -> Result<Self::NonEmptyRtt, Self::Error>;
}

impl<ERT, F, RT, E> TransAddRoot<ERT> for F where F: FnOnce(ERT) -> Result<RT, E> {
    type NonEmptyRtt = RT;
    type Error = E;

    fn add_root(self, empty_rtt: ERT) -> Result<Self::NonEmptyRtt, Self::Error> {
        (self)(empty_rtt)
    }
}

impl<ERT> PlannerInit<ERT> {
    pub fn add_root<TR>(self, trans: TR) ->
        Result<Planner<TR::NonEmptyRtt>, TR::Error>
        where TR: TransAddRoot<ERT>
    {
        Ok(Planner { rtt: trans.add_root(self.empty_rtt)?, })
    }

    pub fn add_root_ok<TR>(self, trans: TR) -> Planner<TR::NonEmptyRtt>
        where TR: TransAddRoot<ERT, Error = util::NeverError>
    {
        self.add_root(trans)
            .unwrap_or_else(|_: util::NeverError| unreachable!())
    }
}

// Planner

pub struct Planner<RT> {
    rtt: RT,
}

pub trait TransRootNode<RT> {
    type RttNodeRef;
    type Error;

    fn root_node(self, rtt: &mut RT) -> Result<Self::RttNodeRef, Self::Error>;
}

impl<RT, F, NR, E> TransRootNode<RT> for F where F: FnOnce(&mut RT) -> Result<NR, E> {
    type RttNodeRef = NR;
    type Error = E;

    fn root_node(self, rtt: &mut RT) -> Result<Self::RttNodeRef, Self::Error> {
        (self)(rtt)
    }
}

impl<RT> Planner<RT> {
    pub fn root_node<TR>(mut self, trans: TR) ->
        Result<PlannerRttNode<RT, TR::RttNodeRef>, TR::Error>
        where TR: TransRootNode<RT>
    {
        Ok(PlannerRttNode {
            node_ref: trans.root_node(&mut self.rtt)?,
            rtt: self.rtt,
        })
    }

    pub fn root_node_ok<TR>(self, trans: TR) -> PlannerRttNode<RT, TR::RttNodeRef>
        where TR: TransRootNode<RT, Error = util::NeverError>
    {
        self.root_node(trans)
            .unwrap_or_else(|_: util::NeverError| unreachable!())
    }
}

// PlannerRttNode

pub struct PlannerRttNode<RT, NR> {
    rtt: RT,
    node_ref: NR,
}

pub trait TransIntoPath<RT, NR> {
    type RttPath;
    type Error;

    fn into_path(self, rtt: RT, node_ref: NR) -> Result<Self::RttPath, Self::Error>;
}

impl<RT, NR, F, P, E> TransIntoPath<RT, NR> for F where F: FnOnce(RT, NR) -> Result<P, E> {
    type RttPath = P;
    type Error = E;

    fn into_path(self, rtt: RT, node_ref: NR) -> Result<Self::RttPath, Self::Error> {
        (self)(rtt, node_ref)
    }
}

pub trait TransPrepareSample<RT, NR> {
    type Error;

    fn prepare_sample(self, rtt: &mut RT, node_ref: NR) -> Result<(), Self::Error>;
}

impl<RT, NR, F, E> TransPrepareSample<RT, NR> for F where F: FnOnce(&mut RT, NR) -> Result<(), E> {
    type Error = E;

    fn prepare_sample(self, rtt: &mut RT, node_ref: NR) -> Result<(), Self::Error> {
        (self)(rtt, node_ref)
    }
}

impl<RT, NR> PlannerRttNode<RT, NR> {
    pub fn rtt(&self) -> &RT {
        &self.rtt
    }

    pub fn node_ref(&self) -> &NR {
        &self.node_ref
    }

    pub fn into_path<TR>(self, trans: TR) -> Result<TR::RttPath, TR::Error>
        where TR: TransIntoPath<RT, NR>
    {
        trans.into_path(self.rtt, self.node_ref)
    }

    pub fn prepare_sample<TR>(mut self, trans: TR) -> Result<PlannerReadyToSample<RT>, TR::Error>
        where TR: TransPrepareSample<RT, NR>
    {
        let () = trans.prepare_sample(&mut self.rtt, self.node_ref)?;
        Ok(PlannerReadyToSample { rtt: self.rtt, })
    }
}

// PlannerReadyToSample

pub struct PlannerReadyToSample<RT> {
    rtt: RT,
}

pub trait TransSample<RT> {
    type Sample;
    type Error;

    fn sample(self, rtt: &mut RT) -> Result<Self::Sample, Self::Error>;
}

impl<RT, F, S, E> TransSample<RT> for F where F: FnOnce(&mut RT) -> Result<S, E> {
    type Sample = S;
    type Error = E;

    fn sample(self, rtt: &mut RT) -> Result<Self::Sample, Self::Error> {
        (self)(rtt)
    }
}

impl<RT> PlannerReadyToSample<RT> {
    pub fn rtt(&self) -> &RT {
        &self.rtt
    }

    pub fn sample<TR>(mut self, trans: TR) ->
        Result<PlannerSample<RT, TR::Sample>, TR::Error>
        where TR: TransSample<RT>
    {
        let sample = trans.sample(&mut self.rtt)?;
        Ok(PlannerSample { rtt: self.rtt, sample, })
    }
}

// PlannerSample

pub struct PlannerSample<RT, S> {
    rtt: RT,
    sample: S,
}

pub trait TransClosestToSample<RT, S> {
    type RttNodeRef;
    type Error;

    fn closest_to_sample(self, rtt: &mut RT, sample: S) -> Result<Self::RttNodeRef, Self::Error>;
}

impl<RT, S, F, NR, E> TransClosestToSample<RT, S> for F where F: FnOnce(&mut RT, S) -> Result<NR, E> {
    type RttNodeRef = NR;
    type Error = E;

    fn closest_to_sample(self, rtt: &mut RT, sample: S) -> Result<Self::RttNodeRef, Self::Error> {
        (self)(rtt, sample)
    }
}

impl<RT, S> PlannerSample<RT, S> {
    pub fn rtt(&self) -> &RT {
        &self.rtt
    }

    pub fn sample(&self) -> &S {
        &self.sample
    }

    pub fn closest_to_sample<TR>(mut self, trans: TR) ->
        Result<PlannerClosestNodeFound<RT, TR::RttNodeRef>, TR::Error>
        where TR: TransClosestToSample<RT, S>
    {
        let node_ref = trans.closest_to_sample(&mut self.rtt, self.sample)?;
        Ok(PlannerClosestNodeFound { rtt: self.rtt, node_ref, })
    }
}

// PlannerClosestNodeFound

pub struct PlannerClosestNodeFound<RT, NR> {
    rtt: RT,
    node_ref: NR,
}

pub trait TransNoTransition<RT, NR> {
    type Error;

    fn no_transition(self, rtt: &mut RT, node_ref: NR) -> Result<(), Self::Error>;
}

impl<RT, NR, F, E> TransNoTransition<RT, NR> for F where F: FnOnce(&mut RT, NR) -> Result<(), E> {
    type Error = E;

    fn no_transition(self, rtt: &mut RT, node_ref: NR) -> Result<(), Self::Error> {
        (self)(rtt, node_ref)
    }
}

pub trait TransTransition<RT, NR> {
    type Error;

    fn transition(self, rtt: &mut RT, node_ref: NR) -> Result<NR, Self::Error>;
}

impl<RT, NR, F, E> TransTransition<RT, NR> for F where F: FnOnce(&mut RT, NR) -> Result<NR, E> {
    type Error = E;

    fn transition(self, rtt: &mut RT, node_ref: NR) -> Result<NR, Self::Error> {
        (self)(rtt, node_ref)
    }
}

impl<RT, NR> PlannerClosestNodeFound<RT, NR> {
    pub fn rtt(&self) -> &RT {
        &self.rtt
    }

    pub fn node_ref(&self) -> &NR {
        &self.node_ref
    }

    pub fn no_transition<TR>(mut self, trans: TR) -> Result<PlannerReadyToSample<RT>, TR::Error>
        where TR: TransNoTransition<RT, NR>
    {
        let () = trans.no_transition(&mut self.rtt, self.node_ref)?;
        Ok(PlannerReadyToSample { rtt: self.rtt, })
    }

    pub fn transition<TR>(mut self, trans: TR) -> Result<PlannerRttNode<RT, NR>, TR::Error>
        where TR: TransTransition<RT, NR>
    {
        let node_ref = trans.transition(&mut self.rtt, self.node_ref)?;
        Ok(PlannerRttNode { rtt: self.rtt, node_ref, })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton() {

        #[derive(PartialEq, Debug)]
        enum Expect {
            PlannerInit,
            Planner,
            PlannerRttNode,
            PlannerReadyToSample(usize),
            PlannerSample(usize),
            PlannerClosestNodeFound(usize),
        }

        let planner = PlannerInit::new(Expect::PlannerInit);
        let planner = planner.add_root_ok(|rtt| {
            assert_eq!(rtt, Expect::PlannerInit);
            Ok(Expect::Planner)
        });

        let mut planner_node = planner.root_node_ok(|rtt: &mut _| {
            assert_eq!(rtt, &mut Expect::Planner);
            *rtt = Expect::PlannerRttNode;
            Ok(1)
        });

        let mut sample_counter = 0;
        loop {
            let expected_nodes_count = sample_counter / 2 + 1;
            assert_eq!(planner_node.rtt(), &Expect::PlannerRttNode);
            assert_eq!(planner_node.node_ref(), &expected_nodes_count);
            if *planner_node.node_ref() >= 10 {
                break;
            }

            let mut planner_ready_to_sample = planner_node.prepare_sample(|rtt: &mut Expect, node_ref| {
                assert_eq!(rtt, &mut Expect::PlannerRttNode);
                *rtt = Expect::PlannerReadyToSample(node_ref);
                Ok::<_, ()>(())
            }).unwrap();

            loop {
                let planner_sample = planner_ready_to_sample.sample(|rtt: &mut Expect| {
                    assert_eq!(rtt, &mut Expect::PlannerReadyToSample(expected_nodes_count));
                    sample_counter += 1;
                    *rtt = Expect::PlannerSample(expected_nodes_count);
                    Ok::<_, ()>(sample_counter)
                }).unwrap();
                assert_eq!(planner_sample.sample(), &sample_counter);

                let planner_closest = planner_sample.closest_to_sample(|rtt: &mut Expect, sample| {
                    assert_eq!(rtt, &mut Expect::PlannerSample(expected_nodes_count));
                    assert_eq!(sample, sample_counter);
                    *rtt = Expect::PlannerClosestNodeFound(expected_nodes_count);
                    Ok::<_, ()>(expected_nodes_count)
                }).unwrap();

                if sample_counter % 2 == 0 {
                    planner_node = planner_closest.transition(|rtt: &mut Expect, node_ref| {
                        assert_eq!(rtt, &mut Expect::PlannerClosestNodeFound(expected_nodes_count));
                        *rtt = Expect::PlannerRttNode;
                        Ok::<_, ()>(node_ref + 1)
                    }).unwrap();
                    break;
                } else {
                    planner_ready_to_sample = planner_closest.no_transition(|rtt: &mut Expect, _node_ref| {
                        assert_eq!(rtt, &mut Expect::PlannerClosestNodeFound(expected_nodes_count));
                        *rtt = Expect::PlannerReadyToSample(expected_nodes_count);
                        Ok::<_, ()>(())
                    }).unwrap();
                }
            }
        }

        let path = planner_node.into_path(|rtt: Expect, node_ref| {
            assert_eq!(rtt, Expect::PlannerRttNode);
            Ok::<_, ()>(node_ref)
        }).unwrap();
        assert_eq!(path, sample_counter / 2 + 1);
    }
}
