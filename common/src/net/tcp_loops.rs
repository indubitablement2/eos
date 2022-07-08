use super::packets::Packet;
use tokio::{
    io::*,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::mpsc::UnboundedReceiver,
};

pub async fn tcp_out_loop(mut outbound_tcp_buffer_receiver: UnboundedReceiver<Vec<u8>>, mut write: OwnedWriteHalf) {
    loop {
        if let Some(buf) = outbound_tcp_buffer_receiver.recv().await {
            // Write buffer.
            if let Err(err) = write.write_all(&buf).await {
                log::debug!("{} while writting buffer to tcp stream. Disconnecting...", err);
                break;
            }
        } else {
            log::debug!("Tcp sender channel shutdown. Disconnecting...");
            break;
        }
    }

    log::debug!("Tcp out loop shutdown.");
}

pub async fn tcp_in_loop<P>(inbound_tcp_sender: crossbeam::channel::Sender<P>, read: OwnedReadHalf)
where
    P: Packet,
{
    let mut buf_read = BufReader::new(read);

    let mut payload_buffer: Vec<u8> = Vec::new();
    loop {
        // Get a header.
        match buf_read.read_u16().await {
            Ok(next_payload_size) => {
                if next_payload_size == 0 {
                    log::debug!("Received a reliable packet of 0 byte. Ignoring...");
                    continue;
                }

                let next_payload_size = usize::from(next_payload_size);

                // Increase buffer if needed.
                if next_payload_size > payload_buffer.len() {
                    payload_buffer.resize(next_payload_size, 0);
                }

                // Get packet.
                match buf_read.read_exact(&mut payload_buffer[..next_payload_size]).await {
                    Ok(num) => {
                        if num != next_payload_size {
                            log::debug!("Peer disconnected while sending a tcp packet. Ignoring packet...");
                            break;
                        }

                        if inbound_tcp_sender
                            .send(P::deserialize(&payload_buffer[..next_payload_size]))
                            .is_err()
                        {
                            log::debug!("Inbound tcp packet sender channel shutdown. Disconnecting...");
                            break;
                        }
                    }
                    Err(err) => {
                        log::debug!("{} while reading tcp stream for a payload. Disconnecting...", err);
                        break;
                    }
                }
            }
            Err(err) => {
                log::debug!("{} while reading tcp stream for a header. Disconnecting...", err);
                break;
            }
        }
    }

    log::debug!("Tcp in loop shutdown.");
}
