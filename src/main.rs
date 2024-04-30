mod server;

use clap::{arg, command, value_parser, ArgAction, Command};

fn main() {
    let matches = command!()
        .subcommand(
            Command::new("server")
                .about("Runs the server")
                .arg(arg!(-t --threads <N> "number of tcp threads").default_value("1000"))
                
        )
        .subcommand(
            Command::new("client")
                .about("Runs a client that will connect to the server")
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("server") {
        let threads: usize = matches
            .get_one::<String>("threads").expect("required")
            .parse::<usize>().expect("Invalid thread number");

        server::start_server(threads);
    } else if let Some(matches) = matches.subcommand_matches("client") {
        println!("You just ran the client!");
    }
}
