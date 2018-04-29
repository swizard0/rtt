pub mod search;
pub mod hash_set;

pub trait SeenCache<S> {
    type Error;

    fn remember(&mut self, state: &S) -> Result<(), Self::Error>;

    fn already_seen<'a, I>(&self, state: &S, states_iter: I) ->
        Result<bool, Self::Error>
        where I: Iterator<Item = Result<&'a S, Self::Error>>,
              S: 'a;
}
