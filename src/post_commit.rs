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
    Id, Repository,
};
use glob::{glob, Paths};
use itertools::Itertools;
use serde::Serialize;
use std::{
    collections::HashSet,
    fs::{self, File},
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

fn get_large_files_paths(repo: &Repository) -> Result<Vec<PathBuf>> {
    let tree = repo.head_commit()?.tree()?;

    let patterns = get_patterns(repo)?;

    // assuming this is relative to the root.
    let large_files_paths: Vec<_> = tree
        .iter()
        .map(|result| -> Result<_> { Ok(result?) })
        // keep only if the thing is a file.
        .filter_ok(is_entry_file)
        // keep only if it uses our filter.
        .filter_ok(|entry| patterns_contains_entry(&patterns, entry))
        .map_ok_then(|entry| Ok(entry.filename().to_path()?.to_path_buf()))
        .try_collect()?;

    Ok(large_files_paths)
}

fn get_unstaged_parts(repo: &Repository, filename: &Path) -> Result<Paths> {
    let pattern = create_git_parts_pattern(repo, filename)?;
    let parts = glob(&pattern)?;
    Ok(parts)
}

fn create_entry_from_part(repo: &Repository, part: &Path) -> Result<Entry> {
    let oid = repo.write_blob_stream(File::open(part)?)?.into();

    let entry = Entry {
        filename: part.to_str().unwrap().into(),
        mode: EntryKind::Blob.into(),
        oid,
    };
    Ok(entry)
}

fn create_entries(repo: &Repository, parts: Paths) -> Result<Vec<Entry>> {
    let entries: Vec<_> = parts
        .map(|result| -> Result<_> { Ok(result?) })
        .map_ok_then(|part| create_entry_from_part(repo, &part))
        .try_collect()?;

    Ok(entries)
}

fn create_tree_id(repo: &Repository, parts: Paths) -> Result<Id<'_>> {
    let entries = create_entries(repo, parts)?;
    let tree = Tree { entries };
    let tree_id = repo.write_object(&tree)?;
    Ok(tree_id)
}

fn create_reference<'repo>(repo: &'repo Repository, tree_id: Id) -> Result<Id<'repo>> {
    let name = format!("/refs/split/{}", &tree_id);
    let ref_id = repo
        .reference(name, tree_id, PreviousValue::MustNotExist, "")?
        .id();

    Ok(ref_id)
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
        let files = get_large_files_paths(repo)?;

        // nice!
        let files: Vec<_> = files
            .into_iter()
            .map(|filepath| get_unstaged_parts(repo, &filepath))
            .map_ok_then(|parts| -> Result<()> {
                let tree_id = create_tree_id(repo, parts)?;
                let ref_id = create_reference(repo, tree_id)?;

                let pointer = Pointer::from_sha(HashType::SHA256, ref_id.to_string());
                pointer.write_to_file("/fill/me/out/")?;

                // alright well I'm stuck here let's regroup later.
                let index = repo.index()?;

                // need to remove file from latest commit
                // replace the contents
                // Well I could just add the file using git add ?

                Ok(())
            })
            .try_collect()?;

        Ok(())
    }
}

#[derive(Serialize, Default)]
pub enum HashType {
    SHA1,
    #[default]
    SHA256,
}

#[derive(Serialize, Default)]
pub enum Version {
    #[default]
    One,
}

#[derive(Serialize)]
pub struct Pointer {
    hash_function: HashType,
    hash: String,
    version: Version,
}

impl Pointer {
    pub fn from_sha(hash_function: HashType, hash: String) -> Self {
        Self {
            hash,
            hash_function,
            version: Version::default(),
        }
    }

    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let contents = toml::to_string_pretty(&self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}
