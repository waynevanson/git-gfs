use crate::{create_gfs_ref, pointer::Pointer, splitter::Splitter, traits::MapOkThen};
use anyhow::{anyhow, Result};
use bytesize::ByteSize;
use gix::{
    objs::{
        tree::{Entry, EntryKind},
        Tree,
    },
    refs::transaction::PreviousValue,
    Id, Repository,
};
use itertools::Itertools;
use scopeguard::defer_on_unwind;
use std::{
    fs::{create_dir_all, remove_dir_all, File},
    io::{copy, stdout, Write},
    path::{Path, PathBuf},
};

pub fn clean(repo: &Repository, filepath: PathBuf, size: ByteSize) -> Result<()> {
    let parts_file_dir = repo.path().join("parts").join(&filepath);
    split(filepath, &parts_file_dir, size)?;

    let reference_id = create_reference_id(repo, &parts_file_dir)?;

    let pointer = Pointer::from(reference_id.to_string());
    write_pointer(&pointer)?;

    Ok(())
}

fn write_pointer(pointer: &Pointer) -> Result<()> {
    let contents = pointer.try_to_string()?;

    // write to file
    stdout().write_all(contents.as_bytes())?;

    Ok(())
}

/// Split a file a `filepath` into pieces (ie. `aaaa`, `aaab`, `aaac`)
/// into pieces with the max size of `size` into the `target_directory`
fn split(
    source_file: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
    size: ByteSize,
) -> Result<()> {
    defer_on_unwind!(remove_dir_all(&target_dir).unwrap());
    create_dir_all(&target_dir)?;

    let mut writer = Splitter::new(&target_dir, size.as_u64(), 4);
    let mut reader = File::open(&source_file)?;

    copy(&mut reader, &mut writer)?;

    Ok(())
}

fn create_reference_id(repo: &Repository, parts_file_dir: impl AsRef<Path>) -> Result<Id<'_>> {
    let parts = create_parts_as_entries(repo, parts_file_dir)?;
    let tree_id = create_tree_id(repo, parts)?;
    let id = create_tree_reference_id(repo, tree_id)?;
    Ok(id)
}

/// Read the content of a file part from `part_path`,
/// creates a blob in our reference namespaces
/// and returns and entry (for use in creating a tree).
fn create_entry_from_part(repo: &Repository, part_path: &Path) -> Result<Entry> {
    let file = File::open(part_path)?;

    let filename = part_path
        .to_str()
        .ok_or_else(|| anyhow!("Expected file path to be a string"))?
        .into();

    // todo: delete blob when unwinding
    let oid = repo.write_blob_stream(file)?.into();

    let entry = Entry {
        filename,
        mode: EntryKind::Blob.into(),
        oid,
    };

    Ok(entry)
}

fn create_parts_as_entries(
    repo: &Repository,
    parts_file_dir: impl AsRef<Path>,
) -> Result<Vec<Entry>> {
    let parts = std::fs::read_dir(&parts_file_dir)?
        .map(|result| -> Result<_> { Ok(result?) })
        .map_ok(|dir_entry| dir_entry.path())
        .map_ok_then(|part_path| create_entry_from_part(repo, &part_path))
        .try_collect()?;

    Ok(parts)
}

fn create_tree_reference_id<'repo>(repo: &'repo Repository, tree_id: Id) -> Result<Id<'repo>> {
    let name = create_gfs_ref(tree_id);
    let ref_id = repo.reference(name, tree_id, PreviousValue::Any, "")?.id();
    Ok(ref_id)
}

fn create_tree_id(repo: &Repository, entries: Vec<Entry>) -> Result<Id<'_>> {
    let tree = Tree { entries };
    // todo: delete tree when unwinding
    let tree_id = repo.write_object(&tree)?;
    Ok(tree_id)
}
