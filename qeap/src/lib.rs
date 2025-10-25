pub mod error;
pub mod load;
pub mod save;

// might think about adding different formats... need to make sure, if behind features, that they are additive

extern crate qeap_macro;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

pub use qeap_macro::Qeap;
pub use qeap_macro::scoped;

use crate::error::SaveError;

pub type QeapResult<T> = Result<T, error::Error>;
pub type QeapSaveResult<T> = Result<T, error::SaveError>;

pub trait Qeap {
    const FILE_NAME: &str;

    fn load() -> QeapResult<Self>
    where
        Self: Sized;
    fn save(&self) -> QeapSaveResult<()>;
}

impl<T: Qeap> Qeap for Mutex<T> {
    const FILE_NAME: &str = T::FILE_NAME;

    fn load() -> QeapResult<Self>
    where
        Self: Sized,
    {
        let data = T::load()?;
        Ok(Mutex::new(data))
    }

    fn save(&self) -> QeapSaveResult<()> {
        let guard = self.lock().map_err(|_| SaveError::LockPoison)?;
        (&*guard).save()
    }
}

impl<T: Qeap> Qeap for RefCell<T> {
    const FILE_NAME: &str = T::FILE_NAME;

    fn load() -> QeapResult<Self>
    where
        Self: Sized,
    {
        let data = T::load()?;
        Ok(RefCell::new(data))
    }

    fn save(&self) -> QeapSaveResult<()> {
        let data = self.borrow();

        (&*data).save()
    }
}

impl<T: Qeap> Qeap for RwLock<T> {
    const FILE_NAME: &str = T::FILE_NAME;

    fn load() -> QeapResult<Self>
    where
        Self: Sized,
    {
        let data = T::load()?;
        Ok(RwLock::new(data))
    }

    fn save(&self) -> QeapSaveResult<()> {
        let guard = self.write().map_err(|_| SaveError::LockPoison)?;
        (&*guard).save()?;
        Ok(())
    }
}

impl<T: Qeap> Qeap for Rc<T> {
    const FILE_NAME: &str = T::FILE_NAME;

    fn load() -> QeapResult<Self>
    where
        Self: Sized,
    {
        let data = T::load()?;
        Ok(Rc::new(data))
    }

    fn save(&self) -> QeapSaveResult<()> {
        T::save(&*self)
    }
}

impl<T: Qeap> Qeap for Arc<T> {
    const FILE_NAME: &str = T::FILE_NAME;

    fn load() -> QeapResult<Self>
    where
        Self: Sized,
    {
        let data = T::load()?;
        Ok(Arc::new(data))
    }

    fn save(&self) -> QeapSaveResult<()> {
        T::save(&*self)
    }
}

pub trait Handle {
    fn new_handle(&self) -> Self
    where
        Self: Sized;
}

impl<T> Handle for Rc<T> {
    #[inline]
    fn new_handle(&self) -> Self
    where
        Self: Sized,
    {
        Rc::clone(self)
    }
}

impl<T> Handle for Arc<T> {
    #[inline]
    fn new_handle(&self) -> Self
    where
        Self: Sized,
    {
        Arc::clone(self)
    }
}
