use std::io;
use std::time;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

fn handle_client(mut stream: TcpStream) -> io::Result<()> {

  println!("New client connected.");
  
  let mut buf = [0; 1024 * 1024];
  
  loop {
    let bytes_read = stream.read(&mut buf)?;

    if bytes_read == 0 {
        println!("Client disconnected.");
        return Ok(());
    }
    
    let image_name = format!("received-{}.png", chrono::Utc::now().timestamp());
    
    println!("Saving image as: {}", image_name);
    
    std::fs::write(image_name, &buf[..bytes_read])?;

    println!("Image saved.");

    stream.write(&buf[..bytes_read])?;

    println!("Response sent to client.");

    thread::sleep(time::Duration::from_secs(5));
  }
}

fn main() -> io::Result<()> {

  println!("Starting server...");

  let listener = TcpListener::bind("127.0.0.1:7878")?;

  println!("Listening for clients...");

  let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();

  for stream in listener.incoming() {
    let stream = stream.expect("failed");

    let handle = thread::spawn(move || {
        handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
    });
    
    thread_vec.push(handle);
  }

  println!("Shutting down server...");

  for handle in thread_vec {
    handle.join().unwrap();
  }

  println!("Server shutdown complete.");

  Ok(())
}
