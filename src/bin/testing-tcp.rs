use std::io::{self, Write};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    let address = "127.0.0.1:8000"; // Replace with your server's address and port
    let mut stream = TcpStream::connect(address)?;

    let request = b"Test=Nice\nTest2=nice2\rraw body bytes very nice\0"; // Example HTTP request

    stream.write_all(request)?;
    stream.flush().unwrap();

    Ok(())
}
