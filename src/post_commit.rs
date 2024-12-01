use crate::flatten_ok_then::FlattenOkThen;
use crate::map_ok_then::MapOkThen;
use anyhow::{anyhow, Result};
use clap::Parser;
use gix::{
    attrs::search::Match,
    glob::{wildmatch, Pattern},
    objs::tree::EntryKind,
    refs::transaction::PreviousValue,
    Repository,
};
use glob::glob;
use itertools::Itertools;
use std::collections::HashSet;

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

impl PostCommit {
    pub fn run(repo: &Repository) -> Result<()> {
        // commit each part into refs/split/<commit-hash> from the current commit (with the file)
        // remove the previous part? Maybe we need it so the file is actually added. tow parents, makes sense.
        // merge the last into this.

        let committed = repo.head_commit()?;

        // does this commit contain changes in split parts?
        let tree = committed.tree()?;

        let patterns = get_patterns(repo)?;

        // assuming this is relative to the root.
        let paths_changed = tree
            .iter()
            .map(|result| -> Result<_> { Ok(result?) })
            // keep only if the thing is a file.
            .filter_ok(|entry| {
                matches!(
                    entry.mode().kind(),
                    EntryKind::Blob | EntryKind::BlobExecutable
                )
            })
            // keep only if it uses our filter.
            .filter_ok(|entry| {
                patterns.iter().any(|pattern| {
                    pattern.matches(entry.filename(), wildmatch::Mode::NO_MATCH_SLASH_LITERAL)
                })
            })
            // file names only
            .flatten_ok_then(|entry| {
                let path = repo
                    .path()
                    .join("parts")
                    .join(format!("{}.part.*", entry.filename()));

                let pattern = path
                    .to_str()
                    .ok_or_else(|| anyhow!("Expected to create pattern"))?;

                let files = glob(pattern)?.map(|result| -> Result<_> { Ok(result?) });

                let ding = files.map_ok_then(|filepath| {
                    let id = repo.commit("", "", committed.id(), Some(committed.id()))?;

                    let reference_name = format!("/refs/split/{}", ":commit-id");
                    // create reference after not before.
                    let reference =
                        repo.reference(reference_name, id, PreviousValue::MustNotExist, "")?;
                    Ok(())
                });

                // create ref

                Ok(ding)
            });

        // for each filepath, read all the parts and add one per commit.
        // creat refs for it locally?

        // read split files.

        // how to commit?

        Ok(())
    }
}
