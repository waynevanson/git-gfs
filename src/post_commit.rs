use crate::REFS_NAMESPACE;
use anyhow::{anyhow, Result};
use clap::Parser;
use gix::{refs::transaction::PreviousValue, Repository};

#[derive(Parser)]
pub struct PostCommit;

impl PostCommit {
    pub fn run(repo: &mut Repository) -> Result<()> {
        // get branch the commit is on.
        // should always be here since this should be run post commit.
        let commit = repo.head_commit()?;

        // The commit before our commit, where we can merge stuff.
        let mut parents = commit.parent_ids();
        let parent_id = match (parents.next(), parents.next()) {
            (Some(parent), None) => Ok(parent),
            (None, None) => Err(anyhow!("Expected head commit to have a parent")),
            // skip post commit, encountered merge commit (which has 2 or more parents)
            (Some(_), Some(_)) => return Ok(()),
            _ => unreachable!("Expected iterator to only produce Some before None"),
        }?;

        // create reference to commit for use later during merge
        let name = format!("{REFS_NAMESPACE}/commits/{}", commit.id());
        let commit_reference_id = repo
            .reference(name, commit.id(), PreviousValue::Any, "")?
            .id();

        // git reset --hard HEAD~1
        let mut branch = repo
            .head()?
            .try_into_referent()
            .ok_or_else(|| anyhow!("Expected HEAD to be attached to a branch"))?;
        branch.set_target_id(parent_id, "")?;

        // how to get all the trees?
        // There's no parent.
        // We could get all those in `refs/gfs/trees/*` and only get those that don't have children?
        // If we end up adding parents to those trees, we'll be able to get the children from the parent that
        // are in our custom refs.

        // git merge branch <commit-reference-id> ...trees
        // TODO: add the parts.
        let merge_commit_id = repo.merge_base_octopus(vec![commit_reference_id])?;

        Ok(())
    }
}
