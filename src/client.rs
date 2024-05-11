use std::net::TcpStream;
use std::io::{Read, Write, self};

pub struct Client {
    version: String,
    stream: TcpStream,
}

enum EchoType {
    Once,
    Loop,
}

impl Client {
    pub fn new(host: &str, port: u16, version: &str) -> Self {
        let connection_url = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&connection_url)
            .expect(&format!("Failed to connect to {}", &connection_url));
        let version = version.to_string();

        Client {
            stream,
            version,
        }
    }

    pub fn send_publish(&mut self, channel: &str, message: &str) -> io::Result<()> {
        let command = "publish";

        self.send(command, vec![
            channel.trim(),
            message.trim(),
        ])?;

        self.echo_response(EchoType::Once)?;

        Ok(())
    }

    pub fn send_subscribe(&mut self, channel: &str) -> io::Result<()> {
        let command = "subscribe";

        self.send(command, vec![
            channel.trim(),
        ])?;

        self.echo_response(EchoType::Loop)?;

        Ok(())
    }

    pub fn send_set(&mut self, key: &str, value: &str) -> io::Result<()> {
        let command = "set";

        self.send(command, vec![
            key.trim(),
            value.trim(),
        ])?;

        self.echo_response(EchoType::Once)?;

        Ok(())
    }

    pub fn send_setex(&mut self, key: &str, value: &str, ttl: u64) -> io::Result<()> {
        let command = "setex";

        let ttl = &ttl.to_string();

        self.send(command, vec![
            key.trim(),
            ttl,
            value.trim(),
        ])?;

        self.echo_response(EchoType::Once)?;

        Ok(())
    }

    pub fn send_get(&mut self, key: &str) -> io::Result<()> {
        let command = "get";

        self.send(command, vec![
            key.trim(),
        ])?;

        self.echo_response(EchoType::Once)?;

        Ok(())
    }

    pub fn send_ttl(&mut self, key: &str) -> io::Result<()> {
        let command = "ttl";

        self.send(command, vec![
            key.trim(),
        ])?;

        self.echo_response(EchoType::Once)?;

        Ok(())
    }

    pub fn send(&mut self, command: &str, lines: Vec<&str>) -> io::Result<()> {
        let version_line = format!("Rustis {}", self.version);
        let message = format!("{}\n{}\n{}\n\n", version_line, command, lines.join("\n"));

        let bytes = message.as_bytes();
        self.stream.write_all(bytes)?;
        Ok(())
    }

    fn echo_response(&mut self, echo_type: EchoType) -> io::Result<()> {
        let mut buffer = [0; 1024];
        let mut data = Vec::new();


        loop {
            let bytes_read = self.stream.read(&mut buffer)?;
            // End of stream
            if bytes_read == 0 {
                break; 
            }

            data.extend_from_slice(&buffer[..bytes_read]);

            if let Some(index) = data.windows(2).position(|window| window == b"\n\n") {
                // Process the packet
                let packet = &data[..=index-1];
                println!("{}", String::from_utf8_lossy(&packet));

                data.drain(..=index+1);
                if let EchoType::Once = echo_type {
                    break;
                }
            }
        }

        Ok(())
    }
}
