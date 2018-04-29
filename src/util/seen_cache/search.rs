
pub struct Search;

impl<S> super::SeenCache<S> for Search where S: PartialEq {
    type Error = ();

    fn remember(&mut self, _state: &S) -> Result<(), Self::Error> {
        Ok(())
    }

    fn already_seen<'a, I>(&self, state: &S, states_iter: I) ->
        Result<bool, Self::Error>
        where I: Iterator<Item = Result<&'a S, Self::Error>>,
              S: 'a,
    {
        for maybe_seen_state in states_iter {
            let seen_state = maybe_seen_state?;
            if seen_state == state {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
