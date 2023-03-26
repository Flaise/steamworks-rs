use steamworks_sys as sys;

/// Used to separate client and game server modes
pub unsafe trait Manager {
    unsafe fn get_pipe() -> sys::HSteamPipe;
}

/// Manages keeping the steam api active for clients
pub struct ClientManager {
    pub(crate) _priv: (),
}

unsafe impl Manager for ClientManager {
    unsafe fn get_pipe() -> sys::HSteamPipe {
        sys::SteamAPI_GetHSteamPipe()
    }
}

impl Drop for ClientManager {
    fn drop(&mut self) {
        unsafe {
            sys::SteamAPI_Shutdown();
        }
    }
}

/// Manages keeping the steam api active for servers
pub struct ServerManager {
    pub(crate) _priv: (),
}

unsafe impl Manager for ServerManager {
    unsafe fn get_pipe() -> sys::HSteamPipe {
        sys::SteamGameServer_GetHSteamPipe()
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        unsafe {
            sys::SteamGameServer_Shutdown();
        }
    }
}
