use rustis::pubsub::PubSub;
use rustis::threadpool::ThreadPool;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

struct ServerState {
    ps: PubSub,
}

pub fn start_server(threads: usize) {
    println!("Starting server with {threads} threads!");

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let ps = PubSub::new();
    let tcp_pool = ThreadPool::new(threads);

    let state = ServerState { ps };
    let state = Arc::new(Mutex::new(state));

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

fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<ServerState>>) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut buf = String::new();

    let _ = buf_reader.read_line(&mut buf);

    let components: Vec<&str> = buf.split('|').map(|c| c.trim()).collect();

    let command = match *components.first().unwrap() {
        "subscribe" => RustisCommand::Subscribe,
        "publish" => RustisCommand::Publish,
        _ => RustisCommand::Unknown,
    };

    match command {
        RustisCommand::Subscribe => handle_subscribe(stream, state, components),
        RustisCommand::Publish => handle_publish(stream, state, components),
        RustisCommand::Unknown => {
            let _ = write_message(&mut stream, "unknown");
        }
    };
}

fn handle_subscribe(mut stream: TcpStream, state: Arc<Mutex<ServerState>>, components: Vec<&str>) {
    let channel = components.get(1);
    if channel.is_some() {
        let mut state = state.lock().unwrap();
        let receiver = state.ps.subscribe(channel.unwrap().to_string());
        drop(state);

        match receiver {
            Ok(receiver) => {
                loop {
                    let message = receiver.recv().unwrap();

                    let _ = write_message(&mut stream, &message[..]);
                }
            }
            Err(_) => (),
        }
    } else {

    }
}

fn handle_publish(mut stream: TcpStream, state: Arc<Mutex<ServerState>>, components: Vec<&str>) {
    let channel = components.get(1);
    let message = components.get(2);

    if channel.is_some() && message.is_some() {
        let mut state = state.lock().unwrap();
        state
            .ps
            .publish(channel.unwrap().to_string(), message.unwrap().to_string());
        let _ = write_message(&mut stream, "published");
    } else {
        let _ = write_message(&mut stream, "invalid");
    }
}

fn write_message(stream: &mut TcpStream, message: &str) {
    let result = stream.write_all(message.as_bytes());

    match result {
        Ok(_) => {
            print!("messaged");
        },
        Err(e) => {
            println!("error: {}", e.to_string());
        }
    }
}

