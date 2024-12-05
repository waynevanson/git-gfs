use prng_split::AlphaPathSegment;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

/// Split a reader into parts.
pub struct Splitter {
    size: u64,
    prefix: PathBuf,
    suffix: AlphaPathSegment,
    remaining: u64,
    writer: Option<File>,
}

impl Splitter {
    pub fn new(prefix: impl AsRef<Path>, size: u64, factor: usize) -> Self {
        let suffix = AlphaPathSegment::from_factor(factor);

        Self {
            size,
            prefix: prefix.as_ref().to_path_buf(),
            suffix,
            remaining: size,
            writer: None,
        }
    }
}

impl Write for Splitter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.writer.is_none() {
            let mut path = self.prefix.clone();
            path.push(&self.suffix);
            self.writer = Some(File::create(path)?);
        }

        let writer = self.writer.as_mut().expect("Expected to have 'writer' set");

        let last: usize = (buf.len() as u64).min(self.remaining).try_into().expect(
            "Expected the minimum of buffer length and the remaining bytes to fix inside 'usize'",
        );

        let buf = &buf[0..last];

        let bytes = writer.write(buf)?;
        let written = bytes as u64;

        // Have we written enough for this file?
        if written < self.remaining {
            self.remaining -= written;
        } else {
            self.writer = None;
            self.remaining = self.size;
        }

        Ok(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(writer) = &mut self.writer {
            writer.flush()?;
        }

        Ok(())
    }
}
