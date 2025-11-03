use std::{rc::Rc, sync::Arc};

pub trait Handle {
    type In<'a>;
    type Out;
    fn new_handle<'a>(s: Self::In<'a>) -> Self::Out;
}

impl<T> Handle for Rc<T>
where
    T: 'static,
{
    type In<'a> = &'a Rc<T>;

    type Out = Rc<T>;

    fn new_handle<'a>(s: Self::In<'a>) -> Self::Out {
        Rc::clone(s)
    }
}

impl<T> Handle for Arc<T>
where
    T: 'static,
{
    type In<'a> = &'a Arc<T>;

    type Out = Arc<T>;

    fn new_handle<'a>(s: Self::In<'a>) -> Self::Out {
        Arc::clone(s)
    }
}

impl<'a, T> Handle for &'a T {
    type In<'b> = &'a T;

    type Out = Self;

    fn new_handle<'b>(s: Self::In<'b>) -> Self::Out {
        s
    }
}

impl<'a, T> Handle for &'a mut T {
    type In<'b> = &'a mut T;

    type Out = Self;

    fn new_handle<'b>(s: Self::In<'b>) -> Self::Out {
        s
    }
}
