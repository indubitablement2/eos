use crate::global::GlobalList;
use eos_common::packet_common::*;
use parking_lot::RwLock;
use std::{convert::TryInto, sync::Arc};

/// Control server with console commands.
pub struct ServerCommand {
    command_receiver: flume::Receiver<(String, i32)>,
    broadcasting: bool,
}

impl ServerCommand {
    pub fn new() -> ServerCommand {
        let (command_sender, command_receiver) = flume::bounded::<(String, i32)>(0);

        std::thread::spawn(move || {
            loop {
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer).unwrap();

                // Remove new line.
                if buffer.ends_with('\n') {
                    buffer.pop();
                }
                if buffer.ends_with('\r') {
                    buffer.pop();
                }

                // Separate cmd from value.
                let mut value = String::with_capacity(10);
                for c in buffer.chars().rev() {
                    if c.is_ascii_digit() {
                        value.push(c);
                    } else {
                        break;
                    }
                }

                // Get the numbers at the end if any.
                value = value.chars().rev().collect::<String>();

                // Remove the value from cmd.
                for _ in 0..value.len() {
                    buffer.pop();
                }

                // Remove trailing whitespace from cmd.
                while buffer.ends_with(' ') {
                    buffer.pop();
                }

                if command_sender.send((buffer, value.parse().unwrap_or_default())).is_err() {
                    error!("Command receiver was drop. No more command can be issued.");
                    break;
                }
            }
        });

        info!("Command loop ready");

        ServerCommand {
            command_receiver,
            broadcasting: false,
        }
    }

    pub fn process_command(&mut self, exit: &mut bool, accept_login: &mut bool, global_list: &Arc<RwLock<GlobalList>>) {
        if let Ok((cmd, value)) = self.command_receiver.try_recv() {
            if self.broadcasting {
                let say = OtherPacket::Broadcast {
                    importance: value.try_into().unwrap_or_default(),
                    message: cmd,
                }
                .serialize();

                // Send to all client.
                global_list.read().connected_client.values().for_each(|connection| {
                    connection.send_packet(say.clone());
                });

                self.broadcasting = false;
                info!("Stopped broadcasting.");
                return;
            }

            match cmd.as_str() {
                "exit" => {
                    warn!("Shutting down...");
                    *exit = true;
                }
                "save" => {
                    error!("TODO");
                }
                "login" => {
                    let v = value != 0;
                    warn!("Set login to: {}", v);
                    *accept_login = v;
                }
                "broadcast" => {
                    warn!(
                        "Your next command will be broadcasted to everyone. If your message ends with a number, it will be lost."
                    );
                    self.broadcasting = true;
                }
                "connected" => {
                    info!("{}", global_list.read().connected_client.len());
                }
                "disconnect" => {
                    error!("TODO");
                }
                _ => {
                    info!("Unimplemented command: {:?}, {:?}", cmd, value);
                }
            }
        }
    }
}
