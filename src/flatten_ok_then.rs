pub trait FlattenOkThen<Value, Error, Kleisli, Iter> {
    fn flatten_ok_then<Next>(self, kleisli: Kleisli) -> FlattenOkThenIter<Self, Iter, Kleisli>
    where
        Self: Iterator<Item = Result<Value, Error>> + Sized,
        Kleisli: FnMut(Value) -> Result<Iter, Error>,
        Iter: Iterator<Item = Result<Next, Error>> + Sized,
    {
        FlattenOkThenIter {
            iter_outer: self,
            kleisli,
            iter_inner: None,
        }
    }
}

impl<T, Value, Error, Kleisli, Iter> FlattenOkThen<Value, Error, Kleisli, Iter> for T {}

pub struct FlattenOkThenIter<IterOuter, IterInner, Kleisli> {
    iter_outer: IterOuter,
    kleisli: Kleisli,
    iter_inner: Option<IterInner>,
}

impl<IterOuter, Value, Error, Next, Kleisli, IterInner> Iterator
    for FlattenOkThenIter<IterOuter, IterInner, Kleisli>
where
    IterOuter: Iterator<Item = Result<Value, Error>> + Sized,
    Kleisli: FnMut(Value) -> IterInner,
    IterInner: Iterator<Item = Result<Next, Error>> + Sized,
{
    type Item = Result<Next, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // let mut iter_inner_ = self.iter_inner;

        // while let Some(iter_inner) = iter_inner_ {
        //     if let Some(inner) = iter_inner.next() {
        //         return Some(inner);
        //     }

        //     let result_outer = self.iter_outer.next()?;
        //     let result_iter_inner = result_outer.map(self.kleisli);

        //     match result_iter_inner {
        //         Err(error) => return Some(Err(error)),
        //         Ok(iter_inner) => {
        //             self.iter_inner = Some(iter_inner);
        //         }
        //     }
        // }

        // if inner is none, get next outer we can make inner.

        // try get the inner value repetedly until we cannot no more.

        None
    }
}
