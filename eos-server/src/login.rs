use ahash::AHashSet;
use eos_common::connection_manager::*;
use eos_common::const_var::*;
use eos_common::data::FleetData;
use eos_common::idx::*;
use eos_common::packet_common::*;
use flume::{bounded, Receiver, Sender};
use std::mem::swap;
use tokio::time::Instant;
use crate::global::GlobalList;

// * Accept TcpStream, convert to Connection and send to login loop.
// * Wait for ClientLogin.
// * Verify with steam.
// TODO Gather the client's data.
// TODO Check ban.
// * Send to main.

/// Connection that have fully passed login process.
pub struct LoginSuccess {
    pub connection: Connection,
    pub client_id: ClientId,
    /// FleetData fetched from database.
    pub database_fleet_data: Vec<FleetData>,
}

/// Connection that are accepted, but not ready to join.
struct LoginInProgress {
    /// Instant that this Connection last answered.
    /// Used to prevent Connection from sitting in login for too long.
    pub last_answer: Instant,
    pub connection: Connection,
    pub client_id: ClientId,
    pub ticket: String,
}

/// Continually try to login client.
/// Take only two threads, so it is slow and inexpensive.
pub struct LoginThread {
    /// Copy of the senders used after successful login.
    login_success_receiver: Receiver<LoginSuccess>,
}

impl LoginThread {
    /// There should only be one of these.
    pub fn new(connection_starter: ConnectionStarter) -> LoginThread {
        // Create channel to send success login.
        let (login_success_sender, login_success_receiver) = bounded(LOGIN_SUCCESS_BUFFER);
        let runtime_handle = connection_starter.runtime_handle.clone();

        // Start login loop.
        runtime_handle.spawn(login_loop(connection_starter, login_success_sender));

        LoginThread { login_success_receiver }
    }

    /// Take successful login and add them to GlobalList. Add their fleets to ecs.
    pub fn process_login(&self, global_list: &mut GlobalList, sector_sender: &[Sender<FleetData>]) {
        while let Ok(new_login) = self.login_success_receiver.try_recv() {
            // Add to GlobalList.
            if global_list
                .connected_client
                .insert(new_login.client_id, new_login.connection)
                .is_some()
            {
                trace!("Two clients logged-in with the same {:?}", new_login.client_id);
            }

            // TODO: Check that fleet is not in game and send to sector.
        }
    }
}

async fn login_loop(connection_starter: ConnectionStarter, login_success_sender: Sender<LoginSuccess>) {
    // Start listening for new socket.
    let listener = tokio::net::TcpListener::bind(SERVER_PORT).await.unwrap();
    info!("Listening on {:?}", listener.local_addr());

    // Used to send request to steam web api.
    let steam_client = reqwest::Client::new();

    // Store LoginInProgress. This should not grow.
    let mut in_progress: Vec<LoginInProgress> = Vec::with_capacity(LOGIN_PROGRESS_BUFFER);
    let mut in_progress_future = Vec::with_capacity(LOGIN_PROGRESS_BUFFER);

    // Address that can not reconnect for a while..
    let mut temp_banned_address = AHashSet::with_capacity(LOGIN_PROGRESS_BUFFER * 10);
    let mut new_temp_banned_address = AHashSet::with_capacity(LOGIN_PROGRESS_BUFFER * 10);
    let mut last_temp_banned_address_clear = Instant::now();

    // Keep a buffer of client recently removed(success or failure), so that they can not relog too fast and cause double login.
    let mut recently_removed: AHashSet<ClientId> = AHashSet::with_capacity(LOGIN_PROGRESS_BUFFER * 10);
    let mut new_recently_removed: AHashSet<ClientId> = AHashSet::with_capacity(LOGIN_PROGRESS_BUFFER * 10);
    let mut last_recently_removed_clear = Instant::now();

    let mut exit = false;

    while !exit {
        // * Check if we should clear temp_banned_address.
        if last_temp_banned_address_clear.elapsed() > LOGIN_TEMP_BAN_DURATION {
            swap(&mut new_temp_banned_address, &mut temp_banned_address);
            new_temp_banned_address.clear();
            last_temp_banned_address_clear = Instant::now();
        }

        let timeout = Instant::now() + LOGIN_LOOP_LISTENING_DURATION;

        // * Get some new Connection.
        while let Ok(result) = tokio::time::timeout_at(timeout, listener.accept()).await {
            if let Ok((new_socket, address)) = result {
                trace!("Accepted new socket: {}", &address);

                // Check if address is banned.
                if temp_banned_address.contains(&address.ip()) {
                    trace!("Banned address was denied connection.");
                    continue;
                }

                // Prevent this address from reconnecting for a while.
                temp_banned_address.insert(address.ip());
                new_temp_banned_address.insert(address.ip());

                // Convert to Connection.
                let new_connection = connection_starter.create_connection(new_socket);

                // Add to in_progress.
                in_progress.push(LoginInProgress {
                    last_answer: Instant::now(),
                    connection: new_connection,
                    client_id: ClientId(0),
                    ticket: String::new(),
                });
            }

            // Check if we are at max capacity.
            if in_progress.len() >= LOGIN_PROGRESS_BUFFER {
                break;
            }
        }

        // * Try to login all LoginInProgress in parallel.
        in_progress.drain(..).for_each(|progress| {
            in_progress_future.push(
                connection_starter
                    .runtime_handle
                    .spawn(login_one(progress, steam_client.clone())),
            );
        });

        in_progress_future.drain(..).for_each(|handle| {
            match connection_starter.runtime_handle.block_on(handle) {
                Ok(future_result) => {
                    match future_result {
                        Ok(success) => {
                            // Check if ClientId was not used recently.
                            debug_assert_ne!(success.client_id.0, 0);
                            if recently_removed.contains(&success.client_id) {
                                trace!("Some client tried to login while its client_id is still in recently_removed.");
                                success.connection.send_packet(
                                    OtherPacket::Broadcast {
                                        importance: 0,
                                        message: "Wait at least 20 seconds before retrying to login.".to_string(),
                                    }
                                    .serialize(),
                                );
                            } else {
                                // Add ClientId to recently_removed.
                                recently_removed.insert(success.client_id);
                                new_recently_removed.insert(success.client_id);

                                // Send success to GlobalList.
                                if let Err(err) = login_success_sender.send(success) {
                                    error!("Error while sending successful login: {:?}", err);
                                    exit = true;
                                }
                            }
                        }
                        Err(login_unsuccessful) => {
                            if let Some(unsuccessful) = login_unsuccessful {
                                // Give another chance if under wait threshold.
                                if unsuccessful.last_answer.elapsed() < MAX_LOGIN_WAIT {
                                    in_progress.push(unsuccessful);
                                } else {
                                    trace!("Client did not answer in time while login-in.");
                                }
                            }
                        }
                    }
                }
                Err(future_err) => {
                    debug!("A login attempt did not complete: {:?}", future_err);
                }
            }
        });

        // * Clear recently_removed client_id.
        if last_recently_removed_clear.elapsed() > RECENTLY_REMOVED_TIMEOUT {
            swap(&mut recently_removed, &mut new_recently_removed);
            new_recently_removed.clear();
            last_recently_removed_clear = Instant::now();
        }
    }
}

