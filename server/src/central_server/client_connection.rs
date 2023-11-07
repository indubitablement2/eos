use super::*;

#[derive(Clone, Copy)]
enum BattlescapeChange {
    None,
    Leave {
        old_addr: SocketAddr,
    },
    SameInstance {
        battlescape_id: BattlescapeId,
        addr: SocketAddr,
    },
    ChangeInstance {
        battlescape_id: BattlescapeId,
        old_addr: SocketAddr,
        new_addr: SocketAddr,
    },
    Join {
        battlescape_id: BattlescapeId,
        new_addr: SocketAddr,
    },
}

pub struct ClientConnection {
    outbound: ConnectionOutbound,
    pub client_id: ClientId,
    pub token: u64,
    current_battlescape: Mutex<Option<BattlescapeId>>,
}
impl ClientConnection {
    pub fn new(outbound: ConnectionOutbound, client_id: ClientId, token: u64) -> Self {
        Self {
            outbound,
            client_id,
            token,
            current_battlescape: Default::default(),
        }
    }

    /// Automatically notify instances and client.
    ///
    /// Locks battlescapes and instances.
    pub fn set_battlescape(&self, new_battlescape_id: Option<BattlescapeId>) {
        let mut lock = self.current_battlescape.lock();
        let old_battlescape_id = *lock;
        *lock = new_battlescape_id;

        let mut changes = BattlescapeChange::None;

        if let Some(battlescape_id) = old_battlescape_id {
            if let Some(battlescape) = state().battlescapes.get(&battlescape_id) {
                battlescape.clients.lock().remove(&self.client_id);

                changes = BattlescapeChange::Leave {
                    old_addr: battlescape.instance_addr,
                };
            }
        }

        if let Some(battlescape_id) = new_battlescape_id {
            if let Some(battlescape) = state().battlescapes.get(&battlescape_id) {
                battlescape.clients.lock().insert(self.client_id);
                let new_addr = battlescape.instance_addr;
                drop(battlescape);

                match changes {
                    BattlescapeChange::None => {
                        changes = BattlescapeChange::Join {
                            battlescape_id,
                            new_addr,
                        };
                    }
                    BattlescapeChange::Leave { old_addr } => {
                        if old_addr == new_addr {
                            changes = BattlescapeChange::SameInstance {
                                battlescape_id,
                                addr: new_addr,
                            };
                        } else {
                            changes = BattlescapeChange::ChangeInstance {
                                battlescape_id,
                                old_addr,
                                new_addr,
                            };
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        let new = match changes {
            BattlescapeChange::None => None,
            BattlescapeChange::Leave { old_addr } => {
                if let Some(instance) = state().instances.get(&old_addr) {
                    instance.send(CentralInstancePacket::ClientChangedBattlescape {
                        client_id: self.client_id,
                        token: self.token,
                        battlescape_id: None,
                    });
                }
                None
            }
            BattlescapeChange::SameInstance {
                battlescape_id,
                addr,
            } => {
                if let Some(instance) = state().instances.get(&addr) {
                    instance.send(CentralInstancePacket::ClientChangedBattlescape {
                        client_id: self.client_id,
                        token: self.token,
                        battlescape_id: Some(battlescape_id),
                    });
                }
                Some((battlescape_id, true, addr))
            }
            BattlescapeChange::ChangeInstance {
                battlescape_id,
                old_addr,
                new_addr,
            } => {
                if let Some(instance) = state().instances.get(&old_addr) {
                    instance.send(CentralInstancePacket::ClientChangedBattlescape {
                        client_id: self.client_id,
                        token: self.token,
                        battlescape_id: None,
                    });
                }
                if let Some(instance) = state().instances.get(&new_addr) {
                    instance.send(CentralInstancePacket::ClientChangedBattlescape {
                        client_id: self.client_id,
                        token: self.token,
                        battlescape_id: Some(battlescape_id),
                    });
                }
                Some((battlescape_id, false, new_addr))
            }
            BattlescapeChange::Join {
                battlescape_id,
                new_addr,
            } => {
                if let Some(instance) = state().instances.get(&new_addr) {
                    instance.send(CentralInstancePacket::ClientChangedBattlescape {
                        client_id: self.client_id,
                        token: self.token,
                        battlescape_id: Some(battlescape_id),
                    });
                }
                Some((battlescape_id, false, new_addr))
            }
        };

        // Notify client.
        if let Some((battlescape_id, same_addr, addr)) = new {
            self.send(CentralClientPacket::ChangeBattlescape {
                battlescape_id: Some(battlescape_id),
                same_addr,
                instance_addr: Some(addr),
            });
        } else {
            self.send(CentralClientPacket::ChangeBattlescape {
                battlescape_id: None,
                same_addr: false,
                instance_addr: None,
            });
        }
    }

    pub fn send(&self, packet: CentralClientPacket) {
        self.outbound.send(packet);
    }

    pub fn send_raw(&self, msg: tokio_tungstenite::tungstenite::Message) {
        self.outbound.send_raw(msg);
    }

    pub fn close(mut self, reason: &'static str) {
        self.outbound.close_reason = reason;
    }
}
impl Drop for ClientConnection {
    fn drop(&mut self) {
        log::debug!(
            "{:?} disconnected: {}",
            self.client_id,
            self.outbound.close_reason
        );
        self.set_battlescape(None);
    }
}
