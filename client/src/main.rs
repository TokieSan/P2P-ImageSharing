use std::io;
use std::net::TcpStream;
use std::io::Read;
use std::io::Write;

fn main() -> io::Result<()> {

  let mut stream = TcpStream::connect("127.0.0.1:7878")?;
  
  let img_data = std::fs::read("image.png")?;

  stream.write(&img_data)?;

  let mut buffer = [0; 1024 * 1024];
  
  stream.read(&mut buffer)?;

  println!("Response from server: {} bytes", buffer.len());

  Ok(())
}
