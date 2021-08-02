use ahash::AHashSet;
use crossbeam_channel::{bounded, Receiver, Sender};
use eos_common::connection_manager::*;
use eos_common::const_var::*;
use eos_common::data::FleetData;
use eos_common::idx::*;
use eos_common::packet_mod::*;
use parking_lot::RwLock;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crate::global::GlobalList;

// * Accept TcpStream, convert to Connection and send to login loop.

// * Wait for ClientHello.

// TODO Check that username exist.

// TODO Make salt.

// TODO Send salt to the client.

// TODO Hash client password with salt.

// TODO Compare hash.

// TODO Set as loged-in to prevent someone else from taking this account.

// TODO Gather the client data. Look at dc first.

// * Send to sector.

/// Connection that have fully passed login process.
struct LoginSuccess {
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
    pub step: LoginStep,
    pub client_id: ClientId,
    pub username: String,
}

#[derive(PartialEq)]
enum LoginStep {
    WaitingForClientHello,
    WaitingForClientHash,
    Ready,
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
        // Create channel to send new connection from listening loop to login loop.
        let (con_sender, con_receiver) = bounded::<Connection>(LISTENING_BUFFER);

        // Create channel to send success login.
        let (login_success_sender, login_success_receiver) = bounded(LOGIN_SUCCESS_BUFFER);

        // Start listening loop.
        std::thread::spawn(move || {
            listening_loop(con_sender, connection_starter);
        });

        // Start login loop.
        std::thread::spawn(move || {
            login_loop(con_receiver, login_success_sender);
        });

