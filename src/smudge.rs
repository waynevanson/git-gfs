use anyhow::Result;
use std::fs::File;
use std::io::{self, stdin, stdout};
use std::path::PathBuf;
use std::str::FromStr;

// deserialize the file and concat the contents
// content is line separated paths to read
pub fn smudge() -> Result<()> {
    let path = PathBuf::from_str(".gfs/contents")?;

    let mut out = stdout();

    for line in stdin().lines() {
        let path = path.join(line?);
        let mut file = File::open(path)?;
        io::copy(&mut file, &mut out)?;
    }

    Ok(())
}
