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
use serde::{Serialize, Deserialize};
use bincode;

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

#[derive(Serialize, Deserialize, Debug)]
struct ServerMessage {
    sender: i32,
    client: String,
    data: Vec<u8>,
    msg_type: i32, // 0 broadcast question, 1 information
}

// Function to determine if the message is serialized or direct.
fn is_serialized_message(data: &[u8]) -> bool {
    deserialize_message(&data).is_ok()
}

// Function to deserialize the message.
fn deserialize_message(data: &[u8]) -> Result<ServerMessage> {
  bincode::deserialize(data)
     .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

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

    let my_index : i32 = server_addresses.iter().position(|&s| s == server_address).unwrap() as i32;

    let mut processed: HashMap<(String, usize), bool> = HashMap::new(); // first is client second
                                                                        // is message size

    let handle = thread::spawn(move || {
        println!("Listening for clients...");
        let mut current_leader: i32 = -1;
        loop {
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    // TODO Split it to two functions, one handle if the message is serialized and
                    // the other if it is not (server vs client message)

                    println!("Received {} bytes from: {}", amt, src);

                    let message = ServerMessage {
                        sender: my_index,
                        msg_type: 0,
                        client: src.clone().to_string(),
                        data: buf[..amt].to_vec(),
                    };

                    // Serialize the struct into a byte array.
                    let serialized_message = bincode::serialize(&message).unwrap();

                    if deserialize_message(&serialized_message).is_ok() {
                        println!("Message is serialized.");
                    } else {
                        println!("Message is not serialized.");
                    }

                    // Relay to next server so we can find who is the leader
                    if current_leader == -1 {
                        // Broadcast to all servers to find the leader, if not I am the leader
                    }

                    if current_leader == my_index || current_leader == -1 {
                        let cur_client = src.to_string().split(':').next().unwrap().to_owned();
                        if !processed.contains_key(&(cur_client.clone(), amt)) {
                            if let Err(err) = write_image_to_file(&buf[..amt], &src) {
                                eprintln!("Error writing image data to file: {}", err);
                            }
                            processed.insert((cur_client, amt), true);
                        }
                        current_leader = my_index;
                        // let all of them know i am the leader
                    } else {
                        // is the leader alive? if so, do nothing he should have recieved the
                        // message and processed it
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
