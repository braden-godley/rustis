mod server;
mod client;

use clap::{arg, command, Command};
use std::io;

fn main() -> io::Result<()> {
    let matches = command!()
        .subcommand(
            Command::new("server")
                .about("Runs the server")
                .arg(arg!(-t --threads <N> "number of tcp threads")
                     .default_value("10")
                     .value_parser(clap::value_parser!(u16)))
                .arg(arg!(--host <host> "The host to connect to")
                     .default_value("127.0.0.1"))
                .arg(arg!(-p --port <port> "The port to connect to")
                     .default_value("7878")
                     .value_parser(clap::value_parser!(u16)))
        )
        .subcommand(
            Command::new("client")
                .about("Runs a client that will connect to the server")
                .arg(arg!(--host <host> "The host to connect to")
                     .default_value("127.0.0.1"))
                .arg(arg!(-p --port <port> "The port to connect to")
                     .default_value("7878")
                     .value_parser(clap::value_parser!(u16)))
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
                .subcommand(
                    Command::new("ttl")
                        .about("Get a key's remaining expiration time in the KV store")
                        .arg(arg!(<key> "Key to get the ttl of"))
                )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("server") {
        let threads = matches
            .get_one::<u16>("threads").unwrap();
        let host = matches.get_one::<String>("host").unwrap();
        let port = matches.get_one::<u16>("port").unwrap();

        server::start_server(host, *port, *threads)
            .expect("Failed to start Rustis!");
    } else if let Some(matches) = matches.subcommand_matches("client") {
        let host = matches.get_one::<String>("host").unwrap();
        let port = matches.get_one::<u16>("port").unwrap();

        let mut client = client::Client::new(host, *port, "1.0.0");

        if let Some(matches) = matches.subcommand_matches("publish") {
            let channel = matches.get_one::<String>("channel").unwrap();
            let message = matches.get_one::<String>("message").unwrap();

            client.send_publish(channel, message)?;
        } else if let Some(matches) = matches.subcommand_matches("subscribe") {
            let channel = matches.get_one::<String>("channel").unwrap();

            client.send_subscribe(channel)?;
        } else if let Some(matches) = matches.subcommand_matches("set") {
            let key = matches.get_one::<String>("key").unwrap();
            let value = matches.get_one::<String>("value").unwrap();

            client.send_set(key, value)?;
        } else if let Some(matches) = matches.subcommand_matches("setex") {
            let key = matches.get_one::<String>("key").unwrap();
            let value = matches.get_one::<String>("value").unwrap();
            let ttl = matches.get_one::<u64>("ttl").unwrap();

            client.send_setex(key, value, *ttl)?;
        } else if let Some(matches) = matches.subcommand_matches("get") {
            let key = matches.get_one::<String>("key").unwrap();

            client.send_get(key)?;
        } else if let Some(matches) = matches.subcommand_matches("ttl") {
            let key = matches.get_one::<String>("key").unwrap();

            client.send_ttl(key)?;
        }
    };
    Ok(())
}

