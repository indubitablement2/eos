use super::packets::Packet;
use crate::net::MAX_UNRELIABLE_PACKET_SIZE;
use std::net::SocketAddr;
use std::net::UdpSocket;

pub fn udp_in_loop<P>(socket: UdpSocket, inbound_udp_sender: crossbeam::channel::Sender<(SocketAddr, P)>)
where
    P: Packet,
{
    let mut buf = [0; MAX_UNRELIABLE_PACKET_SIZE];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((num, addr)) => {
                if inbound_udp_sender.send((addr, P::deserialize(&buf[..num]))).is_err() {
                    log::debug!("Inbound udp packet sender channel shutdown. Terminating loop...");
                    break;
                }
            }
            Err(err) => {
                let kind = err.kind();
                if kind != std::io::ErrorKind::Interrupted {
                    log::error!("{} while receiving udp packet. Terminating in loop...", err);
                    break;
                }
            }
        }
    }

    log::info!("Udp in loop shutdown.");
}

pub fn udp_out_loop(socket: UdpSocket, outbound_udp_receiver: crossbeam::channel::Receiver<(SocketAddr, Vec<u8>)>) {
    loop {
        match outbound_udp_receiver.recv() {
            Ok((addr, buf)) => match socket.send_to(&buf, addr) {
                Ok(_) => {}
                Err(err) => {
                    let kind = err.kind();
                    if kind != std::io::ErrorKind::Interrupted {
                        log::error!("{} while sending udp packet. Terminating udp out loop...", err);
                        break;
                    }
                }
            },
            Err(_) => {
                break;
            }
        }
    }

    log::info!("Udp out loop shutdown.");
}
