// network.rs
use std::{
    thread,
    io::{prelude::*, stdin},
    net::{TcpListener, TcpStream, SocketAddr}, //, IpAddr, Ipv4Addr},
    time::{Duration},
};

pub fn server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Started listening on port 7878");

    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(|| {
            let _ = handle_connection(stream);
        });
    }
    Ok(())
}

pub fn try_connect(address: &SocketAddr, timeout: Duration) -> bool {
    match TcpStream::connect_timeout(address, timeout) {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn client() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    println!("connected to server.");
    
    println!("Enter a Username: ");
    let username = get_input();

    loop {
        println!("enter a message: ");
        let message = get_input();
        let packet = format!("{username}: {message}\n");
        stream.write_all(packet.as_bytes())?;

        let mut resp = vec![0u8; 128];
        let n = stream.read(&mut resp)?;

        if message == "goodbye" || message.len() == 0 {
            break;
        }

        if n > 0 {
            println!("server replied: {}", String::from_utf8_lossy(&resp[..n]));
        }
    }
    Ok(())
}

pub fn handle_connection(mut stream: TcpStream) -> std::io::Result<()>{
    let mut buf = [0u8; 128];
    println!("client connected");

    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 { break; }

        let msg = String::from_utf8_lossy(&buf[..n]);
        stream.write_all(msg.as_bytes())?;
    }

    println!("client disconnected");
    Ok(())
}

pub fn get_input() -> String {
    let mut buffer:String = String::new();
    let _ = stdin().read_line(&mut buffer);
    return buffer.trim().to_string();
}

