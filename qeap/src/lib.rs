pub mod error;
pub mod load;
pub mod save;

// might think about adding different formats... need to make sure, if behind features, that they are additive

extern crate qeap_macro;
pub use qeap_macro::Qeap;

pub type QeapLoadResult<T> = Result<T, error::LoadError>;
pub type QeapSaveResult<T> = Result<T, error::SaveError>;

pub trait Qeap {
    const FILE_NAME: &str;

    fn load() -> QeapLoadResult<Self>
    where
        Self: Sized;
    fn save(&self) -> QeapSaveResult<()>;
}
