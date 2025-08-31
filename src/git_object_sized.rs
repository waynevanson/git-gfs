use crate::git_object_id::GitObjectId;
use std::io::{Error, Result};

pub struct GitObjectSized {
    object_id: GitObjectId,
    size: u32,
}

impl GitObjectSized {
    pub fn from_contents(contents: impl AsRef<[u8]>) -> Result<Self> {
        let git_object = GitObjectId::from_contents(contents)?;
        Self::try_from(git_object)
    }

    pub fn size(&self) -> &u32 {
        &self.size
    }

    pub fn object_id(&self) -> &GitObjectId {
        &self.object_id
    }
}

impl TryFrom<GitObjectId> for GitObjectSized {
    type Error = Error;

    fn try_from(object_id: GitObjectId) -> Result<Self> {
        let size = object_id.size()?;
        Ok(Self { object_id, size })
    }
}
