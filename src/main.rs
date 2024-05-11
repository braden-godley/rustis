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
                )
                .subcommand(
                    Command::new("set")
                        .about("Set a key's value in the KV store")
                        .arg(arg!(<key> "Key to set"))
                        .arg(arg!(<value> "New value"))
                )
                .subcommand(
                    Command::new("setex")
                        .about("Set a key's value in the KV store with an expiration time")
                        .arg(arg!(<key> "Key to set"))
                        .arg(arg!(<value> "New value"))
                        .arg(arg!(<ttl> "Expiration time to live")
                             .value_parser(clap::value_parser!(u64)))
                )
                .subcommand(
                    Command::new("get")
                        .about("Get a key's value in the KV store")
                        .arg(arg!(<key> "Key to get the value of"))
                )
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
            dbg!(&message);
            let mut stream = TcpStream::connect("127.0.0.1:7878")
                .expect("Failed to connect");

            let msg = format!("Rustis 1.0.0\npublish\n{}\n{}\n\n", channel, message);
            dbg!(&msg);

            stream.write_all(msg.as_bytes()).unwrap();

            stream.flush().unwrap();

            echo_stream(&mut stream, true).expect("Failed to echo stream");
        } else if let Some(matches) = matches.subcommand_matches("subscribe") {
            let channel = matches.get_one::<String>("channel").unwrap();
            let mut stream = TcpStream::connect("127.0.0.1:7878")
                .expect("Failed to connect");

            let msg = format!("Rustis 1.0.0\nsubscribe\n{}\n\n", channel.trim());

            dbg!(&msg);

            stream.write_all(msg.as_bytes()).unwrap();

            stream.flush().unwrap();

            echo_stream(&mut stream, false).expect("Failed to echo stream");
        } else if let Some(matches) = matches.subcommand_matches("set") {
            let key = matches.get_one::<String>("key").unwrap();
            let value = matches.get_one::<String>("value").unwrap();

            let mut stream = TcpStream::connect("127.0.0.1:7878")
                .expect("Failed to connect");
    
            let msg = format!("Rustis 1.0.0\nset\n{}\n{}\n\n", key.trim(), value.trim());

            stream.write_all(msg.as_bytes()).unwrap();
            stream.flush().unwrap();

            echo_stream(&mut stream, true).expect("Failed to echo stream");
        } else if let Some(matches) = matches.subcommand_matches("setex") {
            let key = matches.get_one::<String>("key").unwrap();
            let value = matches.get_one::<String>("value").unwrap();
            let ttl = matches.get_one::<u64>("ttl").unwrap();

            let mut stream = TcpStream::connect("127.0.0.1:7878")
                .expect("Failed to connect");
    
            let msg = format!("Rustis 1.0.0\nsetex\n{}\n{}\n{}\n\n", key.trim(), ttl, value.trim());

            stream.write_all(msg.as_bytes()).unwrap();
            stream.flush().unwrap();

            echo_stream(&mut stream, true).expect("Failed to echo stream");
        } else if let Some(matches) = matches.subcommand_matches("get") {
            let key = matches.get_one::<String>("key").unwrap();

            let mut stream = TcpStream::connect("127.0.0.1:7878")
                .expect("Failed to connect");
    
            let msg = format!("Rustis 1.0.0\nget\n{}\n\n", key.trim());

            stream.write_all(msg.as_bytes()).unwrap();
            stream.flush().unwrap();

            echo_stream(&mut stream, true).expect("Failed to echo stream");
        }
    }
}

fn echo_stream(stream: &mut TcpStream, once: bool) -> io::Result<()> {
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
            let packet = &data[..=index-1];
            println!("{}", String::from_utf8_lossy(&packet));

            data.drain(..=index+1);
        }
        if once {
            break;
        }
    }

    Ok(())
}

