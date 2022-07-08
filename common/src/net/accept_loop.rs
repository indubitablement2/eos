use crate::net::{tcp_connection::TcpConnection, *};
use std::time::Duration;
use tokio::spawn;

pub struct LoginSuccess {
    pub auth: Auth,
    pub udp_addr: SocketAddr,
    pub tcp_connection: TcpConnection<ClientPacket>,
}

pub async fn accept_loop(
    listener: tokio::net::TcpListener,
    login_sender: crossbeam::channel::Sender<LoginSuccess>,
) -> tokio::io::Result<()> {
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let tcp_connection = TcpConnection::new(stream).await;
                let login_sender = login_sender.clone();
                spawn(authenticate(tcp_connection, login_sender));
            }
            Err(err) => {
                log::error!(
                    "{} while accepting incoming connection. Terminating accept loop...",
                    err
                );
                break;
            }
        }
    }

    Ok(())
}

async fn authenticate(
    mut tcp_connection: TcpConnection<ClientPacket>,
    login_sender: crossbeam::channel::Sender<LoginSuccess>,
) {
    // Get the first packet fo the client.
    let mut num_wait = 0;
    let login_packet = loop {
        tokio::time::interval(Duration::from_millis(500)).tick().await;
        num_wait += 1;

        match tcp_connection.inbound_tcp_receiver.try_recv() {
            Ok(packet) => {
                if let ClientPacket::LoginPacket(login_packet) = packet {
                    break login_packet;
                } else {
                    tcp_connection.send_tcp_packet(&ServerPacket::LoginResponse(LoginResponse::FirstPacketNotLogin));
                    tcp_connection.flush_tcp_buffer();
                    return;
                }
            }
            Err(err) => {
                if err.is_disconnected() {
                    return;
                } else if num_wait > 10 {
                    tcp_connection.send_tcp_packet(&ServerPacket::LoginResponse(LoginResponse::LoginTimeOut));
                    tcp_connection.flush_tcp_buffer();
                    return;
                }
            }
        }
    };

    if let Some(auth) = login_packet.credential_checker.check().await {
        let _ = login_sender.send(LoginSuccess {
            auth,
            udp_addr: login_packet.requested_udp_addr,
            tcp_connection,
        });
    } else {
        tcp_connection.send_tcp_packet(&ServerPacket::LoginResponse(LoginResponse::BadCredential));
        tcp_connection.flush_tcp_buffer();
    }
}
