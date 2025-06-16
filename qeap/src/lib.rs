pub mod error;
pub mod load;
pub mod save;

// might think about adding different formats... need to make sure, if behind features, that they are additive

extern crate qeap_macro;
pub use qeap_macro::Qeap;
pub use qeap_macro::scoped;

pub type QeapResult<T> = Result<T, error::Error>;
pub type QeapSaveResult<T> = Result<T, error::SaveError>;

pub trait Qeap {
    const FILE_NAME: &str;

    fn load() -> QeapResult<Self>
    where
        Self: Sized;
    fn save(&self) -> QeapSaveResult<()>;
}
