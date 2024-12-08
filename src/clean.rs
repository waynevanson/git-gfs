use crate::{create_gfs_ref, map_ok_then::MapOkThen, pointer::Pointer, splitter::Splitter};
use anyhow::{anyhow, Result};
use bytesize::ByteSize;
use gix::{
    objs::{
        tree::{Entry, EntryKind},
        Tree,
    },
    refs::transaction::PreviousValue,
    Id, Repository, ThreadSafeRepository,
};
use itertools::Itertools;
use scopeguard::defer_on_unwind;
use std::{
    fs::{create_dir_all, remove_dir_all, File},
    io::{copy, stdout, Write},
    path::{Path, PathBuf},
};

pub struct Clean {
    filepath: PathBuf,
    size: ByteSize,
    repo: Repository,
    /// `.git/parts/<filepath>`
    parts_file_dir: PathBuf,
}

impl Clean {
    pub fn new(filepath: PathBuf, size: ByteSize) -> Result<Self> {
        let repo = ThreadSafeRepository::open(".")?.to_thread_local();
        let parts_file_dir = repo.path().join("parts").join(&filepath);

        let clean = Self {
            filepath,
            size,
            repo,
            parts_file_dir,
        };

        Ok(clean)
    }

    pub fn git_clean(&self) -> Result<()> {
        self.split()?;

        let reference_id = self.create_reference_id()?;

        let pointer = Pointer::V1 {
            hash: reference_id.to_string(),
        }
        .try_to_string()?;

        // print to stdout
        stdout().write_all(pointer.as_bytes())?;

        Ok(())
    }

    /// Split a file a `filepath` into pieces (ie. `aaaa`, `aaab`, `aaac`)
    /// into pieces with the max size of `size` into the `target_directory`
    fn split(&self) -> Result<()> {
        defer_on_unwind!(remove_dir_all(&self.parts_file_dir).unwrap());
        create_dir_all(&self.parts_file_dir)?;

        let mut writer = Splitter::new(&self.parts_file_dir, self.size.as_u64(), 4);
        let mut reader = File::open(&self.filepath)?;

        copy(&mut reader, &mut writer)?;

        Ok(())
    }

    /// Read the content of a file part from `part_path`,
    /// creates a blob in our reference namespaces
    /// and returns and entry (for use in creating a tree).
    fn create_entry_from_part(&self, part_path: &Path) -> Result<Entry> {
        let file = File::open(part_path)?;

        let filename = part_path
            .to_str()
            .ok_or_else(|| anyhow!("Expected file path to be a string"))?
            .into();

        // todo: delete blob when unwinding
        let oid = self.repo.write_blob_stream(file)?.into();

        let entry = Entry {
            filename,
            mode: EntryKind::Blob.into(),
            oid,
        };

        Ok(entry)
    }

    fn create_reference_id(&self) -> Result<Id<'_>> {
        let parts = self.create_parts_as_entries()?;
        let tree_id = create_tree_id(&self.repo, parts)?;
        let id = create_tree_reference_id(&self.repo, tree_id)?;
        Ok(id)
    }

    fn create_parts_as_entries(&self) -> Result<Vec<Entry>> {
        let parts = std::fs::read_dir(&self.parts_file_dir)?
            .map(|result| -> Result<_> { Ok(result?) })
            .map_ok(|dir_entry| dir_entry.path())
            .map_ok_then(|part_path| self.create_entry_from_part(&part_path))
            .try_collect()?;

        Ok(parts)
    }
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
