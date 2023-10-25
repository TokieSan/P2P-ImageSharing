use std::io;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::thread;
use chrono;
use core::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn receive_images_from_server(mut stream: TcpStream, client_id: usize) -> io::Result<()> {
    let mut buf = [0; 1024 * 1024];

    loop {
        let bytes_read = stream.read(&mut buf)?;

        if bytes_read == 0 {
            println!("Server disconnected.");
            return Ok(());
        }

        let image_name = format!("received-{}.png", chrono::Utc::now().timestamp());

        println!("Saving image as: {}", image_name);

        std::fs::write(&image_name, &buf[..bytes_read])?;

        println!("Image saved.");

        thread::sleep(Duration::from_secs(5));
    }
}

fn main() -> io::Result<()> {
    println!("Starting client...");

    let server_address = "127.0.0.1:7878";
    let mut stream = TcpStream::connect(server_address)?;

    println!("Connected to server at {}", server_address);

    let mut input = String::new();
    let client_id: Option<usize> = None;

    let client_id_to_stream: Arc<Mutex<HashMap<usize, TcpStream>>> =
        Arc::new(Mutex::new(HashMap::new()));
    client_id_to_stream.lock().unwrap().insert(0, stream.try_clone()?);

    let listener_stream = stream.try_clone()?;
    let client_id_to_stream_clone = client_id_to_stream.clone();
    let mut listening = false;
    let listen_thread = thread::spawn(move || {
        let mut buf = [0; 1024 * 1024];
        loop {
            if listening {
                let mut client_map = client_id_to_stream_clone.lock().unwrap();
                for (client_id, client_stream) in client_map.iter_mut() {
                    let bytes_read = client_stream.read(&mut buf).unwrap_or(0);

                    if bytes_read > 0 {
                        let image_name = format!("received-{}.png", chrono::Utc::now().timestamp());
                        println!("Received image from Client {}: Saving as {}", client_id, image_name);
                        std::fs::write(&image_name, &buf[..bytes_read]).unwrap();
                    }
                }
            }
            thread::sleep(Duration::from_secs(2));
        }
    });

    loop {
        println!("Enter a command (e.g., 'send <client_id> <image_name>', 'list', or 'listen'): ");
        io::stdin().read_line(&mut input)?;

        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        match parts[0] {
            "send" => {
                if parts.len() >= 3 {
                    if let Ok(client_id) = parts[1].parse::<usize>() {
                        let image_name = parts[2];
                        let mut client_map = client_id_to_stream.lock().unwrap();
                        if let Some(client_stream) = client_map.get_mut(&client_id) {
                            let image_data = std::fs::read(image_name)?;
                            client_stream.write(&image_data)?;
                            println!("Image sent to Client {}.", client_id);
                        } else {
                            println!("Client {} not found or not connected.", client_id);
                        }
                    } else {
                        println!("Invalid client ID.");
                    }
                } else {
                    println!("Please specify a client ID and an image name.");
                }
            }
            "list" => {
                stream.write(b"list")?;
                let mut response = [0; 1024];
                let bytes_read = stream.read(&mut response)?;
                if bytes_read > 0 {
                    let response_str = String::from_utf8_lossy(&response[..bytes_read]);
                    println!("Available clients and their IPs:");
                    println!("{:<10}{}", "Client ID", "IP Address");
                    for line in response_str.lines() {
                        println!("{}", line);
                    }
                }
            }
            "listen" => {
                listening = true;
            }
            _ => {
                println!("Invalid command. Try 'send <client_id> <image_name>', 'list', or 'listen'.");
            }
        }

        input.clear();
    }
}
