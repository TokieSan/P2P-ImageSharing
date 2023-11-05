use std::io;
use std::net::TcpStream;
use std::io::{BufRead, Read, Write, Result};
use std::thread;
use chrono;
use core::time::Duration;
use std::collections::HashMap;

use std::fs::File;
use std::net::UdpSocket;
use std::env;
use std::str;



//fn receive_images_from_server(mut stream: TcpStream, client_id: usize) -> io::Result<()> {
    //let mut buf = [0; 1024 * 1024];

    //loop {
        //let bytes_read = stream.read(&mut buf)?;

        //if bytes_read == 0 {
            //println!("Server disconnected.");
            //return Ok(());
        //}

        //let image_name = format!("received-{}.png", chrono::Utc::now().timestamp());

        //println!("Saving image as: {}", image_name);

        //std::fs::write(&image_name, &buf[..bytes_read])?;

        //println!("Image saved.");

        //thread::sleep(Duration::from_secs(5));
    //}
//}

fn send_image_to_server(server_address: &str, image_path: &str) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    let mut file = File::open(image_path)?;

    // Read the image data from the file into a buffer.
    let mut image_data = Vec::new();
    file.read_to_end(&mut image_data)?;

    // Send the image data to the server at the specified address.
    socket.send_to(&image_data, server_address)?;

    Ok(())
}

fn main() -> io::Result<()> {
    println!("Starting client...");

    //let server_address = "127.0.0.1:8888";
    let server_addresses = &["127.0.0.1:8887", "127.0.0.1:8888", "127.0.0.1:8889"]; // Replace with the server IPs.

    println!("Started successfully, broadcasting on port 8888");

    let mut input = String::new();

    loop {
        println!("Enter a command (e.g., 'send <image_name>', 'list', or 'listen'): ");
        io::stdin().read_line(&mut input)?;

        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        match parts[0] {
            "send" => {
                let image_path = parts[1];
                for server_address in server_addresses {
                    if let Err(err) = send_image_to_server(server_address, image_path) {
                        eprintln!("Error sending image to the server: {}", err);
                    }
                }
                println!("Image sent to the server.");
            }
            "list" => {
                //stream.write(b"list")?;
                //let mut response = [0; 1024];
                //let bytes_read = stream.read(&mut response)?;
                //if bytes_read > 0 {
                    //let response_str = String::from_utf8_lossy(&response[..bytes_read]);
                    //println!("Available clients and their IPs:");
                    //println!("{:<10}{}", "Client ID", "IP Address");
                    //for line in response_str.lines() {
                        //println!("{}", line);
                    //}
                //}
            }
            "listen" => {
            }
            _ => {
                println!("Invalid command.");
            }
        }

        input.clear();
    }
}
