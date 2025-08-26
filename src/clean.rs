use crate::config::CleanConfig;
use anyhow::anyhow;
use anyhow::{Error, Result};
use fastcdc::v2020::StreamCDC;
use sha1::{Digest, Sha1};
use std::path::Path;
use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
    path::PathBuf,
    process::Command,
    str::FromStr,
};

pub struct CleanOptions {
    pub min_size: u32,
    pub max_size: u32,
    pub avg_size: u32,
}

impl TryFrom<CleanConfig> for CleanOptions {
    type Error = Error;

    fn try_from(config: CleanConfig) -> anyhow::Result<Self> {
        Ok(Self {
            min_size: u32::try_from(config.min_size.0)?,
            avg_size: u32::try_from(config.avg_size.0)?,
            max_size: u32::try_from(config.max_size.0)?,
        })
    }
}

fn git_ensure_blob(contents: &[u8]) -> Result<Vec<u8>> {
    let child = Command::new("git")
        .args(["hash-object", "type", "-w", "--no-filters", "--stdin"])
        .spawn()?;

    child
        .stdin
        .as_ref()
        .ok_or_else(|| anyhow!("Expected stdin to exist"))?
        .write_all(&contents)?;

    let git_sha = child.wait_with_output()?.stdout;

    Ok(git_sha)
}

fn git_update_index_skip_worktree(
    directory: &Path,
    content_sha: &str,
    git_sha: &str,
) -> Result<()> {
    let path = directory.join(content_sha);
    let arg = format!("100644,{},{}", git_sha, path.display());

    Command::new("git")
        .args(["update-index", "--add", "--cache-info", &arg])
        .output()?;

    // --skip-worktree only works with existing files in index
    // From here, we are doing some very funky stuff.
    Command::new("git")
        .args(["update-index", "--skip-worktree", content_sha])
        .output()?;

    Ok(())
}

pub fn clean(options: CleanOptions) -> Result<()> {
    let (file_names_ordered, file_name_to_content) = split_into_chunks(options)?;

    let base = PathBuf::from_str(".gfs/contents")?;

    // add chunks to git index only.
    for (content_sha, contents) in &file_name_to_content {
        // remember, this sha is `blob <contents>` as git intended.
        let git_sha = git_ensure_blob(contents)?;
        let git_sha: String = git_sha.try_into()?;

        git_update_index_skip_worktree(&base, content_sha, &git_sha)?;
    }

    // write to stdout for git clean
    let pointer_file = file_names_ordered.join("\n");

    stdout().write_all(pointer_file.as_bytes())?;

    Ok(())
}

//
fn split_into_chunks(options: CleanOptions) -> Result<(Vec<String>, HashMap<String, Vec<u8>>)> {
    let source = stdin();

    let iter = StreamCDC::new(source, options.min_size, options.avg_size, options.max_size);

    let mut files = HashMap::<String, Vec<u8>>::new();
    let mut file_names_ordered = Vec::<String>::new();

    for item in iter {
        let chunk = item?;

        // create a unique name - SHA1 seemed acceptable.
        let sha: String = Sha1::digest(&chunk.data).to_vec().try_into()?;

        file_names_ordered.push(sha.clone());
        files.insert(sha, chunk.data);
    }

    Ok((file_names_ordered, files))
}
