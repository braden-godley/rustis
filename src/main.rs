mod server;

use clap::{arg, command, Command};
use std::net::TcpStream;
use std::io::{self, Write, Read};

fn main() {
    let matches = command!()
        .subcommand(
            Command::new("server")
                .about("Runs the server")
                .arg(arg!(-t --threads <N> "number of tcp threads").default_value("10")),
        )
        .subcommand(
            Command::new("client")
                .about("Runs a client that will connect to the server")
                .subcommand(
                    Command::new("publish")
                        .about("Publish a message to a channel")
                        .arg(arg!(<channel> "Channel to publish to"))
                        .arg(arg!(<message> "Message to publish"))
                )
                .subcommand(
                    Command::new("subscribe")
                        .about("Subscribe to a channel")
                        .arg(arg!(<channel> "Channel to subscribe to"))
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("server") {
        let threads: usize = matches
            .get_one::<String>("threads")
            .expect("required")
            .parse::<usize>()
            .expect("Invalid thread number");

        server::start_server(threads)
            .expect("Failed to start Rustis!");

    } else if let Some(matches) = matches.subcommand_matches("client") {
        if let Some(matches) = matches.subcommand_matches("publish") {
            let channel = matches.get_one::<String>("channel").unwrap();
            let message = matches.get_one::<String>("message").unwrap();
            let mut stream = TcpStream::connect("127.0.0.1:7878")
                .expect("Failed to connect");

            let msg = format!("Rustis 1.0.0\npublish\n{}\n{}\n\n", channel, message);

            stream.write_all(msg.as_bytes()).unwrap();

            stream.flush().unwrap();

            echo_stream(&mut stream).expect("Failed to echo stream");
        } else if let Some(matches) = matches.subcommand_matches("subscribe") {
            let channel = matches.get_one::<String>("channel").unwrap();
            let mut stream = TcpStream::connect("127.0.0.1:7878")
                .expect("Failed to connect");

            let msg = format!("Rustis 1.0.0\nsubscribe\n{}\n\n", channel.trim());

            stream.write_all(msg.as_bytes()).unwrap();

            stream.flush().unwrap();

            echo_stream(&mut stream).expect("Failed to echo stream");
        }
    }
}

fn echo_stream(stream: &mut TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];
    let mut data = Vec::new();

    loop {
        let bytes_read = stream.read(&mut buffer)?;

        // End of stream
        if bytes_read == 0 {
            break; 
        }

        data.extend_from_slice(&buffer[..bytes_read]);

        if let Some(index) = data.windows(2).position(|window| window == b"\n\n") {
            // Process the packet
            let packet = &data[..=index+1];
            println!("{}", String::from_utf8_lossy(&packet));

            data.drain(..=index+1);
        }
    }

    Ok(())
}

