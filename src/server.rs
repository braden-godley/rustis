use rustis::threadpool::ThreadPool;
use rustis::pubsub::PubSub;
use std::net::{TcpStream, TcpListener};
use std::io::{prelude::*, BufReader};
use std::sync::Arc;

struct ServerState {
    ps: PubSub,
}

pub fn start_server(threads: usize) {
    println!("Starting server with {threads} threads!");

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let ps = PubSub::new();
    let tcp_pool = ThreadPool::new(threads);

    let state = Arc::new(&mut ServerState { ps });

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let state = Arc::clone(&state);

        tcp_pool.execute(move || {
            handle_connection(stream, state);
        });
    }
}

enum RustisCommand {
    Subscribe,
    Publish,
    Unknown,
}

fn handle_connection(mut stream: TcpStream, state: Arc<&mut ServerState>) {
    let mut buf_reader = BufReader::new(&mut stream);

    let mut buf = String::new();
    while let Ok(_) = buf_reader.read_line(&mut buf) {
        let components: Vec<&str> = buf.split("|").collect();

        use RustisCommand::*;

        let command = match *components.first().unwrap() {
            "subscribe" => Subscribe,
            "publish" => Publish,
            _ => Unknown,
        };

        let response = match command {
            Subscribe => {
                "success"
            },
            Publish => {
                let channel = components.get(1);
                let message = components.get(2);

                if channel.is_some() && message.is_some() {
                    state.ps.publish(channel.unwrap().to_string(), message.unwrap().to_string());
                    "success"
                } else {
                    "invalid"
                }
            },
            Unknown => {
                "unknown command"
            },
        };

        stream.write_all(response.as_bytes()).unwrap();
    }

    // let contents = fs::read_to_string("hello.html").unwrap();
    // let length = contents.len();
    //
    // let response = 
    //     format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    //
    // stream.write_all(response.as_bytes()).unwrap();
}