        LoginThread { login_success_receiver }
    }

    /// Receive login from the login loop.
    pub fn process_login(&self, global_list: &Arc<RwLock<GlobalList>>, sector_sender: &[Sender<FleetData>]) {
        let mut global_list_write = global_list.write();

        while let Ok(login_success) = self.login_success_receiver.try_recv() {
            trace!(
                "Client {:?} connected: {:?}",
                login_success.client_id,
                login_success.connection.address
            );
            // * Add to connected.
            if let Some(old_connection) = global_list_write
                .connected_client
                .insert(login_success.client_id, login_success.connection)
            {
                debug!(
                    "Someone connected with an already connected ClientId: {:?}",
                    login_success.client_id
                );
                old_connection.send_packet(
                    ServerPacket::Broadcast {
                        importance: 0,
                        message: "Someone logged-in this account. If this is unintended, you should change your password."
                            .to_string(),
                    }
                    .serialize(),
                );
            }

            // * Send their fleets to sectors, if they are not there already.
            for mut fleet in login_success.database_fleet_data {
                if !global_list_write.fleet_current_sector.contains_key(&fleet.fleet_id) {
                    for i in 0..3 {
                        // TODO: Add to const var
                        let sec_id =
                            (usize::from(fleet.location.sector_id.0) + rand::random::<usize>() * i) % sector_sender.len();
                        if let Some(sector_send) = sector_sender.get(sec_id) {
                            trace!("Sending fleet {:?} to sector {}", &fleet.fleet_id, &sec_id);
                            match sector_send.send(fleet) {
                                Ok(_) => break,
                                Err(err) => {
                                    fleet = err.0;
                                    debug!("Could not send fleet {:?} to sector {}", &fleet.fleet_id, &sec_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Accept new socket, convert to Connection then send to login loop.
fn listening_loop(con_sender: Sender<Connection>, connection_starter: ConnectionStarter) {
    // Start listening for new socket.
    let listener = std::net::TcpListener::bind(eos_common::const_var::SERVER_PORT).unwrap();
    info!("Listening on {:?}", listener.local_addr());

    // Prevent address from trying to connect too fast.
    let mut temp_banned_address = AHashSet::with_capacity(512);
    let mut new_temp_banned_address = AHashSet::with_capacity(512);
    let mut last_temp_banned_address_clear = Instant::now();

    loop {
        if let Ok((new_socket, address)) = listener.accept() {
            trace!("Accepted new socket: {}", &address);

            // Check if we should clear temp banned address.
            if last_temp_banned_address_clear.elapsed() > LISTENING_TEMP_BAN_DURATION {
                std::mem::swap(&mut temp_banned_address, &mut new_temp_banned_address);
                new_temp_banned_address.clear();
                last_temp_banned_address_clear = Instant::now();
            }

            // Check if address is banned.
            if temp_banned_address.contains(&address.ip()) {
                trace!("Banned address was denied connection.");
                continue;
            }

            // Prevent this address from reconnecting for a while.
            temp_banned_address.insert(address.ip());
            new_temp_banned_address.insert(address.ip());

            // Convert TcpStream to Connection.
            match connection_starter.create_connection(new_socket, address) {
                Some(new_connection) => {
                    // Block until login loop receive.
                    if let Err(err) = con_sender.send(new_connection) {
                        error!("Listening loop terminated: {:?}", &err);
                        return;
                    }
                }
                None => {
                    debug!("Listening loop could not create connection.");
                }
            }
        }
    }
}

/// Verify username and app version.
/// Send salt.
/// Verify hashed password.
/// Load user data.
/// Send to sector.
fn login_loop(con_receiver: Receiver<Connection>, login_success_sender: Sender<LoginSuccess>) {
    // Store LoginInProgress. This should not grow.
    let mut in_progress: Vec<LoginInProgress> = Vec::with_capacity(LOGIN_PROGRESS_BUFFER);

    // To remove (position) from in_progress, because of success or failure.
    let mut to_remove: Vec<usize> = Vec::with_capacity(LOGIN_PROGRESS_BUFFER);

    // Address that have done something wrong.
    let mut temp_warned_address = AHashSet::with_capacity(256);
    let mut temp_banned_address = AHashSet::with_capacity(256);
    let mut last_temp_banned_address_clear = Instant::now();

    // Keep a buffer of client recently removed(success or failure), so that they can not relog too fast and cause double login.
    let mut recently_removed: AHashSet<ClientId> = AHashSet::with_capacity(LOGIN_PROGRESS_BUFFER * 10);
    let mut new_recently_removed: AHashSet<ClientId> = AHashSet::with_capacity(LOGIN_PROGRESS_BUFFER * 10);
    let mut last_recently_removed_clear = Instant::now();

    loop {
        let start_instant = Instant::now();

        // * Check if we should clear temp banned address.
        if last_temp_banned_address_clear.elapsed() > LOGIN_TEMP_BAN_DURATION {
            std::mem::swap(&mut temp_warned_address, &mut temp_banned_address);
            temp_banned_address.clear();
            last_temp_banned_address_clear = Instant::now();
        }

        // * Get some new Connection, if we have space.
        while in_progress.len() < LOGIN_PROGRESS_BUFFER {
            match con_receiver.try_recv() {
                Ok(new_connection) => {
                    if temp_banned_address.contains(&new_connection.address.ip()) {
                        trace!("Login loop received a temp banned address from listening loop.");
                        continue;
                    }
                    // Convert to LoginInProgress.
                    in_progress.push(LoginInProgress {
                        last_answer: Instant::now(),
                        connection: new_connection,
                        step: LoginStep::WaitingForClientHello,
                        username: String::new(),
                        client_id: ClientId(0),
                    });
                }
                Err(err) => {
                    match err {
                        crossbeam_channel::TryRecvError::Empty => {
                            // Running out of new Connection.
                            break;
                        }
                        crossbeam_channel::TryRecvError::Disconnected => {
                            error!("Login loop terminated. Sender dropped.");
                            return;
                        }
                    }
                }
            }
        }

        // * Iterate over all LoginInProgress.
        in_progress.iter_mut().enumerate().for_each(|(i, progress)| {
            // * Check if Connection is disconnected.
            if progress.connection.is_disconnected() {
                trace!("Connection disconnected while trying to login.");
                warn(
                    &progress.connection.address,
                    &mut temp_warned_address,
                    &mut temp_banned_address,
                );
                to_remove.push(i);
            }

            // * Check if Connection is not responding in time.
            if progress.last_answer.elapsed() >= eos_common::const_var::MAX_LOGIN_WAIT {
                trace!("Connection is not responding during login.");
                warn(
                    &progress.connection.address,
                    &mut temp_warned_address,
                    &mut temp_banned_address,
                );
                to_remove.push(i);
            }

            // * Check that this client_id did not try to login recently.
            if recently_removed.contains(&progress.client_id) {
                trace!("Some client tried to login while its client_id is still in recently_removed.");
                progress.connection.send_packet(
                    ServerPacket::Broadcast {
                        importance: 0,
                        message: "Wait at least 20 seconds before retrying to login.".to_string(),
                    }
                    .serialize(),
                );
                to_remove.push(i);
            }

            match progress.step {
                LoginStep::WaitingForClientHello => {
                    if let Ok(packet) = progress.connection.login_packet.try_recv() {
                        match packet {
                            ClientLoginPacket::Hello { username, app_version } => {
                                // * Check app version.
                                if app_version != APP_VERSION {
                                    trace!("Client app version ({}) does not match server ({})", app_version, APP_VERSION);
                                    progress.connection.send_packet(
                                        ServerPacket::Broadcast {
                                            importance: 0,
                                            message: "Your app version does not match with server.".to_string(),
                                        }
                                        .serialize(),
                                    );
                                    warn(
                                        &progress.connection.address,
                                        &mut temp_warned_address,
                                        &mut temp_banned_address,
                                    );
                                    to_remove.push(i);
                                } else {
                                    progress.last_answer = Instant::now();
                                    progress.username = username.to_owned();
                                    // TODO: Check if username exist.
                                    // TODO: Fetch password.
                                    // TODO: Hash password with salt.
                                    // TODO: Send salt.
                                    progress.step = LoginStep::WaitingForClientHash;
                                }
                            }
                            _ => {
                                // * Client sent wrong packet or gibberish.
                                debug!(
                                    "Temp banned {:?}, because first packet was not ClientHello.",
                                    &progress.connection.address
                                );
                                temp_banned_address.insert(progress.connection.address.ip());
                                to_remove.push(i);
                            }
                        }
                    }
                }
                LoginStep::WaitingForClientHash => {
                    // TODO: skip auth for now.
                    progress.last_answer = Instant::now();
                    progress.client_id = ClientId(rand::random());
                    progress.step = LoginStep::Ready;
                }
                _ => {
                    to_remove.push(i);
                }
            }
        });

        // * Send/remove
        to_remove.sort_unstable();
        to_remove.dedup();
        to_remove.reverse();
        to_remove.drain(..).for_each(|i| {
            if i > in_progress.len() {
                warn!("Tried to swap_remove from in_progress with index out of bound: {}", i);
            } else {
                let removed_progress = in_progress.swap_remove(i);

                if removed_progress.step == LoginStep::Ready {
                    debug_assert_ne!(removed_progress.client_id.0, 0);
                    recently_removed.insert(removed_progress.client_id);
                    new_recently_removed.insert(removed_progress.client_id);
                    get_data_and_send(removed_progress, &login_success_sender);
                }
            }
        });

        // * Clear old recently removed client_id.
        if last_recently_removed_clear.elapsed() > RECENTLY_REMOVED_TIMEOUT {
            last_recently_removed_clear = Instant::now();

            recently_removed = new_recently_removed.clone();
            new_recently_removed.clear();
        }

        // * Sleep until at least 100ms have passed from last update.
        let remaining = MAIN_LOOP_DURATION.saturating_sub(start_instant.elapsed());
        if !Duration::is_zero(&remaining) {
            sleep(remaining);
        }
    }
}

/// Add address to warn and maybe temp ban.
fn warn(address: &SocketAddr, warned: &mut AHashSet<IpAddr>, banned: &mut AHashSet<IpAddr>) {
    if !warned.insert(address.ip()) {
        banned.insert(address.ip());
        trace!("Temp banned {:?}", address.ip());
    }
}

/// Fetch the user's FleetData and send to main.
fn get_data_and_send(progress: LoginInProgress, login_success_sender: &Sender<LoginSuccess>) {
    // TODO: Try to fetch FleetData from file.

    let fleet_datas = vec![];

    let login_success = LoginSuccess {
        connection: progress.connection,
        client_id: progress.client_id,
        database_fleet_data: fleet_datas,
    };

    // Send.
    if let Err(err) = login_success_sender.send(login_success) {
        error!("Error while sending LoginSuccess: {:?}", &err);
    }
}
