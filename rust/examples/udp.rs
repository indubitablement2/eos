use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let so1 = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();

    let so2 = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();

    so1.connect(so2.local_addr().unwrap()).unwrap();
    so2.connect(so1.local_addr().unwrap()).unwrap();

    println!("so1: {:?} to {:?}", so1.local_addr().unwrap(), so1.peer_addr().unwrap());
    println!("so2: {:?} to {:?}", so2.local_addr().unwrap(), so2.peer_addr().unwrap());

    let mut buf = [0u8; 1024];

    so1.send("hello".to_string().as_bytes()).unwrap();
    sleep(Duration::from_millis(10));

    match so2.recv(&mut buf) {
        Ok(num) => {
            println!("{}", String::from_utf8(buf[..num].to_vec()).unwrap());
        }
        Err(err) => {
            println!("{:?}", &err);
        }
    }

    so1.send("hello".to_string().as_bytes()).unwrap();
    sleep(Duration::from_millis(10));
}
