pub mod search;

pub trait SeenCache {
    type State;
    type Error;

    fn remember(&mut self, state: &Self::State) -> Result<(), Self::Error>;

    fn already_seen<'a, I>(&self, state: &Self::State, states_iter: I) ->
        Result<bool, Self::Error>
        where I: Iterator<Item = Result<&'a Self::State, Self::Error>>,
              Self::State: 'a;
}
