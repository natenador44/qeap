use std::{
    io,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub mod json;

use crate::{PersistenceMechanism, QeapResult, error::Error, transform::DynError};

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("failed to open file '{0}': {1}")]
    Open(String, io::Error),
    #[error("failed to parse '{0}' as {1}: {2}")]
    Parse(String, &'static str, DynError),
}

impl FileError {
    fn parse<E>(path: &Path, format: &'static str, cause: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self::Parse(
            path.display().to_string(),
            format,
            Box::new(cause) as DynError,
        )
    }
}

pub trait FileFormat {
    type Data: Serialize + for<'a> Deserialize<'a>;
    fn serialize_to(data: &Self::Data, path: &Path) -> Result<(), FileError>;
    fn deserialize_from(path: &Path) -> Result<Self::Data, FileError>;
    fn ext() -> &'static str;
}

pub struct FileMechanism<F> {
    root_dir: PathBuf,
    _phantom: PhantomData<F>,
}

impl<F> FileMechanism<F>
where
    F: FileFormat,
{
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
            _phantom: PhantomData,
        }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn file_path(&self, name: &str) -> PathBuf {
        let mut file_path = self.root_dir.join(name.to_lowercase());
        file_path.set_extension(F::ext());
        file_path
    }
}

impl<T, F> PersistenceMechanism for FileMechanism<F>
where
    F: FileFormat<Data = T>,
    T: Default,
{
    type Output = T;

    fn load(&self, name: &str) -> QeapResult<Self::Output> {
        let file_path = self.file_path(name);
        if !file_path.exists() {
            let data = T::default();
            self.save(&data, name)?;
            Ok(data)
        } else {
            F::deserialize_from(&file_path).map_err(Error::load)
        }
    }

    fn save(&self, data: &Self::Output, name: &str) -> QeapResult<()> {
        F::serialize_to(data, &self.file_path(name)).map_err(Error::save)
    }

    fn init(&self) -> QeapResult<()> {
        std::fs::create_dir_all(&self.root_dir).map_err(Error::init)
    }
}
