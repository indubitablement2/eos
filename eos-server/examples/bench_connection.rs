use eos_common::{connection_manager::PollingThread, const_var::*, packet_mod::*};
use std::{
    net::TcpStream,
    time::{Duration, Instant},
};

fn main() {
    println!("Number of connections to test?");

    // Wait for input.
    let mut buffer = String::with_capacity(32);
    std::io::stdin().read_line(&mut buffer).unwrap();

    // Remove new line.
    if buffer.ends_with('\n') {
        buffer.pop();
    }
    if buffer.ends_with('\r') {
        buffer.pop();
    }

    // Separate letter from value.
    let mut value_str = String::with_capacity(32);
    for c in buffer.chars().rev() {
        if c.is_ascii_digit() {
            value_str.push(c);
        } else {
            break;
        }
    }

    // Get the numbers at the end if any.
    let num_client: usize = value_str
        .chars()
        .rev()
        .collect::<String>()
        .parse::<usize>()
        .unwrap_or_default()
        .clamp(1, 10000);

    println!("Bench {} client...", num_client);

    let now = Instant::now();

    let mut connections = Vec::with_capacity(num_client);

    println!("Connecting...");

    for i in 0..num_client {
        // Connect to server.
        let socket = TcpStream::connect(SERVER_ADDR).unwrap();
        // Make a thread and create connection with server.
        let mut pt = PollingThread::new(false);
        let con = pt
            .connection_starter
            .create_connection(socket, "127.0.0.1:8080".parse().unwrap())
            .unwrap();
        // Send ClientHello.
        assert!(con.send_packet(
            ClientLoginPacket::Hello {
                username: i.to_string(),
                app_version: APP_VERSION,
            }
            .serialize()
        ));
        // Send.
        std::thread::sleep(Duration::from_millis(5));
        assert!(pt.poll());

        connections.push((con, pt));

        if i % (num_client / 10).max(1) == 0 {
            println!("{}/{}", i, num_client);
        }
    }

    println!("{} connection done in {:?}", num_client, now.elapsed());

    println!("Send anything to end...");
    std::io::stdin().read_line(&mut buffer).unwrap();
}
