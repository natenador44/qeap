use std::{fs::OpenOptions, path::Path};

use serde::Serialize;

use crate::{QeapSaveResult, error::SaveError};

pub fn json<P, T>(path: P, value: &T) -> QeapSaveResult<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let path = path.as_ref();
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|e| SaveError::Open(path.to_owned(), e))?;

    serde_json::to_writer(file, value).map_err(|e| SaveError::JsonWrite(path.to_owned(), e))
}
