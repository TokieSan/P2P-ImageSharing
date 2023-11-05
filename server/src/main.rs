use std::io;
use std::time;
//use std::net::{TcpListener, TcpStream};
use std::net::{UdpSocket, SocketAddr};
use std::io::{Read, Write, Result};
use std::thread;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs::{File, create_dir_all};
use chrono::Utc;

//fn handle_client(
    //mut stream: TcpStream,
    //client_id: usize,
    //clients: Arc<Mutex<HashMap<usize, TcpStream>>>,
//) -> io::Result<()> {
    //println!("Client {} connected from {}.", client_id, stream.peer_addr()?);

    //let mut buf = [0; 1024];
    //loop {
        //let bytes_read = stream.read(&mut buf)?;

        //if bytes_read == 0 {
            //println!("Client {} disconnected.", client_id);
            //clients.lock().unwrap().remove(&client_id);
            //return Ok(());
        //}

        //let command = String::from_utf8_lossy(&buf[..bytes_read]).trim().to_string().to_owned();

        //match command.as_str() {
            //"list" => {
                //let client_list = clients.lock().unwrap();
                //let client_info: Vec<String> = client_list.iter().map(|(id, client)| {
                    //format!("Client {}: {}", id, client.peer_addr().unwrap())
                //}).collect();
                //let response = client_info.join("\n");
                //stream.write(response.as_bytes())?;
            //}
            //_ => {
                //// Handle other commands or ignore unknown commands
            //}
        //}
    //}
//}

fn write_image_to_file(data: &[u8], client_addr: &SocketAddr) -> Result<()> {
    // Get the current UTC date and time as a string.
    let current_datetime = Utc::now().to_rfc3339();

    // Create a file name using the client's address and the current date and time.
    let filename = format!("received_{}_{}.jpg", client_addr, current_datetime);
    let filename_clone = filename.clone();

    // Create and write the image data to the file.
    let mut file = File::create(filename)?;
    file.write_all(data)?;

    println!("Image saved as: {}", filename_clone);
    Ok(())
}

fn main() -> io::Result<()> {
    println!("Starting server...");

    //let listener = TcpListener::bind("127.0.0.1:7878")?;

    println!("Write the ip:port to bind to:");
    let mut input_= String::new();
    io::stdin().read_line(&mut input_)?;

    let input_server = input_.trim().to_string();
    let server_address = input_server.clone();

    let socket = UdpSocket::bind(input_server).expect("Failed to bind socket");
    let mut buf = [0u8; 65507];

    let server_addresses = &[
        "0.0.0.0:8888", 
        "0.0.0.0:8889", 
        "0.0.0.0:8887", 
    ];

    let shared_directory = "shared_directory"; // Directory for shared coordination.

    // Create a directory for coordination (if it doesn't exist).
    create_dir_all(shared_directory).expect("Failed to create shared directory");


    let next_server = match server_addresses.iter().position(|&addr| addr == server_address) {
        Some(index) => (index + 1) % server_addresses.len(),
        None => {
            eprintln!("Server address not found in the list.");
            return Ok(());
        }
    };


    let next_server_address: String = server_addresses[next_server].parse().expect("Invalid server address");
    
    let leader_addr = "0.0.0.0:8888";
    let handle = thread::spawn(move || {
        println!("Listening for clients...");
        loop {
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    println!("Received {} bytes from: {}", amt, src);

                    // Relay to next server so we can find who is the leader
                    //if let Err(err) = socket.send_to(&buf[..amt], &next_server_address) {
                        //eprintln!("Error relaying image data to the next server: {}", err);
                    //}                
                    // Recieve Responses from previous server

                    // Am I the leader? If so write to file, if not do nothing.
                    if server_address == leader_addr {
                        if let Err(err) = write_image_to_file(&buf[..amt], &src) {
                            eprintln!("Error writing image data to file: {}", err);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving data: {}", e);
                }
            }
        }
    });

    // Wait for the handling thread to finish.
    handle.join().expect("Thread join failed");
    println!("Shutting down server...");

    Ok(())
}
