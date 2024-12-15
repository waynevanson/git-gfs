use anyhow::{bail, Result};
use gix::bstr::ByteSlice;
use std::process::Output;

pub trait SealedOutput {
    fn exit_ok_or_stderror(self) -> Result<()>;
}

impl SealedOutput for Output {
    fn exit_ok_or_stderror(self) -> Result<()> {
        if !self.status.success() {
            let str = self.stderr.to_str()?.to_owned();
            bail!(str);
        }

        Ok(())
    }
}
