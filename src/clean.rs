use crate::config::Config;
use anyhow::anyhow;
use anyhow::{Error, Result};
use fastcdc::v2020::StreamCDC;
use itertools::Itertools;
use sha1::{Digest, Sha1};
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

impl TryFrom<Config> for CleanOptions {
    type Error = Error;

    fn try_from(config: Config) -> anyhow::Result<Self> {
        Ok(Self {
            avg_size: u32::try_from(config.avg_size.0)?,
            min_size: u32::try_from(config.min_size.0)?,
            max_size: u32::try_from(config.max_size.0)?,
        })
    }
}

fn git_ensure_blob(contents: &[u8]) -> Result<String> {
    let child = Command::new("git")
        .args(["hash-object", "type", "-w", "--no-filters", "--stdin"])
        .spawn()?;

    // using stdin so we don't have to specify a file to read
    child
        .stdin
        .as_ref()
        .ok_or_else(|| anyhow!("Expected stdin to exist"))?
        .write_all(&contents)?;

    let git_sha: String = child.wait_with_output()?.stdout.try_into()?;

    Ok(git_sha)
}

fn git_update_index_add_many(
    entries: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
) -> Result<()> {
    let base = PathBuf::from_str(".gfs/contents")?;

    // create all the entries in the index
    // the reverse will still exist in the worktree for now.
    let entries = entries
        .into_iter()
        .map(|(content_sha, git_sha)| {
            let path = base.join(content_sha.as_ref());
            let arg = format!("100644,{},{}", git_sha.as_ref(), path.display());
            arg
        })
        .join("\n");

    let mut child = Command::new("git")
        .args(["update-index", "--add", "--index-info"])
        .spawn()?;

    child
        .stdin
        .as_ref()
        .ok_or_else(|| anyhow!("Expected stdin handle"))?
        .write_all(entries.as_bytes())?;

    child.wait()?;

    Ok(())
}

fn git_update_index_skip_worktree_many(
    entries: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
) -> Result<()> {
    let entries = entries.into_iter().map(|(content_sha, git_sha)| {
        format!("100644 {}\t{}", git_sha.as_ref(), content_sha.as_ref())
    });

    Command::new("git")
        .args(["update-index", "--skip-worktree"])
        .args(entries)
        .output()?;

    Ok(())
}

pub fn clean(options: CleanOptions) -> Result<()> {
    let (file_names_ordered, file_name_to_content) = split_into_chunks(options)?;

    // create all the blobs
    let file_name_to_git_sha = file_name_to_content
        .iter()
        .map(|(content_sha, contents)| {
            let git_sha = git_ensure_blob(&contents)?;
            Ok((content_sha, git_sha))
        })
        .collect::<Result<HashMap<_, _>>>()?;

    git_update_index_add_many(&file_name_to_git_sha)?;

    // skip the worktree for all files
    git_update_index_skip_worktree_many(&file_name_to_git_sha)?;

    // write to stdout for git clean
    let pointer_file = file_names_ordered
        .iter()
        .map(|content_sha| {
            file_name_to_git_sha
                .get(content_sha)
                .ok_or_else(|| anyhow!("Expected to find this"))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .intersperse(&"\n".to_string())
        .cloned()
        .collect::<String>();

    stdout().write_all(pointer_file.as_bytes())?;

    Ok(())
}

fn split_into_chunks(options: CleanOptions) -> Result<(Vec<String>, HashMap<String, Vec<u8>>)> {
    let source = stdin();

    let iter = StreamCDC::new(source, options.min_size, options.avg_size, options.max_size);

    let mut files = HashMap::<String, Vec<u8>>::new();

    // todo: make &string and borrow from the hashmap
    let mut file_names_ordered = Vec::<String>::new();

    for item in iter {
        let chunk = item?;

        // create a unique name - SHA1 seemed acceptable.
        let sha: String = Sha1::digest(&chunk.data).to_vec().try_into()?;

        files.insert(sha.clone(), chunk.data);

        file_names_ordered.push(sha);
    }

    Ok((file_names_ordered, files))
}
