use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    Qeap, QeapResult,
    error::{Error, SimpleErr},
};

impl<T: Qeap> Qeap for Mutex<T> {
    type Persistence = T::Persistence;

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

    fn create_persistence() -> Self::Persistence {
        T::create_persistence()
    }
}

impl<T: Qeap> Qeap for RefCell<T> {
    type Persistence = T::Persistence;

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

    fn create_persistence() -> Self::Persistence {
        T::create_persistence()
    }
}

impl<T: Qeap> Qeap for RwLock<T> {
    type Persistence = T::Persistence;

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

    fn create_persistence() -> Self::Persistence {
        T::create_persistence()
    }
}

impl<T: Qeap> Qeap for Rc<T> {
    type Persistence = T::Persistence;

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

    fn create_persistence() -> Self::Persistence {
        T::create_persistence()
    }
}

impl<T: Qeap> Qeap for Arc<T> {
    type Persistence = T::Persistence;

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

    fn create_persistence() -> Self::Persistence {
        T::create_persistence()
    }
}
