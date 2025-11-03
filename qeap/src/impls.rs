use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    Qeap, QeapResult, Qeaper,
    error::{Error, SimpleErr},
};

impl<T: Qeap> Qeap for Mutex<T> {
    fn load() -> QeapResult<Self>
    where
        Self: Sized,
    {
        let data = T::load()?;
        Ok(Mutex::new(data))
    }

    fn save(&self) -> QeapResult<()> {
        let guard = self
            .lock()
            .map_err(|e| Error::save(SimpleErr(e.to_string())))?;
        (&*guard).save()
    }
}

impl<T: Qeap> Qeap for RefCell<T> {
    fn load() -> QeapResult<Self>
    where
        Self: Sized,
    {
        let data = T::load()?;
        Ok(RefCell::new(data))
    }

    fn save(&self) -> QeapResult<()> {
        let data = self.borrow();

        (&*data).save()
    }
}

impl<T: Qeap> Qeap for RwLock<T> {
    fn load() -> QeapResult<Self>
    where
        Self: Sized,
    {
        let data = T::load()?;
        Ok(RwLock::new(data))
    }

    fn save(&self) -> QeapResult<()> {
        let guard = self
            .write()
            .map_err(|e| Error::save(SimpleErr(e.to_string())))?;
        (&*guard).save()?;
        Ok(())
    }
}

impl<T: Qeap> Qeap for Rc<T> {
    fn load() -> QeapResult<Self>
    where
        Self: Sized,
    {
        let data = T::load()?;
        Ok(Rc::new(data))
    }

    fn save(&self) -> QeapResult<()> {
        T::save(&*self)
    }
}

impl<T: Qeap> Qeap for Arc<T> {
    fn load() -> QeapResult<Self>
    where
        Self: Sized,
    {
        let data = T::load()?;
        Ok(Arc::new(data))
    }

    fn save(&self) -> QeapResult<()> {
        T::save(&*self)
    }
}

impl<T> Qeaper for &T
where
    T: Qeaper,
{
    type Output = T::Output;

    fn init(&self) -> QeapResult<()> {
        (&**self).init()
    }

    fn load(&self, name: &str) -> QeapResult<Self::Output> {
        (&**self).load(name)
    }

    fn save(&self, data: &Self::Output, name: &str) -> QeapResult<()> {
        (&**self).save(data, name)
    }
}
