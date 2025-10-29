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

pub trait IntoFlattenErasedResult<T, E> {
    fn into_flatten_erased(self) -> Result<T, error::FlattenedError<DynError>>;
}

impl<T> IntoFlattenErasedResult<T, Infallible> for T {
    fn into_flatten_erased(self) -> Result<T, error::FlattenedError<DynError>> {
        Ok(self)
    }
}

impl<T, E> IntoFlattenErasedResult<T, E> for Result<T, E>
where
    E: std::error::Error + 'static,
{
    fn into_flatten_erased(self) -> Result<T, error::FlattenedError<DynError>> {
        self.map_err(|e| error::FlattenedError::User(Box::new(e) as DynError))
    }
}
