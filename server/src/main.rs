use std::io;
use std::cmp::max;
use std::time::Duration;
use std::net::{UdpSocket, SocketAddr};
use std::io::{Write, Result};
use std::thread;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs::File;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use bincode;

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
    let current_datetime = Utc::now().to_rfc3339();
    let filename = format!("received_{}.jpg", current_datetime);
    let filename_clone = filename.clone();
    let mut file = File::create(filename)?;
    file.write_all(data)?;

    println!("Image saved as: {}", filename_clone);
    Ok(())
}

fn decrement_port(addr: &str) -> String {
    let mut parts = addr.split(':');
    let ip = parts.next().unwrap();
    let mut port = parts.next().unwrap().chars().collect::<Vec<char>>();
    port[0] = (port[0].to_digit(10).unwrap() - 1).to_string().chars().next().unwrap();
    format!("{}:{}", ip, port.iter().collect::<String>())
}

fn is_leader_alive(
    socket: &UdpSocket,
    message: &mut ServerMessage,
    server_addresses: &&[&str],
    current_leader: Arc<Mutex<i32>>,
) -> bool {
    message.msg_type = 3;
    let serialized_message = bincode::serialize(&message).unwrap();
    socket
        .send_to(
            &serialized_message,
            server_addresses[*current_leader.lock().expect("Mutex lock failed") as usize],
        )
        .expect("Failed to send message");

    let mut buf_2 = [0u8; 65507];

    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("Failed to set read timeout");

    std::thread::sleep(std::time::Duration::from_secs_f32(rand::random::<f32>() * 2.0 + 1.0));
    match socket.recv_from(&mut buf_2) {
        Ok((_amt_2, _src_2)) => {
            let message_2 = deserialize_message(&buf_2).unwrap();
            println!(
                "Message: Sent by: {} to: {} type: {} leader: {}",
                message_2.sender, message_2.client, message_2.msg_type, message_2.cur_leader
            );
            if message_2.msg_type == 3 {
                println!("Leader is alive");
            } else {
                println!("Collision happened");
            }
            socket.set_read_timeout(None).expect("set_read_timeout call failed");
            true 
        }
        Err(_) => {
            socket.set_read_timeout(None).expect("set_read_timeout call failed");
            false
        }
    }
}

