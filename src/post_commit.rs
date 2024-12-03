use crate::map_ok_then::MapOkThen;
use anyhow::{anyhow, Result};
use clap::Parser;
use gix::{
    attrs::search::Match,
    bstr::ByteSlice,
    glob::{wildmatch, Pattern},
    object::tree::EntryRef,
    objs::{
        tree::{Entry, EntryKind},
        Tree,
    },
    refs::transaction::PreviousValue,
    Repository,
};
use glob::glob;
use itertools::Itertools;
use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Parser)]
pub struct PostCommit;

fn get_patterns(repo: &Repository) -> Result<HashSet<Pattern>> {
    let worktree = repo
        .worktree()
        .ok_or_else(|| anyhow!("Expected to find work tree"))?;

    let attributes = worktree.attributes(None)?.attribute_matches();

    // all patterns contain our custom filter=split -text
    let patterns = attributes
        .iter()
        .filter(has_split_attributes)
        .map(|r#match| r#match.pattern.to_owned())
        .collect::<HashSet<_>>();

    Ok(patterns)
}

fn has_split_attributes(r#match: &Match<'_>) -> bool {
    let is_filter = r#match.assignment.name.as_str() == "filter";

    let is_split = r#match
        .assignment
        .state
        .as_bstr()
        .map(|bstr| bstr == "split");

    matches!((is_filter, is_split), (true, Some(true)))
}

fn is_entry_file(entry: &EntryRef<'_, '_>) -> bool {
    matches!(
        entry.mode().kind(),
        EntryKind::Blob | EntryKind::BlobExecutable
    )
}

fn patterns_contains_entry(patterns: &HashSet<Pattern>, entry: &EntryRef<'_, '_>) -> bool {
    patterns
        .iter()
        .any(|pattern| pattern.matches(entry.filename(), wildmatch::Mode::NO_MATCH_SLASH_LITERAL))
}

fn create_git_parts_pattern(repo: &Repository, filename: &Path) -> Result<String> {
    let path = repo
        .path()
        .join("parts")
        .join(format!("{}.part.*", filename.to_string_lossy()));

    let pattern = path
        .to_str()
        .ok_or_else(|| anyhow!("Expected to create pattern as string slice"))?
        .to_string();

    Ok(pattern)
}

fn large_files_changed(repo: &Repository) -> Result<Vec<PathBuf>> {
    let tree = repo.head_commit()?.tree()?;

    let patterns = get_patterns(repo)?;

    // assuming this is relative to the root.
    let paths_changed: Vec<_> = tree
        .iter()
        .map(|result| -> Result<_> { Ok(result?) })
        // keep only if the thing is a file.
        .filter_ok(is_entry_file)
        // keep only if it uses our filter.
        .filter_ok(|entry| patterns_contains_entry(&patterns, entry))
        .map_ok_then(|entry| Ok(entry.filename().to_path()?.to_path_buf()))
        .try_collect()?;

    Ok(paths_changed)
}

impl PostCommit {
    /// The script that runs during the post-commit stage via a git hook.
    ///
    /// Here's the new approach now we understand how git works.
    ///
    /// When a files are commited, we check to see if any of the files
    /// match patterns specified in `.gitattributes#"filter.split -text"`.
    ///
    /// On files that match, we need to get all the parts that were created when checking in,
    /// and ensure they're stored in git as objects so we can retrieve them later.
    ///
    /// We'll currently store each file as a tree that contains parts stored as blobs.
    /// To ensure we can send these to the remote, we need to create a reference to the tree too.
    ///
    /// 1. Get patterns.
    /// 2. Get files.
    /// 3. Filter for files that match the pattern.
    /// 4. For each file, create the blobs and put them in a tree.
    /// 5. Create a reference to the tree as `refs/gfs/:tree-id`.
    /// 6. Revert previous commit.
    /// 7. Replace the contents of the file with a pointer to the reference with some metadata.
    /// 8. Commit
    pub fn run(repo: &Repository) -> Result<()> {
        let files = large_files_changed(repo)?;

        // nice!
        let files: Vec<_> = files
            .into_iter()
            .map(|filepath| create_git_parts_pattern(repo, &filepath))
            .map_ok_then(|pattern| Ok(glob(&pattern)?))
            .map_ok_then(|filepaths_parts| -> Result<()> {
                let entries: Vec<_> = filepaths_parts
                    .map(|filepath_part| -> Result<_> {
                        let part = filepath_part?;
                        let entry = Entry {
                            filename: part.to_str().unwrap().into(),
                            mode: EntryKind::Blob.into(),
                            oid: repo.write_blob_stream(File::open(part)?)?.into(),
                        };
                        Ok(entry)
                    })
                    .try_collect()?;

                let tree = Tree { entries };

                let tree_id = repo.write_object(&tree)?;
                let name = format!("/refs/split/{}", &tree_id);
                let ref_id = repo
                    .reference(name, tree_id, PreviousValue::MustNotExist, "")?
                    .id();

                // now we need to write the pointer using the reference
                // into the main file
                //
                // need to remove file from latest commit

                // Bro I can actually merge N+3 amount of commits?

                Ok(())
            })
            .try_collect()?;

        Ok(())
    }
}

struct Pointer {
    /// The reference that points to the tree of parts with this file
    /// sha256:<hash>
    reference: String,

    /// What format the pointer is, in case it were to change over time.
    version: String,
}
