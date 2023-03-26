use std::sync::Mutex;
use steamworks_sys as sys;
use crate::manager::Manager;
use crate::{Callbacks, NetworkingSocketsData};

pub(crate) struct Inner {
    manager: Box<dyn Manager + Send + Sync>,
    pub callbacks: Mutex<Callbacks>,
    pub networking_sockets_data: Mutex<NetworkingSocketsData>,
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

impl Inner {
    pub fn new<M: Manager + Send + Sync + 'static>(manager: M) -> Inner {
        Inner {
            manager: Box::new(manager),
            callbacks: Mutex::new(Callbacks::default()),
            networking_sockets_data: Mutex::new(NetworkingSocketsData::default()),
        }
    }

    pub unsafe fn get_pipe(&self) -> sys::HSteamPipe {
        self.manager.get_pipe()
    }
}
