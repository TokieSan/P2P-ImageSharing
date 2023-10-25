use std::io;
use std::time;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn handle_client(
    mut stream: TcpStream,
    client_id: usize,
    clients: Arc<Mutex<HashMap<usize, TcpStream>>>,
) -> io::Result<()> {
    println!("Client {} connected from {}.", client_id, stream.peer_addr()?);

    let mut buf = [0; 1024];
    loop {
        let bytes_read = stream.read(&mut buf)?;

        if bytes_read == 0 {
            println!("Client {} disconnected.", client_id);
            clients.lock().unwrap().remove(&client_id);
            return Ok(());
        }

        let command = String::from_utf8_lossy(&buf[..bytes_read]).trim().to_string().to_owned();

        match command.as_str() {
            "list" => {
                let client_list = clients.lock().unwrap();
                let client_info: Vec<String> = client_list.iter().map(|(id, client)| {
                    format!("Client {}: {}", id, client.peer_addr().unwrap())
                }).collect();
                let response = client_info.join("\n");
                stream.write(response.as_bytes())?;
            }
            _ => {
                // Handle other commands or ignore unknown commands
            }
        }
    }
}

fn main() -> io::Result<()> {
    println!("Starting server...");

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let clients: Arc<Mutex<HashMap<usize, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut client_id = 0;

    println!("Listening for clients...");

    for stream in listener.incoming() {
        let stream = stream.expect("failed");
        let clients_copy = Arc::clone(&clients);
        clients.lock().unwrap().insert(client_id, stream.try_clone().unwrap());

        let handle = thread::spawn(move || {
            handle_client(stream, client_id, clients_copy)
                .unwrap_or_else(|error| eprintln!("{:?}", error));
        });

        client_id += 1;
    }

    println!("Shutting down server...");

    Ok(())
}
