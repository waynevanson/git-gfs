pub trait MapOkThen<Value, Error, Kleisli> {
    /// When the item of an iterator is a `Result`, apply a function "bind" (kleisli)
    /// to the item. Fn(value) -> Result<value, error>
    fn map_ok_then<Next>(self, kleisli: Kleisli) -> MapOkThenIter<Self, Kleisli>
    where
        Self: Iterator<Item = Result<Value, Error>> + Sized,
        Kleisli: Fn(Value) -> Result<Next, Error>,
    {
        MapOkThenIter {
            iter: self,
            kleisli,
        }
    }
}

impl<T, Value, Error, Kleisli> MapOkThen<Value, Error, Kleisli> for T {}

pub struct MapOkThenIter<I, F> {
    iter: I,
    kleisli: F,
}

impl<Iter, Kleisli, Value, Error, Next> Iterator for MapOkThenIter<Iter, Kleisli>
where
    Iter: Iterator<Item = Result<Value, Error>> + Sized,
    Kleisli: Fn(Value) -> Result<Next, Error>,
{
    type Item = Result<Next, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|result| result.and_then(|value| (self.kleisli)(value)))
    }
}
