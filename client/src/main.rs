use std::io::{BufRead, Read, Write, Result};
use std::thread;
use chrono;
use core::time::Duration;
use std::collections::HashMap;

use std::net::UdpSocket;
use std::env;
use std::str;
use std::fs::File;
use std::{fs, io};


use std::prelude::*;
use std::path::Path;
use image::{ImageBuffer, DynamicImage, ImageFormat};
use rocket::response::content;
use rocket::{Rocket, get, launch, routes, Build};
use rocket::http::ContentType;
use image::GenericImageView;
use build_html::{HtmlPage, Html};
use base64::encode;
use std::net::{TcpListener, TcpStream};
use image::open;
use std::str::from_utf8;
use glob::{glob_with, MatchOptions};
use error_chain::error_chain;
use rouille::Response;
use rouille::Server;

fn get_files_in_directory(path: &str) -> io::Result<Vec<String>> {
    // Get a list of all entries in the folder
    let entries = fs::read_dir(path)?;

    // Extract the filenames from the directory entries and store them in a vector
    let file_names: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();


    Ok(file_names)
}


fn serve_static_file(route: &str) -> Response {
  let file_path = format!(".{}", route);
  println!("file_path: {}", file_path);

  Response::from_file("image/png", File::open(file_path).unwrap())
}

fn handle_server() {

    let entries = async_std::fs::read_dir("./");
    // Extract the filenames from the directory entries and store them in a vector
    let mut images = get_files_in_directory("./").unwrap();
    // only get PNG and JPG files
    images.retain(|x| x.contains(".png") || x.contains(".jpg"));

    for u in &images {
        println!("Found image {:?}", u);
        let mut image_resp = Response::from_file("image/png", File::open(u).unwrap());
        image_resp = image_resp.with_content_disposition_attachment("image.png");
    }

    // Get all image files in current directory
    let server = Server::new("127.0.0.1:8080", move |request| {
        let route = request.url();

        // Check if the request is for a static file
        if route.starts_with("/static/") {
            return serve_static_file(route.as_str());
        }
                
        let mut html = String::from(r#"<html><head><style>
    #grid {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      grid-gap: 10px;
    }
    img {
      width: 200px; 
      height: 200px; 
      object-fit: cover;
      filter: blur(5px);
    }
  </style></head><body>
    <div id="grid">"#);

        for img in &images {
            let filename = img.split('/').last().unwrap(); 
            html.push_str(&format!(r#"<div>
        <img src="/static/{}" />
        <p style="text-align:center;">{}</p>  
      </div>"#, img, filename));
        }

        html.push_str(r#"</div></body></html>"#);
       
        Response::html(html)
    }).unwrap();

    println!("Listening on {:?}", server.server_addr());
    server.run();
}

fn decrement_port(addr: &str) -> String {
    let mut parts = addr.split(':');
    let ip = parts.next().unwrap();
    let mut port = parts.next().unwrap().chars().collect::<Vec<char>>();
    port[0] = (port[0].to_digit(10).unwrap() - 1).to_string().chars().next().unwrap();
    format!("{}:{}", ip, port.iter().collect::<String>())
}

fn send_image_to_server(server_address: &str, image_path: &str, socket: &UdpSocket) -> Result<()> {
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

    let socket = UdpSocket::bind("0.0.0.0:1939").expect("Failed to bind socket");

    
    let handle_website = thread::spawn(move || {
        handle_server();
    });

    handle_website.join().expect("Website thread panicked");

    loop {
        println!("Enter a command (e.g., 'send <image_name>', 'list'): ");
        io::stdin().read_line(&mut input)?;

        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        match parts[0] {
            "send" => {
                let image_path = parts[1];
                for server_address in server_addresses {
                    if let Err(err) = send_image_to_server(server_address, image_path, &socket) {
                        eprintln!("Error sending image to the server: {}", err);
                    }
                }
                println!("Image sent to the server.");
            }
            "list" => {
                for server_address in server_addresses {
                    socket.send_to(b"list", server_address)?;
                }
                let mut buf = [0u8; 65507];
                socket.set_read_timeout(Some(Duration::from_secs(10)))?;
                let (amt, _) = socket.recv_from(&mut buf)?;
                println!("{}", str::from_utf8(&buf[..amt]).unwrap());
                socket.set_read_timeout(None)?;
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
