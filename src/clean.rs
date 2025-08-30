use crate::config::Config;
use anyhow::anyhow;
use anyhow::{Error, Result};
use fastcdc::v2020::StreamCDC;
use itertools::Itertools;
use log::trace;
use sha1::{Digest, Sha1};
use std::io::BufRead;
use std::process::Stdio;
use std::{
    collections::HashMap,
    io::{stdin, Write},
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
        .args(["hash-object", "-w", "--no-filters", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // using stdin so we don't have to specify a file to read
    child
        .stdin
        .as_ref()
        .ok_or_else(|| anyhow!("Expected stdin to exist"))?
        .write_all(&contents)?;

    let git_sha: String = child.wait_with_output()?.stdout.try_into()?;

    // remove newlines (always at end)
    let git_sha = git_sha.lines().join("");

    trace!("Created GitSha1 '{}'", git_sha);

    Ok(git_sha)
}

fn git_update_index_add_many(entries: &HashMap<&String, String>) -> Result<()> {
    let base = PathBuf::from_str(".gfs/contents")?;

    // %(objectmode) %(objecttype) %(objectname) %(objectsize:padded)%x09%(path)
    //
    // Various values from structured fields can be used to interpolate into the resulting output. For each outputting line, the following names can be used:

    //
    // objectmode
    //     The mode of the object.
    // objecttype
    //     The type of the object (commit, blob or tree).
    // objectname
    //     The name of the object.
    // objectsize[:padded]
    //     The size of a blob object ("-" if it’s a commit or tree). It also supports a padded format of size with "%(objectsize:padded)".
    // path
    //     The pathname of the object.

    // todo: probably get the sizes when creating them?
    // but batching is probably just so good.
    // todo: use stdin for this. needs some special bytes to kick off stdin
    let sizes: Vec<_> = Command::new("git")
        .args(["cat-file", "--batch-check", "-s"])
        .args(entries.iter().map(|(_, git_sha)| git_sha))
        .output()?
        .stdout
        .lines()
        .try_collect()?;

    // create all the entries in the index
    // the reverse will still exist in the worktree for now.
    let entries = entries
        .into_iter()
        .zip_eq(sizes)
        .map(|((content_sha, git_sha), size)| {
            let path = base.join(content_sha);
            let arg = format!("100644 blob {} {} {}", size, git_sha, path.display());
            arg
        })
        .join("\n");

    let mut child = Command::new("git")
        .args(["update-index", "--add", "--index-info"])
        .stdin(Stdio::piped())
        .spawn()?;

    child
        .stdin
        .as_ref()
        .ok_or_else(|| anyhow!("Expected stdin handle"))?
        .write_all(entries.as_bytes())?;

    child.wait()?;

    Ok(())
}

// // todo: use the actual
// fn git_update_index_skip_worktree_many(entries: &HashMap<&String, String>) -> Result<()> {
//     let entries = entries
//         .into_iter()
//         .map(|(content_sha, git_sha)| format!("100644 blob {} {} {}", git_sha, content_sha));

//     Command::new("git")
//         .args(["update-index", "--skip-worktree"])
//         .args(entries)
//         .output()?;

//     Ok(())
// }

fn git_ensure_blobs(
    file_name_to_content: &HashMap<String, Vec<u8>>,
) -> Result<HashMap<&String, String>> {
    let value = file_name_to_content
        .iter()
        .map(|(content_sha, contents)| {
            let git_sha = git_ensure_blob(&contents)?;
            Ok((content_sha, git_sha))
        })
        .collect::<Result<HashMap<_, _>>>()?;

    Ok(value)
}

pub fn clean(options: CleanOptions) -> Result<()> {
    trace!("Running command 'clean'");

    let (_file_names_ordered, file_name_to_content) = split_into_chunks(options)?;
    trace!("Chunks split");

    // create all the blobs
    let file_name_to_git_sha = git_ensure_blobs(&file_name_to_content)?;
    trace!("{:?}", file_name_to_git_sha);
    trace!("Created blobs");

    git_update_index_add_many(&file_name_to_git_sha)?;
    trace!("Created index but with worktrees");

    todo!();

    // // skip the worktree for all files
    // git_update_index_skip_worktree_many(&file_name_to_git_sha)?;
    // trace!("Applied --skip-worktree");

    // // write to stdout for git clean
    // let pointer_file = create_pointer_file(file_names_ordered, file_name_to_git_sha)?;
    // stdout().write_all(pointer_file.as_bytes())?;

    // trace!("Pointer file sent");

    // Ok(())
}

// fn create_pointer_file(
//     file_names_ordered: Vec<String>,
//     file_name_to_git_sha: HashMap<&String, String>,
// ) -> Result<String> {
//     Ok(file_names_ordered
//         .iter()
//         .map(|content_sha| {
//             file_name_to_git_sha
//                 .get(content_sha)
//                 .ok_or_else(|| anyhow!("Expected to find this"))
//         })
//         .collect::<Result<Vec<_>>>()?
//         .into_iter()
//         .intersperse(&"\n".to_string())
//         .cloned()
//         .collect::<String>())
// }

fn split_into_chunks(options: CleanOptions) -> Result<(Vec<String>, HashMap<String, Vec<u8>>)> {
    let source = stdin();

    let iter = StreamCDC::new(source, options.min_size, options.avg_size, options.max_size);

    let mut files = HashMap::<String, Vec<u8>>::new();

    // todo: make &string and borrow from the hashmap
    let mut file_names_ordered = Vec::<String>::new();

    trace!("Iterating CDC streaming");

    for (index, item) in iter.enumerate() {
        let chunk = item?;
        trace!("Chunk successful at {}", index);

        // create a unique name - SHA1 seemed acceptable.
        let sha = Sha1::digest(&chunk.data);
        trace!("Sha1 generated for chunk {}", index);

        let sha: String = format!("{:x}", sha);
        trace!("Sha1 stringified for chunk {}", index);

        files.insert(sha.clone(), chunk.data);

        file_names_ordered.push(sha);
    }

    Ok((file_names_ordered, files))
}
