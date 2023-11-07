use std::io;
use std::cmp::max;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};
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
    cur_leader: i32,
    msg_type: i32, // 0 broadcast question, 1 information, 2 i am leader, 3 are leader alive
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

fn write_image_to_file(data: &[u8]) -> Result<()> {
    // Get the current UTC date and time as a string.
    let current_datetime = Utc::now().to_rfc3339();

    // Create a file name using the client's address and the current date and time.
    let filename = format!("received_{}.jpg", current_datetime);
    let filename_clone = filename.clone();

    // Create and write the image data to the file.
    let mut file = File::create(filename)?;
    file.write_all(data)?;

    println!("Image saved as: {}", filename_clone);
    Ok(())
}

struct SharedData {
    amt: u64,
    src: SocketAddr,
}

fn decrement_port(addr: &str) -> String {
  let mut parts = addr.split(':');
  let ip = parts.next().unwrap();
  let mut port = parts.next().unwrap().chars().collect::<Vec<char>>();

  port[0] = (port[0].to_digit(10).unwrap() - 1).to_string().chars().next().unwrap();

  format!("{}:{}", ip, port.iter().collect::<String>())
}

fn main() -> io::Result<()> {
    println!("Starting server...");

    //let listener = TcpListener::bind("127.0.0.1:7878")?;

    println!("Write the ip:port to bind to:");
    let mut input_= String::new();
    io::stdin().read_line(&mut input_)?;

    let input_server = input_.trim().to_string();
    let server_address = input_server.clone().to_string();

    let socket = UdpSocket::bind(input_server.clone()).expect("Failed to bind socket");
    let socket_server = UdpSocket::bind(decrement_port(&input_server.clone())).expect("Failed to bind socket");

    let mut buf = [0u8; 65507];

    let server_addresses = &[
        "0.0.0.0:8887", 
        "0.0.0.0:8888", 
        "0.0.0.0:8889", 
    ];

    let my_index : i32 = server_addresses.iter().position(|&s| s == server_address).unwrap() as i32;
    let mut current_leader: i32 = 0;

    let mut processed: HashMap<(String, usize), bool> = HashMap::new(); // first is client second
                 

    // Create a Mutex to safely share data between threads.
    //let amt_mutex = Arc::new(Mutex::new(0));
    //let src_mutex = Arc::new(Mutex::new("".to_string()));

    //// Clone Mutexes for the UDP listener thread.
    //let amt_mutex_clone = amt_mutex.clone();
    //let src_mutex_clone = src_mutex.clone();

    let leader_alive = Arc::new(AtomicBool::new(false));

    let leader_alive1 = Arc::clone(&leader_alive);
    let leader_alive2 = Arc::clone(&leader_alive);

    let handle_server = thread::spawn(move || {
        loop {
            match socket_server.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    println!("Received {} bytes from: {} at server thread", amt, src);

                    if is_serialized_message(&buf) {
                        let message = deserialize_message(&buf).unwrap();
                        println!("Message: Sent by: {} to: {} type: {} leader: {}", message.sender, message.client, message.msg_type, message.cur_leader);

                        let mut response = ServerMessage {
                            sender: my_index,
                            client: message.client.clone(),
                            cur_leader : current_leader,
                            data: vec![],
                            msg_type: 1,
                        };

                        if message.msg_type == 0 {
                            // broadcast question
                            println!("Broadcast question received from: {}", message.sender);
                            let serialized_response = bincode::serialize(&response).unwrap();
                            socket_server.send_to(&serialized_response, decrement_port(server_addresses[message.sender as usize])).expect("Failed to send response");
                        } else if response.msg_type == 3 {
                            //if message.cur_leader == my_index {
                                //response.sender = my_index;
                                //response.msg_type = 3;
                                //let serialized_response = bincode::serialize(&response).unwrap();
                                //socket_server.send_to(&serialized_response, decrement_port(server_addresses[message.sender as usize])).expect("Failed to send response");
                            //} else {
                                //leader_alive1.store(true, Ordering::SeqCst);
                            //}
                        } else if response.msg_type == 2 {
                            // information
                            current_leader = max(current_leader, message.cur_leader);
                            println!("Information received from: {} that {} is the leader", message.sender, current_leader);
                        }
                    }  
                }
                Err(e) => {
                    eprintln!("Error receiving data: {}", e);
                }  

            }
            
        }
    });

    let handle = thread::spawn(move || {
        println!("Listening for clients...");
        loop {
            // TODO Split it to two functions, one handle if the message is serialized and
            // the other if it is not (server vs client message)
            // In the future, with multiple devices, it will be according to port, now it
            // is a fixed list
        
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {

                    println!("Received {} bytes from: {} at clients thread", amt, src);

                    if !is_serialized_message(&buf) {
                        // Recieved an image from a client.
                        let mut message = ServerMessage {
                            sender: my_index,
                            msg_type: 0,
                            cur_leader: current_leader,
                            client: src.clone().to_string(),
                            data: buf[..amt].to_vec(),
                        };

                        // Serialize the struct into a byte array.
                        let serialized_message = bincode::serialize(&message).unwrap();

                        if current_leader == -1 {
                            // Broadcast to all servers to find the leader, if not I am the leader
                            for cur_server in server_addresses {
                                if cur_server != &server_addresses[my_index as usize] {
                                    socket.send_to(&serialized_message, decrement_port(cur_server.clone())).expect("Failed to send message");
                                }
                            }
                        }

                        std::thread::sleep(std::time::Duration::from_secs_f32(rand::random::<f32>() * 6.0 + 3.0));

                        if current_leader == my_index || current_leader == -1 {
                            let cur_client = src.to_string().split(':').next().unwrap().to_owned();
                            if !processed.contains_key(&(cur_client.clone(), amt)) {
                                if let Err(err) = write_image_to_file(&buf[..amt]) {
                                    eprintln!("Error writing image data to file: {}", err);
                                }
                                processed.insert((cur_client, amt), true);
                            } else {
                                println!("Already processed this image");
                            }
                            current_leader = my_index;
                            message.msg_type = 2;
                            message.cur_leader = current_leader;
                            let serialized_message = bincode::serialize(&message).unwrap();
                            // I am the leader
                            for cur_server in server_addresses {
                                socket.send_to(&serialized_message, decrement_port(cur_server.clone())).expect("Failed to send message");
                            }
                        } else {
                            // is the leader alive? if so, do nothing he should have recieved the
                            // message and processed it
                            message.msg_type = 3;
                            socket.send_to(&serialized_message, server_addresses[current_leader as usize].clone()).expect("Failed to send message");

                            let mut buf_2 = [0u8; 65507];

                            socket.set_read_timeout(Some(Duration::from_secs(5)))
                                .expect("Failed to set read timeout");

                            match socket.recv_from(&mut buf_2) {
                                Ok((amt_2, src_2)) => {
                                    let message_2 = deserialize_message(&buf_2).unwrap();
                                    println!("Message: Sent by: {} to: {} type: {} leader: {}", message_2.sender, message_2.client, message_2.msg_type, message_2.cur_leader);
                                    if message_2.msg_type == 3 {
                                        println!("Leader is alive");
                                    } else {
                                        println!("collision happened");
                                    }
                                } Err(_) => {
                                    // Leader is dead, I am the new leader
                                    leader_alive2.store(false, Ordering::SeqCst);
                                    let old_leader = current_leader;
                                    let message_3 = ServerMessage {
                                        sender: my_index,
                                        msg_type: 0,
                                        cur_leader: my_index,
                                        client: src.clone().to_string(),
                                        data: vec![],
                                    };
                                    let serialized_message_3 = bincode::serialize(&message_3).unwrap();
                                    println!("Leader {} is dead, I will see if there is some leader set", current_leader);
                                    for cur_server in server_addresses {
                                        if cur_server != &server_addresses[my_index as usize] {
                                            socket.send_to(&serialized_message_3, decrement_port(cur_server.clone())).expect("Failed to send message");
                                        }
                                    }

                                    std::thread::sleep(std::time::Duration::from_secs_f32(rand::random::<f32>() * 6.0 + 3.0));

                                    if current_leader == old_leader {
                                        // No leader, I am the leader
                                        current_leader = my_index;
                                        message.msg_type = 2;
                                        message.cur_leader = current_leader;
                                        let serialized_message = bincode::serialize(&message).unwrap();
                                        for cur_server in server_addresses {
                                            if cur_server != &server_addresses[my_index as usize] {
                                                socket.send_to(&serialized_message, decrement_port(cur_server.clone())).expect("Failed to send message");
                                            }
                                        }

                                        std::thread::sleep(std::time::Duration::from_secs_f32(rand::random::<f32>() * 3.0));
                                        let cur_client = src.to_string().split(':').next().unwrap().to_owned();
                                        if !processed.contains_key(&(cur_client.clone(), amt)) {
                                            if let Err(err) = write_image_to_file(&buf[..amt]) {
                                                eprintln!("Error writing image data to file: {}", err);
                                            }
                                            processed.insert((cur_client, amt), true);
                                        } else {
                                            println!("Already processed this image");
                                        }

                                    }
                                }
                            }
                            socket.set_read_timeout(None).expect("set_read_timeout call failed");

                        }
                    } else {
                        // Recieved a message from a server.
                        let message: ServerMessage = bincode::deserialize(&buf[..amt]).unwrap();
                        println!("Message: Sent by: {} to: {} type: {} leader: {}", message.sender, message.client, message.msg_type, message.cur_leader);

                        let mut response = ServerMessage {
                            sender: my_index,
                            client: message.client.clone(),
                            cur_leader : current_leader,
                            data: vec![],
                            msg_type: 3,
                        };

                        // message type is 3
                        current_leader = my_index;
                        let serialized_response = bincode::serialize(&response).unwrap();
                        socket.send_to(&serialized_response, server_addresses[message.sender as usize]).expect("Failed to send response");
                        leader_alive2.store(true, Ordering::SeqCst);
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
    handle_server.join().expect("Thread join failed");

    println!("Shutting down server...");

    Ok(())
}