fn handle_server_messages(
    socket_server: &UdpSocket,
    server_addresses: &[&str],
    my_index: i32,
    current_leader: &Arc<Mutex<i32>>,
) {
    let mut buf = [0u8; 65507];

    loop {
        match socket_server.recv_from(&mut buf) {
            Ok((amt, src)) => {
                println!("Received {} bytes from: {} at server thread", amt, src);

                if is_serialized_message(&buf) {
                    let message = deserialize_message(&buf).unwrap();
                    println!(
                        "Message: Sent by: {} to: {} type: {} leader: {}",
                        message.sender, message.client, message.msg_type, message.cur_leader
                    );

                    let mut current_leader_mutex = current_leader.lock().expect("Mutex lock failed");
                    let mut response = ServerMessage {
                        sender: my_index,
                        client: message.client.clone(),
                        cur_leader: *current_leader_mutex,
                        data: vec![],
                        msg_type: 1,
                    };

                    if message.msg_type == 0 {
                        // broadcast question
                        println!("Broadcast question received from: {}", message.sender);
                        let serialized_response = bincode::serialize(&response).unwrap();
                        socket_server
                            .send_to(
                                &serialized_response,
                                decrement_port(server_addresses[message.sender as usize]),
                            )
                            .expect("Failed to send response");
                    } else if message.msg_type == 3 {
                        //*current_leader.lock().unwrap() = max(*current_leader.lock().unwrap(), message.sender);
                    } else if message.msg_type == 2 {
                        // information
                        *current_leader_mutex = max(*current_leader_mutex, message.sender);
                        println!(
                            "Information received from: {} that {} is the leader",
                            message.sender, *current_leader_mutex
                        );
                        response.msg_type = 3;
                        let serialized_response = bincode::serialize(&response).unwrap();
                        socket_server
                            .send_to(
                                &serialized_response,
                                decrement_port(server_addresses[message.sender as usize]),
                            )
                            .expect("Failed to send response");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
            }
        }
    }
}

fn handle_image_save(
    processed: &mut HashMap<(String, usize), bool>,
    src: &SocketAddr,
    buf: &[u8],
    amt: usize,
) {
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

fn get_new_leader(
    socket: &UdpSocket,
    server_addresses: &[&str],
    current_leader: &Arc<Mutex<i32>>,
    my_index: i32,
) -> i32 {
    let message = ServerMessage {
        sender: my_index,
        msg_type: 2,
        cur_leader: my_index,
        client: "".to_string(),
        data: vec![],
    };
    let serialized_message_3 = bincode::serialize(&message).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(5))).expect("set_read_timeout call failed");
    let mut act_leader = my_index;
    std::thread::sleep(std::time::Duration::from_secs_f32(rand::random::<f32>() * 2.0));
    for cur_server in server_addresses {
        if cur_server != &server_addresses[my_index as usize] {
            socket.send_to(&serialized_message_3, decrement_port(cur_server))
                .expect("Failed to send message");
            std::thread::sleep(std::time::Duration::from_secs_f32(rand::random::<f32>() * 8.0));
            act_leader = max(act_leader, *current_leader.lock().unwrap());
        }
    }
    println!("New leader is: {}", act_leader);
    *current_leader.lock().unwrap() = act_leader;
    socket.set_read_timeout(None).expect("set_read_timeout call failed");
    act_leader
}

fn handle_client_messages(
    socket: &UdpSocket,
    server_addresses: &[&str],
    my_index: i32,
    current_leader: &Arc<Mutex<i32>>,
) {
    let mut buf = [0u8; 65507];
    let mut processed: HashMap<(String, usize), bool> = HashMap::new();

    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                println!("Received {} bytes from: {} at clients thread", amt, src);

                if !is_serialized_message(&buf) {
                    // Received an image from a client.
                    let mut message = ServerMessage {
                        sender: my_index,
                        msg_type: 0,
                        cur_leader: *current_leader.lock().expect("Mutex lock failed"),
                        client: src.clone().to_string(),
                        data: buf[..amt].to_vec(),
                    };

                    if *current_leader.lock().expect("Mutex lock failed") == my_index {
                        handle_image_save(&mut processed, &src, &buf, amt);
                    } else {
                        if !is_leader_alive(&socket, &mut message, &server_addresses, Arc::clone(&current_leader)) {
                            println!("Leader is dead");
                            let leader_new = get_new_leader(&socket, &server_addresses, &Arc::clone(&current_leader), my_index);

                            if leader_new == my_index {
                                handle_image_save(&mut processed, &src, &buf, amt);
                            }
                        }
                        socket.set_read_timeout(None).expect("set_read_timeout call failed");
                    }
                } else {
                    // Received a message from a server.
                    let message: ServerMessage = bincode::deserialize(&buf[..amt]).unwrap();
                    println!(
                        "Message: Sent by: {} to: {} type: {} leader: {}",
                        message.sender, message.client, message.msg_type, message.cur_leader
                    );
                    *current_leader.lock().unwrap() = max(*current_leader.lock().unwrap(), message.cur_leader);

                    let response = ServerMessage {
                        sender: my_index,
                        client: message.client.clone(),
                        cur_leader: *current_leader.lock().expect("Mutex lock failed"),
                        data: vec![],
                        msg_type: 3,
                    };

                    let serialized_response = bincode::serialize(&response).unwrap();
                    socket.send_to(&serialized_response, server_addresses[message.sender as usize])
                        .expect("Failed to send response");
                }
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
            }
        }
    }
}

fn main() -> io::Result<()> {
    println!("Starting server...");

    println!("Write the ip:port to bind to:");
    let mut input_ = String::new();
    io::stdin().read_line(&mut input_)?;

    let input_server = input_.trim().to_string();
    let server_address = input_server.clone().to_string();

    let socket = UdpSocket::bind(input_server.clone()).expect("Failed to bind socket");
    let socket_server = UdpSocket::bind(decrement_port(&input_server.clone())).expect("Failed to bind socket");

    let server_addresses = &["0.0.0.0:8887", "0.0.0.0:8888", "0.0.0.0:8889"];
    let my_index: i32 = server_addresses.iter().position(|&s| s == server_address).unwrap() as i32;

    // Create an Arc to safely share data between threads.
    let current_leader = Arc::new(Mutex::new(0));

    let current_leader_clone1 = Arc::clone(&current_leader);
    let current_leader_clone2 = Arc::clone(&current_leader);

    let handle_server = thread::spawn(move || {
        handle_server_messages(&socket_server, server_addresses, my_index, &current_leader_clone1);
    });

    let handle = thread::spawn(move || {
        handle_client_messages(&socket, server_addresses, my_index, &current_leader_clone2);
    });

    let debugging_thread = thread::spawn(move || {
        loop {
            println!("[DEBUG] Current leader: {}", *current_leader.lock().unwrap());
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    });

    // Wait for the handling thread to finish.
    handle.join().expect("Thread join failed");
    handle_server.join().expect("Thread join failed");
    debugging_thread.join().expect("Thread join failed");

    println!("Shutting down server...");

    Ok(())
}
