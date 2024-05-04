mod server;

use clap::{arg, command, Command};
use std::net::TcpStream;
use std::io::{prelude::*, Write, Read, BufReader};

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

        server::start_server(threads);
    } else if let Some(matches) = matches.subcommand_matches("client") {
        if let Some(matches) = matches.subcommand_matches("publish") {
            let channel = matches.get_one::<String>("channel").unwrap();
            let message = matches.get_one::<String>("message").unwrap();
            let stream = TcpStream::connect("127.0.0.1:7878");
            if let Ok(mut stream) = stream {
                let msg = format!("publish|{}|{}\n", channel, message);

                stream.write_all(msg.as_bytes()).unwrap();

                stream.flush().unwrap();

                let mut buf = String::new();
                stream.read_to_string(&mut buf).unwrap();

                println!("{}", buf.to_string());
            } else {
                eprintln!("Failed to connect!");
                std::process::exit(1);
            }
        } else if let Some(matches) = matches.subcommand_matches("subscribe") {
            let channel = matches.get_one::<String>("channel").unwrap();
            let stream = TcpStream::connect("127.0.0.1:7878");
            if let Ok(mut stream) = stream {
                let mut buf_reader = BufReader::new(stream.try_clone().unwrap());
                let msg = format!("subscribe|{}\n", channel);

                stream.write_all(msg.as_bytes()).unwrap();

                stream.flush().unwrap();

                println!("subscribed!");

                let mut buf = String::new();
                loop {
                    let _ = buf_reader.read_line(&mut buf);

                    println!("{}", buf.to_string());
                }
            } else {
                eprintln!("Failed to connect!");
                std::process::exit(1);
            }
        }
    }
}

