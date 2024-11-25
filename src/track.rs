use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use clap::Parser;
use git2::Repository;
use gix_attributes::{
    glob::Pattern,
    parse::{Kind, Lines},
    StateRef,
};
use itertools::Itertools;

#[derive(Parser)]
pub struct Track {
    #[arg(value_parser = pattern_from_str)]
    pattern: Pattern,
}

fn pattern_from_str(str: &str) -> Result<Pattern> {
    let pattern = Pattern::from_bytes(str.as_bytes()).ok_or_else(|| anyhow!("Expected "))?;
    Ok(pattern)
}

impl Track {
    pub fn track(&self, repo: &Repository) -> Result<()> {
        let attributes_path = get_attributes_path(repo)?;

        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&attributes_path)?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        let attributes = gix_attributes::parse(&contents);
        let contains_attr = attributes_has_gfs(attributes, self)?;

        if contains_attr {
            println!("Skipping, pattern already exists in file");
        } else {
            // append
            let contents = format!(r#"{} filter="gfs" -text{}"#, &self.pattern, '\n');

            file.write_all(contents.as_bytes())?;
        };

        // does attribute we're about to create exist?
        // if so, skip.
        // If not, append.

        Ok(())
    }
}

// todo: throw error at any point?
fn attributes_has_gfs(attributes: Lines<'_>, options: &Track) -> Result<bool> {
    let bool = attributes
        .filter_map_ok(|(kind, iter, _)| {
            let is_current_pattern = kind_is_pattern(&kind, &options.pattern);
            let is_filter_gfs = get_is_filter_gfs(iter, &options.pattern);

            Some(is_current_pattern && is_filter_gfs)
        })
        .any(|result| matches!(result, Ok(true)));

    Ok(bool)
}

fn get_is_filter_gfs(mut iter: gix_attributes::parse::Iter<'_>, pattern: &Pattern) -> bool {
    iter.any(|result| {
        if let Ok(assignment) = result {
            let is_filter = assignment.name.as_str() == "filter";
            let is_gfs = get_is_gfs(assignment.state, pattern);
            is_filter && is_gfs
        } else {
            false
        }
    })
}

fn get_is_gfs(state: StateRef<'_>, pattern: &Pattern) -> bool {
    if let StateRef::Value(bstr) = state {
        *bstr.as_bstr() == pattern.to_string()
    } else {
        false
    }
}

fn kind_is_pattern(kind: &Kind, pattern: &Pattern) -> bool {
    if let Kind::Pattern(left) = kind {
        left == pattern
    } else {
        false
    }
}

fn get_attributes_path(repo: &Repository) -> Result<PathBuf> {
    let attributes_path = repo
        .path()
        .parent()
        .ok_or_else(|| anyhow!("Expected the repository to have a parent path"))?
        .join(".gitattributes");

    Ok(attributes_path)
}
