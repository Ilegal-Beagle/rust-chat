use std::{
    thread,
    io::{prelude::*, stdin},
    net::{TcpListener, TcpStream},
};

fn main() {
    // read stdin
    println!("server? (y/n)");
    let mut buffer:String = String::new();
    let _ = stdin().read_line(&mut buffer);
    let choice: &str = buffer.trim();
    
    if choice == "y" {
        let _ = server();
    } else {
        let _ = client();
   }
}

fn server() -> std::io::Result<()> {
    let listener:TcpListener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Started listening on port 7878");

    for stream in listener.incoming() {
        let stream = stream?;
        println!("client connected");
        thread::spawn(|| {
            let _ = handle_connection(stream);
        });
        println!("client disconnected");
    }
    Ok(())
}

fn client() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    println!("connected to server. type a message");

    loop {
        let mut buffer = String::new();
        let _ = stdin().read_line(&mut buffer);
        let message = buffer.trim();

        stream.write_all(message.as_bytes())?;

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

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()>{
    let mut buf = [0u8; 128];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 { break; }
        let msg = String::from_utf8_lossy(&buf[..n]);
        
        println!("received {} bytes: {}", n, msg);
        
        stream.write_all(msg.as_bytes())?;
    }
    Ok(())
}
