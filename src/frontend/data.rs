use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::err_handle::RuntimeFailure;
use crate::frontend::Context;

pub struct Data {
    // Data is wrapped in a Rc<RefCell<>>
    // The Rc is a reference counter which allows for multiple references to the inner value
    // This is necessary to prevent variables from being cloned every time they are read, .clone()
    // on an rc will just increment the reference count.
    // The RefCell allows for data mutation in multiple places
    handle: Rc<RefCell<DataKind>>
}

impl Data {
    pub fn new(data_kind: DataKind) -> Self {
        Self {
            handle: Rc::new(RefCell::new(data_kind)),
        }
    }
    pub fn borrow(&self, context: &Context) -> Result<Ref<DataKind>, RuntimeFailure> {
        match self.handle.try_borrow() {
            // Must return a Ref<T> here, returning a Ref<T>::deref() will error.
            // This happens because RefCell<T>::try_borrow returns a Ref<T> with the lifetime of the &self passed into
            // this method. Calling deref on that Ref<T> will be a borrow of a borrow, where the second borrow will
            // go out of scope when this function ends. The first borrow has the lifetime of &self and can be
            // returned, because the caller gave us &self and knows what the lifetime is.
            Ok(d) => Ok(d),
            Err(_) => Err(RuntimeFailure::BorrowError(
                "Cannot borrow a variable when it has a mutable reference in use".to_owned(),
                context.current_line
            )),
        }
    }
    pub fn borrow_mut(&self, context: &Context) -> Result<RefMut<DataKind>, RuntimeFailure> {
        match self.handle.try_borrow_mut() {
            Ok(d) => Ok(d),
            Err(_) => Err(RuntimeFailure::BorrowError(
                "Cannot borrow a variable mutably when it already has a reference in use"
                    .to_owned(),
                context.current_line
            )),
        }
    }
}

pub enum DataKind {

}
