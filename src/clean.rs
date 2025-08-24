use crate::CleanConfig;
use anyhow::{Error, Result};
use fastcdc::v2020::StreamCDC;
use gix::Repository;
use itertools::Itertools;
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    fs::{write, File},
    io::{stdout, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

pub struct CleanOptions {
    pub min_size: u32,
    pub max_size: u32,
    pub avg_size: u32,
}

impl TryFrom<CleanConfig> for CleanOptions {
    type Error = Error;

    fn try_from(config: CleanConfig) -> anyhow::Result<Self> {
        Ok(Self {
            min_size: u32::try_from(config.min_size.0)?,
            avg_size: u32::try_from(config.avg_size.0)?,
            max_size: u32::try_from(config.max_size.0)?,
        })
    }
}

pub fn clean(_repo: &Repository, filepath: PathBuf, options: CleanOptions) -> Result<()> {
    let (file_names_ordered, file_name_to_content) = split_into_chunks(&filepath, options)?;

    // write to working dir
    let base = PathBuf::from_str(".gfs/contents")?;

    let mut paths = Vec::<PathBuf>::with_capacity(file_name_to_content.len());

    // todo: par_iter
    for (file_name, contents) in file_name_to_content {
        let path = base.join(file_name);
        write(&path, contents)?;

        paths.push(path);
    }

    // git add
    Command::new("git")
        .args(
            ["add"]
                .into_iter()
                .map(|a| a.to_string())
                .chain(paths.into_iter().map(|a| a.display().to_string())),
        )
        .output()?;

    // write to stdout for git clean
    let pointer_file = file_names_ordered.iter().join("\n");
    stdout().write_all(pointer_file.as_bytes())?;

    Ok(())
}

fn split_into_chunks(
    source_file: impl AsRef<Path>,
    options: CleanOptions,
) -> Result<(Vec<String>, HashMap<String, Vec<u8>>)> {
    let source = File::open(&source_file)?;

    let iter = StreamCDC::new(source, options.min_size, options.avg_size, options.max_size);

    let mut files = HashMap::<String, Vec<u8>>::new();
    let mut file_names_ordered = Vec::<String>::new();

    for item in iter {
        let chunk = item?;

        let sha: String = Sha1::digest(&chunk.data).to_vec().try_into()?;

        file_names_ordered.push(sha.clone());
        files.insert(sha, chunk.data);
    }

    Ok((file_names_ordered, files))
}
