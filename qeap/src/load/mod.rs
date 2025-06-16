use std::{fs::File, path::Path};

use serde::Deserialize;

use crate::{QeapResult, error::Error};

pub fn json<P, T>(path: P) -> QeapResult<T>
where
    P: AsRef<Path>,
    T: for<'de> Deserialize<'de>,
{
    let path = path.as_ref();
    let file = File::open(path).map_err(|e| Error::Open(path.to_owned(), e))?;

    serde_json::from_reader(file).map_err(|e| Error::JsonParse(path.to_owned(), e))
}
