use std::io::{Read, Result};

pub struct IterReaderResult<I>
where
    I: Iterator<Item = Result<u8>>,
{
    iter: I,
}

impl<I> Read for IterReaderResult<I>
where
    I: Iterator<Item = Result<u8>>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        for slot in buf.iter_mut() {
            if let Some(byte) = self.iter.next() {
                *slot = byte?;
                i += 1;
            } else {
                break;
            }
        }
        Ok(i)
    }
}

pub trait IntoIterReaderResult
where
    Self: Iterator<Item = Result<u8>> + Sized,
{
    fn into_iter_reader_result(self) -> IterReaderResult<Self> {
        IterReaderResult { iter: self }
    }
}

impl<I> IntoIterReaderResult for I where Self: Iterator<Item = Result<u8>> + Sized {}
