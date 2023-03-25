use super::*;

use crate::sys;

use std::sync::{Arc, Weak};

pub unsafe trait Callback {
    const ID: i32;
    const SIZE: i32;
    unsafe fn from_raw(raw: *mut c_void) -> Self;
}

/// A handle that can be used to remove a callback
/// at a later point.
///
/// Removes the callback when dropped
pub struct CallbackHandle<M: Manager = ClientManager> {
    id: i32,
    inner: Weak<Inner<M>>,
}
unsafe impl<M: Manager> Send for CallbackHandle<M> {}

impl<M: Manager> Drop for CallbackHandle<M> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            match inner.callbacks.lock() {
                Ok(mut cb) => {
                    cb.callbacks.remove(&self.id);
                }
                Err(err) => {
                    eprintln!("error while dropping callback: {:?}", err);
                }
            }
        }
    }
}

pub(crate) unsafe fn register_callback<C, F, M: Manager>(
    inner: &Arc<Inner<M>>,
    mut f: F,
) -> CallbackHandle<M>
where
    C: Callback,
    F: FnMut(C) + Send + 'static,
{
    {
        let mut callbacks = inner.callbacks.lock().unwrap();
        callbacks.callbacks.insert(
            C::ID,
            Box::new(move |param| {
                let param = C::from_raw(param);
                f(param)
            }),
        );
    }
    CallbackHandle {
        id: C::ID,
        inner: Arc::downgrade(&inner),
    }
}

pub(crate) unsafe fn register_call_result<C, F, M: Manager>(
    inner: &Arc<Inner<M>>,
    api_call: sys::SteamAPICall_t,
    _callback_id: i32,
    f: F,
) where
    F: for<'a> FnOnce(&'a C, bool) + 'static + Send,
{
    let mut callbacks = inner.callbacks.lock().unwrap();
    callbacks.call_results.insert(
        api_call,
        Box::new(move |param, failed| f(&*(param as *const C), failed)),
    );
}
