use std::io::{BufRead, Read, Write, Result};
use std::thread;
use chrono;
use core::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::ErrorKind;
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
use uuid::Uuid;

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
  Response::from_file("image/png", File::open(file_path).unwrap())
}

fn handle_server(view_counts: Arc<Mutex<HashMap<String, i32>>>) {

    let entries = async_std::fs::read_dir("./");
    // Extract the filenames from the directory entries and store them in a vector
    let mut images = get_files_in_directory("./").unwrap();
    // only get PNG and JPG files
    images.retain(|x| x.contains(".png") || x.contains(".jpg"));

    for u in &images {
        //println!("Found image {:?}", u);
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

        let view_count_1 = Arc::clone(&view_counts);
        let view_count_2 = Arc::clone(&view_counts);

        if route.starts_with("/temp/") {
            let mut view_count = view_count_1.lock().unwrap();
            let entry = view_count.entry(route.to_string()).or_insert(6);
            let mut file_name = route.replace("/temp/", "");
            let mut image_path = format!("/static/{}", file_name);

            if *entry >= 5 || *entry == -1 {
                image_path = format!("/static/blank.jpg");
                file_name = "blank.jpg".to_string();
            } else {
                *entry += 1;
            }


            let image_url = format!("/temp/{}", file_name);
            let view_count_text = format!("View Count: {}", entry);

            let html = format!(r#"
                <html>
                    <head>
                        <style>
                            img {{
                                width: 1000px;
                                height: 1000px;
                                object-fit: cover;
                            }}
                            a {{
                                text-decoration: none;
                                color: #fff;
                                background-color: #3498db;
                                padding: 8px 16px;
                                border-radius: 4px;
                                transition: background-color 0.3s ease-in-out;
                            }}

                            a:hover {{
                                background-color: #2980b9;
                            }}

                        </style>
                    </head>
                    <body>
                        <img src="{}" />
                        <p>{}</p>
                        <a href="{}">Request Image</a>
                    </body>
                </html>
                 "#, image_path, view_count_text, image_url);

            return Response::html(html);
        }

              
        let mut html = String::from(r#"
            <html>
                <head>
                    <style>
                        body {
                            font-family: 'Arial', sans-serif;
                            background-color: #f0f0f0;
                            margin: 0;
                            padding: 20px;
                        }

                        #grid {
                            display: grid;
                            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
                            grid-gap: 50px;
                        }

                        .image-container {
                            position: relative;
                            overflow: hidden;
                            border-radius: 8px;
                            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
                        }

                        img {
                            width: 100%;
                            height: 100%;
                            object-fit: cover;
                            filter: grayscale(30%) blur(5px);
                            transition: transform 0.3s ease-in-out;
                        }

                        .image-container:hover img {
                            transform: scale(1.1);
                        }

                        .overlay {
                            position: absolute;
                            top: 0;
                            left: 0;
                            width: 100%;
                            height: 100%;
                            display: flex;
                            flex-direction: column;
                            justify-content: center;
                            align-items: center;
                            opacity: 0;
                            background: rgba(0, 0, 0, 0.5);
                            color: #fff;
                            border-radius: 8px;
                            transition: opacity 0.3s ease-in-out;
                        }

                        .image-container:hover .overlay {
                            opacity: 1;
                        }

                        p {
                            margin: 10px 0;
                            font-size: 14px;
                        }

                        a {
                            text-decoration: none;
                            color: #fff;
                            background-color: #3498db;
                            padding: 8px 16px;
                            border-radius: 4px;
                            transition: background-color 0.3s ease-in-out;
                        }

                        a:hover {
                            background-color: #2980b9;
                        }
                    </style>
                </head>
                <body>
                    <div id="grid">
            "#);
        for img in &images {
            let filename = img.split('/').last().unwrap(); 

            //view_count_2.lock().unwrap().insert(format!("/temp/{}", filename), 0);

            html.push_str(&format!(r#"<div>
               <img src="/static/{}" />  
               <p>{}</p>
               <a href="{}">Request Image</a> 
               </div>"#, img, filename, format!("/temp/{}", filename)));

        }

        html.push_str(r#"</div></body></html>"#);
       
        Response::html(html)
    }).unwrap();

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

    let server_addresses = &["127.0.0.1:8887", "127.0.0.1:8888", "127.0.0.1:8889"]; // Replace with the server IPs.

    println!("Started successfully, broadcasting on port 8888");

    let mut input = String::new();

    let socket_m = Arc::new(Mutex::new(UdpSocket::bind("0.0.0.0:1939").expect("Failed to bind socket")));
    let socket_clone = socket_m.clone();
    let socket_clone2 = socket_m.clone();

    let mut view_counts: Arc<Mutex<HashMap<String, i32>>> = Arc::new(Mutex::new(HashMap::new()));
    let view_counts_clone_1 = view_counts.clone();
    let view_counts_clone_2 = view_counts.clone();

    let handle_website = thread::spawn(move || {
        handle_server(view_counts_clone_1);
    });

    let handle_requests_recieved = thread::spawn(move || {
        let mut buf_2 = [0u8; 65507];
        loop {
            //let result = socket_d_clone_1.lock().unwrap().recv_from(&mut buf_2);
            let result = socket_clone.lock().unwrap().recv_from(&mut buf_2);
            match result {
                Ok((amt, src)) => println!("Received request from {}: {}", src, String::from_utf8_lossy(&buf_2[..amt])),
                Err(err) => eprintln!("Error receiving request: {}", err),
            }
        }
    });

    let handle_input = thread::spawn(move || {
        loop {
            //println!("Enter a command (e.g., 'send <image_name>', 'list', 'lease <image_name>', 'request <image_name> <host_ip>): ");
            println!("Enter a command (e.g., 'send <image_name>', 'list', 'lease <image_name>'): ");
            io::stdin().read_line(&mut input);

            let parts: Vec<&str> = input.trim().split_whitespace().collect();

            match parts[0] {
                "send" => {
                    let image_path = parts[1];
                    for server_address in server_addresses {
                        if let Err(err) = send_image_to_server(server_address, image_path, &socket_clone2.lock().unwrap()) {
                            eprintln!("Error sending image to the server: {}", err);
                        }
                    }
                    println!("Image sent to the server.");
                }
                "list" => {
                    let socket = socket_clone2.lock().unwrap();
                    for server_address in server_addresses {
                        socket.send_to(b"list", server_address);
                    }
                    let mut buf = [0u8; 65507];
                    socket.set_read_timeout(Some(Duration::from_secs(10)));
                    let Ok((amt, _)) = socket.recv_from(&mut buf) else {
                        todo!()
                    };
                    println!("{}", str::from_utf8(&buf[..amt]).unwrap());
                    socket.set_read_timeout(None);
                }
                "lease" => {
                    view_counts_clone_2.lock().unwrap().insert(format!("/temp/{}", parts[1].to_string()), 0);
                    println!("Lease granted.");
                }
                "request" => {
                    let mut buf = [0u8; 65507];
                    let socket = socket_clone2.lock().unwrap();
                    socket.send_to(format!("Can I have {}?", parts[1]).as_bytes(), format!("{}:1939", parts[2]));
                    println!("Request sent.");
                }
                _ => {
                    println!("Invalid command.");
                }
            }

            input.clear();
        }

    });

    handle_website.join().unwrap();
    handle_input.join().unwrap();
    handle_requests_recieved.join().unwrap();
    Ok(())
}
