
pub trait RandomTree {
    type State;
    type Error;
    type Node: RandomTreeNode<State = Self::State, Error = Self::Error>;

    fn root(self) -> Self::Node;
    fn nearest_node(self, state: &Self::State) -> Result<Self::Node, Self::Error>;
}

pub trait RandomTreeNode: Sized {
    type State;
    type Error;
    type Tree: RandomTree<State = Self::State, Error = Self::Error>;
    type Path;

    fn expand(self, state: Self::State) -> Result<Self, Self::Error>;

    fn adjust_random_state(&self, random_state: Self::State) -> Result<Self::State, Self::Error> {
        Ok(random_state)
    }

    fn into_tree(self) -> Self::Tree;
    fn into_path(self) -> Self::Path;
}

pub trait Sampler<RT> where RT: RandomTree {
    type Error;

    fn sample(&mut self, rtt: &RT) -> Result<Option<RT::State>, Self::Error>;
}

pub trait Limiter<RT> where RT: RandomTree {
    type Error;

    fn limit_exceeded(&mut self, rtt: &RT) -> Result<bool, Self::Error>;
}

pub trait GoalChecker<RN> where RN: RandomTreeNode {
    type Error;

    fn goal_reached(&mut self, node: &RN) -> Result<bool, Self::Error>;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error<RTE, SE, LE, GCE> {
    RandomTree(RTE),
    Sampler(SE),
    Limiter(LE),
    GoalChecker(GCE),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome<P> {
    PathPlanned(P),
    NoPathExists,
    LimitReached,
}

pub fn plan<RT, RN, S, L, GC>(
    rtt: RT,
    mut sampler: S,
    mut limiter: L,
    mut goal_checker: GC,
    init: RT::State
) ->
    Result<Outcome<RN::Path>, Error<RT::Error, S::Error, L::Error, GC::Error>>
    where RT: RandomTree<Node = RN>,
          RN: RandomTreeNode<State = RT::State, Error = RT::Error, Tree = RT>,
          S: Sampler<RT>,
          L: Limiter<RT>,
          GC: GoalChecker<RN>,
{
    let root = rtt.root();
    let mut node = root.expand(init).map_err(Error::RandomTree)?;

    loop {
        if goal_checker.goal_reached(&node).map_err(Error::GoalChecker)? {
            return Ok(Outcome::PathPlanned(node.into_path()));
        }

        let rtt = node.into_tree();

        if limiter.limit_exceeded(&rtt).map_err(Error::Limiter)? {
            return Ok(Outcome::LimitReached);
        }

        if let Some(random_state) = sampler.sample(&rtt).map_err(Error::Sampler)? {
            node = rtt.nearest_node(&random_state).map_err(Error::RandomTree)?;
            let new_state =
                node.adjust_random_state(random_state).map_err(Error::RandomTree)?;
            node = node.expand(new_state).map_err(Error::RandomTree)?;
        } else {
            return Ok(Outcome::NoPathExists);
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
