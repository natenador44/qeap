use std::convert::Infallible;

use crate::error;

pub type DynError = Box<dyn std::error::Error>;

pub trait IntoFlattenedResult<T, E> {
    fn into_flattened(self) -> Result<T, error::FlattenedError<E>>;
}

impl<T> IntoFlattenedResult<T, Infallible> for T {
    fn into_flattened(self) -> Result<T, error::FlattenedError<Infallible>> {
        Ok(self)
    }
}

impl<T, E> IntoFlattenedResult<T, E> for Result<T, E> {
    fn into_flattened(self) -> Result<T, error::FlattenedError<E>> {
        self.map_err(error::FlattenedError::User)
    }
}