async fn login_one(
    mut progress: LoginInProgress,
    steam_client: reqwest::Client,
) -> Result<LoginSuccess, Option<LoginInProgress>> {
    // * Look for ClientLogin.
    if progress.client_id.0 == 0 {
        match progress.connection.other_packet_receiver.try_recv() {
            Ok(packet) => {
                match packet {
                    OtherPacket::ClientLogin {
                        app_version,
                        steam_id,
                        ticket,
                    } => {
                        // Check app version.
                        if app_version != APP_VERSION {
                            trace!("Client app version ({}) does not match server ({})", app_version, APP_VERSION);
                            progress.connection.send_packet(
                                OtherPacket::Broadcast {
                                    importance: 0,
                                    message: "Your app version does not match with server.".to_string(),
                                }
                                .serialize(),
                            );
                            return Err(Option::None);
                        }

                        // Check if ClientId is valid.
                        if !steam_id.is_valid() {
                            trace!("Some client tried to login with client_id 0.");
                            return Err(Option::None);
                        }

                        // Check if ClientId was not used recently.
                        // if recently_removed.contains(&steam_id) {
                        //     trace!("Some client tried to login while its client_id is still in recently_removed.");
                        //     progress.connection.send_packet(
                        //         OtherPacket::Broadcast {
                        //             importance: 0,
                        //             message: "Wait at least 20 seconds before retrying to login.".to_string(),
                        //         }
                        //         .serialize(),
                        //     );
                        //     return Err(Option::None);
                        // }

                        // All good. Put data into progress.
                        progress.client_id = steam_id;
                        progress.ticket = ticket;
                    }
                    _ => {
                        trace!("Some client sent wrong packet while trying to login.");
                        return Err(Option::None);
                    }
                }
            }
            Err(err) => {
                match err {
                    flume::TryRecvError::Empty => {
                        // Nothing to read. Try again next time.
                        return Err(Some(progress));
                    }
                    flume::TryRecvError::Disconnected => {
                        trace!("Disconnected while trying to login.");
                        return Err(Option::None);
                    }
                }
            }
        }
    }

    // * Verify ticket with steam.
    match steam_client.get(format!(
        "https://partner.steam-api.com/ISteamUserAuth/AuthenticateUserTicket/v1/?key=45B6C4EFE51FDC92AB87FCF8ACC96405&appid=1638530&ticket={}",
        progress.ticket
    ))
    .send().await {
        Ok(resp) => {
            let steam_auth_json = resp.json::<SteamJson>().await.unwrap_or_default();

            // Check steam result.
            if !steam_auth_json.response.params.result.as_str().eq("OK") {
                debug!("Steam denied a client: {}", &steam_auth_json.response.params.result);
                return Err(Option::None);
            }

            // Check if given ClientId match steam id.
            if steam_auth_json.response.params.steamid.parse::<u64>().unwrap_or_default() != progress.client_id.0 {
                debug!("Login failed: steamid {} and ClientId {} don't match.", steam_auth_json.response.params.steamid, progress.client_id.0);
                return Err(Option::None);
            }

            // TODO: Fetch user's data.
            let database_fleet_data = Vec::new();

            // TODO: Check ban

            // Successful login.
            Ok(LoginSuccess {
                connection: progress.connection,
                client_id: progress.client_id,
                database_fleet_data,
            })
        }
        Err(err) => {
            debug!("Error contacting steam while trying to login a client: {:?}", err);
            Err(Option::None)
        }
    }
}

#[derive(serde::Deserialize, Debug)]
struct SteamJson {
    pub response: SteamResponse,
}

impl Default for SteamJson {
    fn default() -> Self {
        SteamJson {
            response: SteamResponse {
                params: SteamParams {
                    result: "Failed to reach steam.".to_string(),
                    steamid: String::new(),
                    ownersteamid: String::new(),
                    vacbanned: false,
                    publisherbanned: false,
                },
            },
        }
    }
}

#[derive(serde::Deserialize, Debug)]
struct SteamResponse {
    pub params: SteamParams,
}

#[derive(serde::Deserialize, Debug)]
struct SteamParams {
    pub result: String,
    pub steamid: String,
    pub ownersteamid: String,
    pub vacbanned: bool,
    pub publisherbanned: bool,
}
