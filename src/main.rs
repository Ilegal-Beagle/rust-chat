use std::{
    thread,
    io,
    fs::File,
    io::{prelude::*, stdin},
    net::{TcpListener, TcpStream},
};
use eframe::egui;

fn main() -> eframe::Result<()> {

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )

//     println!("server? (y/n)");
//     let choice = get_input();
//     
//     if choice == "y" {
//         let _ = server();
//     } else {
//         let _ = client();
//     }
//     Ok(())
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

struct MyApp {
    name: String,
    text: String,
    age: u32,
}

struct Message {
    user_name: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            text: "".to_owned(),
            age: 42,
        }
    }
}

impl Default for Message {

    fn default() -> Self {
        Self {
            user_name: "default".to_owned(),
        }
    }
}

impl egui::Widget for &mut Message {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.vertical(|ui| {
            ui.label("username");
            ui.label("message");
        }).response;

        response
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::SidePanel::right("user_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("username");
            });
        });

        egui::TopBottomPanel::bottom("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.text);
                ui.button("send");
            });
        });

       egui::CentralPanel::default().show(ctx, |ui| {
           ui.add(&mut Message::default());
       });
    }
}
