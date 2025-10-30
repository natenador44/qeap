use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    marker::PhantomData,
};

use serde::{Deserialize, Serialize};

use crate::file::{FileError, FileFormat};

pub struct Toml<T>(PhantomData<T>);
impl<T> FileFormat for Toml<T>
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    type Data = T;

    fn serialize_to(
        data: &Self::Data,
        path: &std::path::Path,
    ) -> Result<(), super::file::FileError> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| FileError::open(path, e))?;

        let as_str = toml::to_string(data).map_err(|e| FileError::parse(path, Self::ext(), e))?;
        let mut writer = BufWriter::new(file);
        writer
            .write_all(as_str.as_bytes())
            .map_err(|e| FileError::write(&path, e))
    }

    fn deserialize_from(path: &std::path::Path) -> Result<Self::Data, super::file::FileError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| FileError::Open(path.display().to_string(), e))?;

        toml::from_str(&content).map_err(|e| FileError::parse(path, Self::ext(), e))
    }

    fn ext() -> &'static str {
        "json"
    }
}
