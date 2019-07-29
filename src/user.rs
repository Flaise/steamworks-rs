use super::*;
#[cfg(test)]
use serial_test_derive::serial;

/// Access to the steam user interface
pub struct User<Manager> {
    pub(crate) user: *mut sys::ISteamUser,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl <Manager> User<Manager> {
    /// Returns the steam id of the current user
    pub fn steam_id(&self) -> SteamId {
        unsafe {
            SteamId(sys::SteamAPI_ISteamUser_GetSteamID(self.user).0)
        }
    }

    /// Retrieve an authentication session ticket that can be sent
    /// to an entity that wishes to verify you.
    ///
    /// This ticket should not be reused.
    ///
    /// When creating ticket for use by the web API you should wait
    /// for the `AuthSessionTicketResponse` event before trying to
    /// use the ticket.
    ///
    /// When the multiplayer session terminates you must call
    /// `cancel_authentication_ticket`
    pub fn authentication_session_ticket(&self) -> (AuthTicket, Vec<u8>) {
        unsafe {
            let mut ticket = vec![0; 1024];
            let mut ticket_len = 0;
            let auth_ticket = sys::SteamAPI_ISteamUser_GetAuthSessionTicket(self.user, ticket.as_mut_ptr() as *mut _, 1024, &mut ticket_len);
            ticket.truncate(ticket_len as usize);
            (AuthTicket(auth_ticket), ticket)
        }
    }

    /// Cancels an authentication session ticket received from
    /// `authentication_session_ticket`.
    ///
    /// This should be called when you are no longer playing with
    /// the specified entity.
    pub fn cancel_authentication_ticket(&self, ticket: AuthTicket) {
        unsafe {
            sys::SteamAPI_ISteamUser_CancelAuthTicket(self.user, ticket.0);
        }
    }

    /// Authenticate the ticket from the steam ID to make sure it is
    /// valid and not reused.
    ///
    /// A `ValidateAuthTicketResponse` callback will be fired if
    /// the entity goes offline or cancels the ticket.
    ///
    /// When the multiplayer session terminates you must call
    /// `end_authentication_session`
    pub fn begin_authentication_session(&self, user: SteamId, ticket: &[u8]) -> Result<(), AuthSessionError> {
        unsafe {
            let res = sys::SteamAPI_ISteamUser_BeginAuthSession(
                self.user,
                ticket.as_ptr() as *const _, ticket.len() as _,
                sys::CSteamID(user.0)
            );
            Err(match res {
                sys::EBeginAuthSessionResult::EBeginAuthSessionResultOK => return Ok(()),
                sys::EBeginAuthSessionResult::EBeginAuthSessionResultInvalidTicket => AuthSessionError::InvalidTicket,
                sys::EBeginAuthSessionResult::EBeginAuthSessionResultDuplicateRequest => AuthSessionError::DuplicateRequest,
                sys::EBeginAuthSessionResult::EBeginAuthSessionResultInvalidVersion => AuthSessionError::InvalidVersion,
                sys::EBeginAuthSessionResult::EBeginAuthSessionResultGameMismatch => AuthSessionError::GameMismatch,
                sys::EBeginAuthSessionResult::EBeginAuthSessionResultExpiredTicket => AuthSessionError::ExpiredTicket,
            })
        }
    }

    /// Ends an authentication session that was started with
    /// `begin_authentication_session`.
    ///
    /// This should be called when you are no longer playing with
    /// the specified entity.
    pub fn end_authentication_session(&self, user: SteamId) {
        unsafe {
            sys::SteamAPI_ISteamUser_EndAuthSession(self.user, sys::CSteamID(user.0));
        }
    }
}

/// Errors from `begin_authentication_session`
#[derive(Debug, Fail)]
pub enum AuthSessionError {
    /// The ticket is invalid
    #[fail(display = "invalid ticket")]
    InvalidTicket,
    /// A ticket has already been submitted for this steam ID
    #[fail(display = "duplicate ticket request")]
    DuplicateRequest,
    /// The ticket is from an incompatible interface version
    #[fail(display = "incompatible interface version")]
    InvalidVersion,
    /// The ticket is not for this game
    #[fail(display = "incorrect game for ticket")]
    GameMismatch,
    /// The ticket has expired
    #[fail(display = "ticket has expired")]
    ExpiredTicket,
}

#[test]
#[serial]
fn test() {
    let (client, single) = Client::init().unwrap();
    let user = client.user();

    let _cb = client.register_callback(|v: AuthSessionTicketResponse| println!("Got response: {:?}", v.result));
    let _cb = client.register_callback(|v: ValidateAuthTicketResponse| println!("{:?}", v));

    let id = user.steam_id();
    let (auth, ticket) = user.authentication_session_ticket();

    println!("{:?}", user.begin_authentication_session(id, &ticket));

    for _ in 0 .. 20 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    println!("END");

    user.cancel_authentication_ticket(auth);

    for _ in 0 .. 20 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    user.end_authentication_session(id);
}

/// A handle for an authentication ticket that can be used to cancel
/// it.
pub struct AuthTicket(pub(crate) sys::HAuthTicket);

/// Called when generating a authentication session ticket.
///
/// This can be used to verify the ticket was created successfully.
pub struct AuthSessionTicketResponse {
    /// The ticket in question
    pub ticket: AuthTicket,
    /// The result of generating the ticket
    pub result: SResult<()>,
}

unsafe impl Callback for AuthSessionTicketResponse {
    const ID: i32 = 163;
    const SIZE: i32 = ::std::mem::size_of::<sys::GetAuthSessionTicketResponse_t>() as i32;

