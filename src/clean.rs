use crate::content_sha::ContentSha;
use crate::{config::Config, git_object_id::GitObjectId};
use anyhow::{anyhow, Error, Result};
use fastcdc::v2020::StreamCDC;
use log::trace;
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

pub fn clean(options: CleanOptions) -> Result<()> {
    trace!("Running command 'clean'");

    let (file_names_ordered, file_name_to_content) = split_into_chunks(options)?;
    trace!("Chunks split");

    // create all the blobs
    let file_name_to_git_sha = git_ensure_blobs(&file_name_to_content)?;
    trace!("Created blobs");

    git_update_index_add_many(&file_name_to_git_sha)?;

    // write to stdout for git clean
    let pointer_file = create_pointer_file(file_names_ordered, file_name_to_git_sha)?;

    stdout().write_all(pointer_file.as_bytes())?;

    trace!("Pointer file sent");

    Ok(())
}

fn git_ensure_blobs(
    file_name_to_content: &HashMap<ContentSha, Vec<u8>>,
) -> Result<HashMap<&ContentSha, GitObjectId>> {
    file_name_to_content
        .iter()
        .map(|(content_sha, contents)| {
            let git_object_sized = GitObjectId::from_contents(contents)?;
            Ok((content_sha, git_object_sized))
        })
        .collect::<Result<HashMap<_, _>>>()
}

// create all the entries in the index
// the reverse will still exist in the worktree for now.
fn git_update_index_add_many(entries: &HashMap<&ContentSha, GitObjectId>) -> Result<()> {
    let base = PathBuf::from_str(".gfs/contents")?;

    let mut paths = Vec::<PathBuf>::with_capacity(entries.len());

    for (content_sha, git_object) in entries {
        let path = base.join(content_sha);

        let mode_sha_path = format!("100644,{},{}\n", git_object, path.display());

        Command::new("git")
            .args(["update-index", "--add", "--cache-info"])
            .arg(mode_sha_path)
            .output()?;

        paths.push(path)
    }

    trace!("Created index but with worktree");

    Command::new("git")
        .args(["update-index", "--skip-worktree"])
        .args(paths)
        .output()?;

    trace!("Updated index by skipping worktree");

    Ok(())
}

fn create_pointer_file(
    file_names_ordered: Vec<ContentSha>,
    file_name_to_git_sha: HashMap<&ContentSha, GitObjectId>,
) -> Result<String> {
    Ok(file_names_ordered
        .iter()
        .map(|content_sha| {
            file_name_to_git_sha
                .get(content_sha)
                .ok_or_else(|| anyhow!("Expected to find this"))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|a| a.value())
        .intersperse(&"\n".to_string())
        .cloned()
        .collect::<String>())
}

fn split_into_chunks(
    options: CleanOptions,
) -> Result<(Vec<ContentSha>, HashMap<ContentSha, Vec<u8>>)> {
    let source = stdin();

    let iter = StreamCDC::new(source, options.min_size, options.avg_size, options.max_size);

    let mut files = HashMap::<ContentSha, Vec<u8>>::new();

    // todo: make &string and borrow from the hashmap
    let mut file_names_ordered = Vec::<ContentSha>::new();

    trace!("Iterating CDC streaming");

    for item in iter {
        let chunk = item?;

        // create a unique name - SHA1 seemed acceptable.
        let sha = ContentSha::from_contents(&chunk.data);

        files.insert(sha.clone(), chunk.data);
        file_names_ordered.push(sha);
    }

    Ok((file_names_ordered, files))
}
