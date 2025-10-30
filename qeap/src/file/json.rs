use std::{
    fs::{File, OpenOptions},
    marker::PhantomData,
};

use serde::{Deserialize, Serialize};

use crate::file::{FileError, FileFormat};

pub struct Json<T>(PhantomData<T>);
impl<T> FileFormat for Json<T>
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    type Data = T;

    fn serialize_to(data: &Self::Data, path: &std::path::Path) -> Result<(), super::FileError> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| FileError::Open(path.display().to_string(), e))?;

        serde_json::to_writer(file, data).map_err(|e| FileError::parse(&path, "json", e))
    }

    fn deserialize_from(path: &std::path::Path) -> Result<Self::Data, super::FileError> {
        let file = File::open(path).map_err(|e| FileError::Open(path.display().to_string(), e))?;

        serde_json::from_reader(file).map_err(|e| FileError::parse(path, "json", e))
    }

    fn ext() -> &'static str {
        "json"
    }
}
