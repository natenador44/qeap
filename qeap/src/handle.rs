use std::{rc::Rc, sync::Arc};

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
