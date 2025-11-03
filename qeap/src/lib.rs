pub mod error;
mod handle;
mod impls;
pub mod transform;

pub use handle::Handle;
// might think about adding different formats... need to make sure, if behind features, that they are additive

extern crate qeap_macro;

pub use qeap_macro::Bundle;
pub use qeap_macro::Qeap;
pub use qeap_macro::scoped;

pub type QeapResult<T> = Result<T, error::Error>;

pub trait Qeap {
    fn load() -> QeapResult<Self>
    where
        Self: Sized;
    fn save(&self) -> QeapResult<()>;
}

pub trait Qeaper {
    type Output;
    fn init(&self) -> QeapResult<()>;
    fn load(&self, name: &str) -> QeapResult<Self::Output>;
    fn save(&self, data: &Self::Output, name: &str) -> QeapResult<()>;
}

pub trait Bundle: Qeap {}
