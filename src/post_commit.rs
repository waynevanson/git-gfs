use crate::{pointer::Pointer, REFS_NAMESPACE};
use anyhow::{anyhow, Result};
use clap::Parser;
use gix::{
    attrs::search::Match,
    bstr::{BString, ByteSlice},
    glob::{wildmatch::Mode, Pattern},
    hashtable::HashSet,
    object::tree::diff::Action,
    refs::transaction::PreviousValue,
    Commit, Id, ObjectId, Repository,
};
use itertools::Itertools;
use std::fs::read_to_string;

#[derive(Parser)]
pub struct PostCommit;

impl PostCommit {
    pub fn run(repo: &mut Repository) -> Result<()> {
        // get branch the commit is on.
        // should always be here since this should be run post commit.
        let child_commit = repo.head_commit()?;

        // The commit before our commit, where we can merge stuff.
        // We have no parents, then we early return so we don't run this on merge commits
        let Some(parent_id) = get_parent_id(&child_commit)? else {
            return Ok(());
        };

        // create reference to commit for use later during merge
        let name = format!("{REFS_NAMESPACE}/commits/{}", child_commit.id());
        let commit_reference_id = repo
            .reference(name, child_commit.id(), PreviousValue::Any, "")?
            .id();

        // git reset --hard HEAD~1
        // TODO: move this down the stack.
        let mut branch = repo
            .head()?
            .try_into_referent()
            .ok_or_else(|| anyhow!("Expected HEAD to be attached to a branch"))?;

        branch.set_target_id(parent_id, "")?;

        let patterns = get_patterns(repo)?;

        let parent_commit = repo.head_commit()?;

        assert_ne!(parent_commit.id(), child_commit.id());

        let child_tree = child_commit.tree()?;
        let parent_tree = parent_commit.tree()?;

        let paths = get_files_from_diff(&parent_tree, &child_tree)?;

        let child_object_id = commit_reference_id.object()?.id;

        let mut pointers: Vec<_> = get_tree_references(paths, &patterns)?;
        pointers.push(child_object_id);

        // read each file to get a pointer, to get the tree reference to add to the commits.
        let _merge_commit_id = repo.merge_base_octopus(pointers)?;
        Ok(())
    }
}

fn get_tree_references(paths: Vec<BString>, patterns: &HashSet<Pattern>) -> Result<Vec<ObjectId>> {
    paths
        .into_iter()
        // ensure files are our big files.
        .filter(|bstr| {
            patterns
                .iter()
                .any(|pattern| pattern.matches(bstr.as_ref(), Mode::NO_MATCH_SLASH_LITERAL))
        })
        // read pointer form file system: working directory.
        .map(|bstr| {
            let path = bstr.to_path()?;
            let data = read_to_string(path)?;
            let pointer = toml::from_str::<Pointer>(&data)?;
            let object_id = ObjectId::try_from(pointer.hash.as_bytes())?;
            Ok(object_id)
        })
        .try_collect()
}

fn get_files_from_diff(
    parent_tree: &gix::Tree<'_>,
    child_tree: &gix::Tree<'_>,
) -> Result<Vec<BString>> {
    // get files that have changed.
    let mut paths = vec![];
    let mut platform = parent_tree.changes()?;

    platform.for_each_to_obtain_tree(child_tree, |change| -> Result<Action> {
        use gix::object::tree::diff::Change::Deletion;

        if !matches!(change, Deletion { .. }) {
            paths.push(change.location().to_owned());
        }

        Ok(Action::Continue)
    })?;

    Ok(paths)
}

fn get_parent_id<'repo>(commit: &'repo Commit<'repo>) -> Result<Option<Id<'repo>>> {
    let mut parents = commit.parent_ids();

    let Some(parent) = parents.next() else {
        return Err(anyhow!("Expected head commit to have a parent"));
    };

    if parents.next().is_some() {
        return Ok(None);
    }

    Ok(Some(parent))
}

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
