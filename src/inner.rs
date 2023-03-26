use std::sync::Mutex;
use crate::{Manager, Callbacks, NetworkingSocketsData};

pub(crate) struct Inner<M: Manager> {
    _manager: M,
    pub callbacks: Mutex<Callbacks>,
    pub networking_sockets_data: Mutex<NetworkingSocketsData<M>>,
}

unsafe impl<M: Manager + Send + Sync> Send for Inner<M> {}
unsafe impl<M: Manager + Send + Sync> Sync for Inner<M> {}

impl<M: Manager> Inner<M> {
    pub fn new(manager: M) -> Inner<M> {
        Inner {
            _manager: manager,
            callbacks: Mutex::new(Callbacks::default()),
            networking_sockets_data: Mutex::new(NetworkingSocketsData::default()),
        }
    }
}
