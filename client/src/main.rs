use std::io;
use std::net::TcpStream;
use std::io::Read;
use std::io::Write;

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    loop {
        // Read the image name from stdin
        let mut image_name = String::new();
        println!("Enter the image name (or 'exit' to quit): ");
        io::stdin().read_line(&mut image_name)?;
        image_name = image_name.trim().to_string();

        if image_name == "exit" {
            break; // Exit the loop if 'exit' is entered
        }

        // Try to read the image file and send it to the server
        match std::fs::read(&image_name) {
            Ok(img_data) => {
                stream.write(&img_data)?;
                let mut buffer = [0; 1024 * 1024];
                stream.read(&mut buffer)?;
                println!("Response from server: {} bytes", buffer.len());
            }
            Err(e) => {
                eprintln!("Error reading image file: {}", e);
            }
        }
    }

    Ok(())
}
