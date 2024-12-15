use anyhow::{bail, Result};
use gix::bstr::ByteSlice;
use std::process::Output;

pub trait SealedOutput
where
    Self: Sized,
{
    /// Ensure the command was successful,
    /// and ensure `stderr` is in the error.
    fn exit_ok_or_stderror(self) -> Result<Self>;
}

impl SealedOutput for Output {
    fn exit_ok_or_stderror(self) -> Result<Self> {
        if !self.status.success() {
            let str = self.stderr.to_str()?.to_owned();
            bail!(str);
        }

        Ok(self)
    }
}
