use std::{collections as coll, hash::Hash};

pub struct HashSet<S>(coll::HashSet<S>);

impl<S> HashSet<S> where S: PartialEq + Eq + Hash {
    pub fn new() -> HashSet<S> {
        HashSet(coll::HashSet::new())
    }
}

impl<S> super::SeenCache<S> for HashSet<S> where S: Clone + PartialEq + Eq + Hash {
    type Error = ();

    fn remember(&mut self, state: &S) -> Result<(), Self::Error> {
        self.0.insert(state.clone());
        Ok(())
    }

    fn already_seen<'a, I>(&self, state: &S, _states_iter: I) ->
        Result<bool, Self::Error>
        where I: Iterator<Item = Result<&'a S, Self::Error>>,
              S: 'a
    {
        Ok(self.0.contains(state))
    }
}
