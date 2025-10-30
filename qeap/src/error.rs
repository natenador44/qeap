use std::{fmt::Display, io, path::PathBuf};

use crate::transform::DynError;

pub type FlattenErasedError = FlattenedError<Box<dyn std::error::Error>>;

#[derive(Debug, thiserror::Error)]
pub enum FlattenedError<E> {
    #[error(transparent)]
    Qeap(#[from] Error),
    #[error(transparent)]
    User(E),
}

#[derive(Debug, thiserror::Error)]
#[error("failed to {ty} qeap data: {cause}")]
pub struct Error {
    cause: DynError,
    ty: ErrorType,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct SimpleErr(pub String);

impl Error {
    pub fn load<E>(err: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            cause: Box::new(err) as DynError,
            ty: ErrorType::Load,
        }
    }

    pub fn save<E>(err: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            cause: Box::new(err) as DynError,
            ty: ErrorType::Save,
        }
    }

    pub fn init<E>(err: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            cause: Box::new(err) as DynError,
            ty: ErrorType::Init,
        }
    }
}

#[derive(Debug)]
pub enum ErrorType {
    Load,
    Save,
    Init,
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::Load => write!(f, "load"),
            ErrorType::Save => write!(f, "save"),
            ErrorType::Init => write!(f, "init"),
        }
    }
}
