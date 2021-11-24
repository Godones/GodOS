use core::cell::{RefCell, RefMut};

pub struct MyRefCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for MyRefCell<T> {}

impl<T> MyRefCell<T> {
    pub fn get_mut(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
    pub fn new(val: T) -> Self {
        Self {
            inner: RefCell::new(val),
        }
    }
}