    unsafe fn from_raw(raw: *mut libc::c_void) -> Self {
        let val = &mut *(raw as *mut sys::GetAuthSessionTicketResponse_t);
        AuthSessionTicketResponse {
            ticket: AuthTicket(val.m_hAuthTicket),
            result: if val.m_eResult == sys::EResult::EResultOK  {
                Ok(())
            } else {
                Err(val.m_eResult.into())
            }
        }
    }
}

/// Called when an authentication ticket has been
/// validated.
#[derive(Debug)]
pub struct ValidateAuthTicketResponse {
    /// The steam id of the entity that provided the ticket
    pub steam_id: SteamId,
    /// The result of the validation
    pub response: Result<(), AuthSessionValidateError>,
    /// The steam id of the owner of the game. Differs from
    /// `steam_id` if the game is borrowed.
    pub owner_steam_id: SteamId,
}


unsafe impl Callback for ValidateAuthTicketResponse {
    const ID: i32 = 143;
    const SIZE: i32 = ::std::mem::size_of::<sys::ValidateAuthTicketResponse_t>() as i32;

    unsafe fn from_raw(raw: *mut libc::c_void) -> Self {
        let val = &mut *(raw as *mut sys::ValidateAuthTicketResponse_t);
        ValidateAuthTicketResponse {
            steam_id: SteamId(val.m_SteamID.0),
            owner_steam_id: SteamId(val.m_OwnerSteamID.0),
            response: match val.m_eAuthSessionResponse {
                sys::EAuthSessionResponse::EAuthSessionResponseOK => Ok(()),
                sys::EAuthSessionResponse::EAuthSessionResponseUserNotConnectedToSteam => Err(AuthSessionValidateError::UserNotConnectedToSteam),
                sys::EAuthSessionResponse::EAuthSessionResponseNoLicenseOrExpired => Err(AuthSessionValidateError::NoLicenseOrExpired),
                sys::EAuthSessionResponse::EAuthSessionResponseVACBanned => Err(AuthSessionValidateError::VACBanned),
                sys::EAuthSessionResponse::EAuthSessionResponseLoggedInElseWhere => Err(AuthSessionValidateError::LoggedInElseWhere),
                sys::EAuthSessionResponse::EAuthSessionResponseVACCheckTimedOut => Err(AuthSessionValidateError::VACCheckTimedOut),
                sys::EAuthSessionResponse::EAuthSessionResponseAuthTicketCanceled => Err(AuthSessionValidateError::AuthTicketCancelled),
                sys::EAuthSessionResponse::EAuthSessionResponseAuthTicketInvalidAlreadyUsed => Err(AuthSessionValidateError::AuthTicketInvalidAlreadyUsed),
                sys::EAuthSessionResponse::EAuthSessionResponseAuthTicketInvalid => Err(AuthSessionValidateError::AuthTicketInvalid),
                sys::EAuthSessionResponse::EAuthSessionResponsePublisherIssuedBan => Err(AuthSessionValidateError::PublisherIssuedBan),
            }
        }
    }
}

/// Errors from `ValidateAuthTicketResponse`
#[derive(Debug, Fail)]
pub enum AuthSessionValidateError {
    /// The user in question is not connected to steam
    #[fail(display = "user not connected to steam")]
    UserNotConnectedToSteam,
    /// The license has expired
    #[fail(display = "the license has expired")]
    NoLicenseOrExpired,
    /// The user is VAC banned from the game
    #[fail(display = "the user is VAC banned from this game")]
    VACBanned,
    /// The user has logged in elsewhere and the session
    /// has been disconnected
    #[fail(display = "the user is logged in elsewhere")]
    LoggedInElseWhere,
    /// VAC has been unable to perform anti-cheat checks on this
    /// user
    #[fail(display = "VAC check timed out")]
    VACCheckTimedOut,
    /// The ticket has been cancelled by the issuer
    #[fail(display = "the authentication ticket has been cancelled")]
    AuthTicketCancelled,
    /// The ticket has already been used
    #[fail(display = "the authentication ticket has already been used")]
    AuthTicketInvalidAlreadyUsed,
    /// The ticket is not from a user instance currently connected
    /// to steam
    #[fail(display = "the authentication ticket is invalid")]
    AuthTicketInvalid,
    /// The user is banned from the game (not VAC)
    #[fail(display = "the user is banned")]
    PublisherIssuedBan,
}
