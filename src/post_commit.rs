use crate::{create_gfs_ref, pointer::Pointer};
use anyhow::{anyhow, Result};
use gix::{
    attrs::search::Match,
    bstr::{BString, ByteSlice},
    glob::{wildmatch::Mode, Pattern},
    hashtable::HashSet,
    object::tree::diff::Action,
    refs::transaction::PreviousValue,
    Commit, Id, ObjectId, Reference, Repository, ThreadSafeRepository,
};
use itertools::Itertools;
use log::debug;
use scopeguard::defer_on_unwind;
use std::fs::read_to_string;

pub struct PostCommit {
    repo: Repository,
}

impl PostCommit {
    pub fn new() -> Result<Self> {
        let repo = ThreadSafeRepository::open(".")?.to_thread_local();
        let post_commit = Self { repo };
        Ok(post_commit)
    }

    pub fn git_post_commit(&mut self) -> Result<()> {
        // get branch the commit is on.
        // should always be here since this should be run post commit.
        let child_commit = self.repo.head_commit()?;

        debug!("child_commit: {child_commit:?}");

        // The commit before our commit, where we can merge stuff.
        // We have no parents, then we early return so we don't run this on merge commits
        let Some(parent_id) = get_parent_id(&child_commit)? else {
            return Ok(());
        };

        debug!("parent_commit: {parent_id}");

        // create reference to commit for use later during merge
        let name = create_gfs_ref(child_commit.id());
        let commit_reference_id = self
            .repo
            .reference(name, child_commit.id(), PreviousValue::Any, "")?
            .id();

        debug!("commit_reference_id:{commit_reference_id}");

        // git reset --hard HEAD~1
        // TODO: move this down the stack.
        let mut branch = self.head_branch()?;

        branch.set_target_id(parent_id, "")?;
        defer_on_unwind!(branch.set_target_id(child_commit.id(), "").unwrap(););

        debug!("parent_commit: {parent_id}");

        let patterns = get_patterns(&self.repo)?;
        debug!("patterns:{patterns:?}");

        let parent_commit = self.repo.head_commit()?;

        debug!("new_parent_commit: {parent_commit:?}");

        assert_ne!(parent_commit.id(), child_commit.id());

        let child_tree = child_commit.tree()?;
        let parent_tree = parent_commit.tree()?;

        let paths = get_non_deleted_files_from_diff(&parent_tree, &child_tree)?;

        let child_object_id = commit_reference_id.object()?.id;

        let mut pointers: Vec<_> = get_tree_references(paths, &patterns)?;
        pointers.push(child_object_id);

        debug!("pointers:{pointers:?}");

        // read each file to get a pointer, to get the tree reference to add to the commits.
        let merge_commit_id = self.repo.merge_base_octopus(pointers)?;
        debug!("merged:{merge_commit_id:?}");

        // store this merge commit in our refs
        let name = create_gfs_ref(merge_commit_id);
        self.repo
            .reference(name, merge_commit_id, PreviousValue::Any, "")?;

        // revert repo back to the original HEAD now merge ref is stored.

        let mut branch = self.head_branch()?;
        branch.set_target_id(child_object_id, "")?;

        debug!("branch_reset:{child_object_id}");

        Ok(())
    }

    fn head_branch(&self) -> Result<Reference<'_>> {
        self.repo
            .head()?
            .try_into_referent()
            .ok_or_else(|| anyhow!("Expected HEAD to be attached to a branch"))
    }
}

/// Gets the object ids that are reference to trees containing parts of split files.
/// The input `paths` are assumed to be from a diff, so we're not reading files in the work tree.
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
            let pointer = serde_json::from_str::<Pointer>(&data)?;
            let object_id = ObjectId::try_from(pointer.hash().as_bytes())?;
            Ok(object_id)
        })
        .try_collect()
}

/// Retrieves a list of files that have been added, modified or rewritten.
/// It does not include deleted files.
fn get_non_deleted_files_from_diff(
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

/// Gets the parent id of a commit.
///
/// Returns `Some` when there is only 1 parent, and none when there is more than 1.
///
/// ## Error
///
/// 1. When there is no parent.
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

/// Gets the patterns used by `git-gfs`.
///
/// When one of the patterns is used to match against a path and it returns true,
/// that path (file) should be managed by `git-gfs`.
fn get_patterns(repo: &Repository) -> Result<HashSet<Pattern>> {
    let worktree = repo
        .worktree()
        .ok_or_else(|| anyhow!("Expected to find work tree"))?;

    let attributes = worktree.attributes(None)?.attribute_matches();

    // all patterns contain our custom filter=split -text
    let patterns = attributes
        .iter()
        .filter(has_gfs_attributes_filter)
        .map(|r#match| r#match.pattern.to_owned())
        .collect::<HashSet<_>>();

    Ok(patterns)
}

/// Checks to see if a match derived from `.gitattributes`
/// matches the filter for `git-gfs`.
fn has_gfs_attributes_filter(r#match: &Match<'_>) -> bool {
    let is_filter = r#match.assignment.name.as_str() == "filter";

    let is_gfs = r#match.assignment.state.as_bstr().map(|bstr| bstr == "gfs");

    matches!((is_filter, is_gfs), (true, Some(true)))
}
