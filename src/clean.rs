use crate::{map_ok_then::MapOkThen, splitter::Splitter};
use anyhow::{anyhow, Result};
use bytesize::ByteSize;
use clap::Parser;
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
    io::copy,
    path::{Path, PathBuf},
};

#[derive(Parser)]
pub struct Clean {
    filepath: PathBuf,

    #[arg(default_value_t = ByteSize::mb(50))]
    size: ByteSize,
}

impl Clean {
    fn parts_file_dir(&self, repo: &Repository) -> PathBuf {
        repo.path().join("parts").join(&self.filepath)
    }

    fn split(&self, parts_file_dir: impl AsRef<Path>) -> Result<()> {
        let mut writer = Splitter::new(&parts_file_dir, self.size.as_u64(), 4);
        let mut reader = File::open(&self.filepath)?;

        defer_on_unwind!(remove_dir_all(&parts_file_dir).unwrap());
        create_dir_all(&parts_file_dir)?;

        copy(&mut reader, &mut writer)?;

        Ok(())
    }

    pub fn run(&self, repo: &Repository) -> Result<()> {
        let parts_file_dir = self.parts_file_dir(repo);

        self.split(&parts_file_dir)?;
        let reference_id = create_reference(repo, &parts_file_dir)?;

        //  git add by adding to the index.

        Ok(())
    }
}

fn create_reference(repo: &Repository, parts_file_dir: impl AsRef<Path>) -> Result<Id<'_>> {
    let parts: Vec<_> = std::fs::read_dir(parts_file_dir)?
        .map(|result| -> Result<_> { Ok(result?) })
        .map_ok(|dir_entry| dir_entry.path())
        .map_ok_then(|part_path| create_entry_from_part(repo, &part_path))
        .try_collect()?;

    let tree_id = create_tree_id(repo, parts)?;

    let id = create_tree_reference_id(repo, tree_id)?;

    Ok(id)
}

fn create_tree_reference_id<'repo>(repo: &'repo Repository, tree_id: Id) -> Result<Id<'repo>> {
    let name = format!("/refs/split/{}", &tree_id);
    let ref_id = repo.reference(name, tree_id, PreviousValue::Any, "")?.id();
    Ok(ref_id)
}

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

fn create_tree_id(repo: &Repository, entries: Vec<Entry>) -> Result<Id<'_>> {
    let tree = Tree { entries };
    // todo: delete tree when unwinding
    let tree_id = repo.write_object(&tree)?;
    Ok(tree_id)
}
