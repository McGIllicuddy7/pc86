use std::{
    borrow::{Borrow, BorrowMut},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub struct Ptr<T: ?Sized> {
    ptr: Arc<dyn PtrInner<T>>,
}
trait PtrInner<T: ?Sized> {
    fn get<'a>(&'a self) -> RwLockReadGuard<'a, T>;
    fn get_mut<'a>(&'a self) -> RwLockWriteGuard<'a, T>;
    fn mark_destroy(&self);
}

pub struct HeapValue<T> {
    value: RwLock<Option<T>>,
}

impl<T, U> PtrInner<U> for HeapValue<T>
where
    T: Borrow<U> + BorrowMut<U>,
{
    fn get<'a>(&'a self) -> RwLockReadGuard<'a, U> {
        self.value.read()
    }

    fn get_mut<'a>(&'a self) -> RwLockWriteGuard<'a, U> {
        todo!()
    }

    fn mark_destroy(&self) {
        todo!()
    }
}

pub struct PtrGuard<T> {}
