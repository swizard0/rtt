pub mod util;

// Planner

pub struct Planner<RT> {
    empty_rtt: RT,
}

impl<RT> Planner<RT> {
    pub fn new(empty_rtt: RT) -> Planner<RT> {
        Planner { empty_rtt, }
    }
}

pub trait TransAddRoot<RT> {
    type RttNodeRef;
    type Error;

    fn add_root(self, rtt: &mut RT) -> Result<Self::RttNodeRef, Self::Error>;
}

impl<RT, F, NR, E> TransAddRoot<RT> for F where F: FnOnce(&mut RT) -> Result<NR, E> {
    type RttNodeRef = NR;
    type Error = E;

    fn add_root(self, rtt: &mut RT) -> Result<Self::RttNodeRef, Self::Error> {
        (self)(rtt)
    }
}

impl<RT> Planner<RT> {
    pub fn add_root<TR>(mut self, trans: TR) ->
        Result<PlannerNodeExpanded<RT, TR::RttNodeRef>, TR::Error>
        where TR: TransAddRoot<RT>
    {
        let node_ref = trans.add_root(&mut self.empty_rtt)?;
        Ok(PlannerNodeExpanded { rtt: self.empty_rtt, node_ref, })
    }
}

// PlannerNodeExpanded

pub struct PlannerNodeExpanded<RT, NR> {
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

impl<RT, NR> PlannerNodeExpanded<RT, NR> {
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
        Result<PlannerNearestNodeFound<RT, TR::RttNodeRef>, TR::Error>
        where TR: TransClosestToSample<RT, S>
    {
        let node_ref = trans.closest_to_sample(&mut self.rtt, self.sample)?;
        Ok(PlannerNearestNodeFound { rtt: self.rtt, node_ref, })
    }
}

// PlannerNearestNodeFound

pub struct PlannerNearestNodeFound<RT, NR> {
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

    fn transition(self, rtt: &mut RT, node_ref: &NR) -> Result<(), Self::Error>;
}

impl<RT, NR, F, E> TransTransition<RT, NR> for F where F: FnOnce(&mut RT, &NR) -> Result<(), E> {
    type Error = E;

    fn transition(self, rtt: &mut RT, node_ref: &NR) -> Result<(), Self::Error> {
        (self)(rtt, node_ref)
    }
}

impl<RT, NR> PlannerNearestNodeFound<RT, NR> {
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

    pub fn transition<TR>(mut self, trans: TR) -> Result<PlannerNodeExpanded<RT, NR>, TR::Error>
        where TR: TransTransition<RT, NR>
    {
        let () = trans.transition(&mut self.rtt, &self.node_ref)?;
        Ok(PlannerNodeExpanded { rtt: self.rtt, node_ref: self.node_ref, })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton() {

        #[derive(PartialEq, Debug)]
        enum Expect {
            Planner,
            PlannerNodeExpanded(usize),
        }

        let planner = Planner::new(Expect::Planner);

        let mut planner_node = planner.add_root(|rtt: &mut Expect| {
            assert_eq!(rtt, &mut Expect::Planner);
            *rtt = Expect::PlannerNodeExpanded(1);
            Ok::<_, ()>(1)
        }).unwrap();
        assert_eq!(planner_node.rtt(), &Expect::PlannerNodeExpanded(1));
        assert_eq!(planner_node.node_ref(), &1);

        // loop {
        //     match planner_node.rtt_node() {
        //         &Step::NodesCount(10) =>
        //             break,
        //         &Step::NodesCount(..) =>
        //             (),
        //         other =>
        //             panic!("Invalid state on limit step: {:?}", other),
        //     }

        //     let mut planner_sample = planner_node.prepare_sample(|s| {
        //         if let Step::NodesCount(count) = s {
        //             Ok::<_, ()>(Step::SamplePrepared(count))
        //         } else {
        //             panic!("Invalid state on prepare sample step: {:?}", s)
        //         }
        //     }).unwrap();

        //     loop {
        //         let next_sample = if let &Step::SamplePrepared(count) = planner_sample.rtt() {
        //             count + 1
        //         } else {
        //             panic!("Invalid state on sample picking step");
        //         };

        //         let planner_nearest = planner_sample.closest_to_sample(|_| {
        //             Ok::<_, ()>(Step::SampleNearest(next_sample))
        //         }).unwrap();

        //         let value = if let &Step::SampleNearest(value) = planner_nearest.rtts_node() {
        //             value
        //         } else {
        //             panic!("Invalid on transition step: {:?}", planner_nearest.rtts_node())
        //         };

        //         if value % 2 != 0 {
        //             planner_sample = planner_nearest.no_transition(|_| {
        //                 Ok::<_, ()>(Step::SamplePrepared(value))
        //             }).unwrap();
        //         } else {
        //             planner_node = planner_nearest.transition(|_| {
        //                 Ok::<_, ()>(Step::NodesCount(value))
        //             }).unwrap();
        //             break;
        //         }
        //     }
        // }

        // let path = planner_node.into_path(|s| {
        //     assert_eq!(s, Step::NodesCount(10));
        //     Ok::<_, ()>(Step::Path)
        // }).unwrap();
        // assert_eq!(path, Step::Path);
    }
}
