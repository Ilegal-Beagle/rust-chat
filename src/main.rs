use std::{
    thread,
    io,
    fs::File,
    io::{prelude::*, stdin},
    net::{TcpListener, TcpStream},
};

fn main() -> io::Result<()> {
    println!("server? (y/n)");
    let choice = get_input();
    
    if choice == "y" {
        let _ = server();
    } else {
        let _ = client();
    }
    Ok(())
}

fn server() -> std::io::Result<()> {
    let listener:TcpListener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Started listening on port 7878");

    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(|| {
            let _ = handle_connection(stream);
        });
    }
    Ok(())
}

fn client() -> std::io::Result<()> {
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

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()>{
    let mut file = File::options()
        .read(true)
        .write(true)
        .open("../chat/chat.txt")?;
    let mut file_buf = [0u8; 512];
    let mut buf = [0u8; 128];
    println!("client connected");

    loop {
        let n = stream.read(&mut buf)?;
        let file_n = file.read(&mut file_buf)?;
        if n == 0 || file_n == 0 { break; }

        let msg = String::from_utf8_lossy(&buf[..n]);
        file.write_all(msg.as_bytes())?;
        stream.write_all(&file_buf[..file_n])?;
    }

    println!("client disconnected");
    Ok(())
}

fn get_input() -> String {
    let mut buffer:String = String::new();
    let _ = stdin().read_line(&mut buffer);
    return buffer.trim().to_string();
}


