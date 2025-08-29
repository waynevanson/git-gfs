pub struct FlatMapOkIter<I, O, F> {
    iter_t: I,
    iter_u: Option<O>,
    kleisli: F,
}

impl<I, O, F, T, U, E> Iterator for FlatMapOkIter<I, O::IntoIter, F>
where
    I: Iterator<Item = Result<T, E>>,
    O: IntoIterator<Item = Result<U, E>>,
    F: Fn(T) -> O,
{
    type Item = Result<U, E>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut inner) = self.iter_u {
                if let Some(u) = inner.next() {
                    return Some(u);
                }
                self.iter_u = None;
            }

            match self.iter_t.next()? {
                Ok(t) => {
                    self.iter_u = Some((self.kleisli)(t).into_iter());
                    continue;
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

pub trait IntoFlatMapOkIter<O, F, T, E> {
    fn flat_map_ok<U>(self, kleisli: F) -> FlatMapOkIter<Self, O::IntoIter, F>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
        O: IntoIterator<Item = Result<U, E>>,
        F: Fn(T) -> O,
    {
        FlatMapOkIter {
            iter_t: self,
            iter_u: None,
            kleisli,
        }
    }
}

impl<I, O, F, T, E> IntoFlatMapOkIter<O, F, T, E> for I where I: Iterator<Item = Result<T, E>> {}
