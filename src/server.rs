use rustis::pubsub::PubSub;
use rustis::threadpool::ThreadPool;
use rustis::packetreader::RequestPacket;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::str;

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

fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<ServerState>>) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut buf = Vec::new();

    let _ = buf_reader.read_to_end(&mut buf);

    let buf: String = str::from_utf8(&buf)
        .expect("Invalid utf-8 sequence")
        .to_string();

    let packet = RequestPacket::new(buf);

    match packet {
        RequestPacket::Subscribe { channel } => handle_subscribe(stream, state, channel),
        RequestPacket::Publish { channel, message } => handle_publish(stream, state, channel, message),
        RequestPacket::Invalid { error } => {
            write_message(&mut stream, &error)
        },
        RequestPacket::Unknown => {
            write_message(&mut stream, "unknown")
        },
    }
}

fn handle_subscribe(mut stream: TcpStream, state: Arc<Mutex<ServerState>>, channel: String) {
    let mut state = state.lock().unwrap();
    let receiver = state.ps.subscribe(channel);
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
}

fn handle_publish(mut stream: TcpStream, state: Arc<Mutex<ServerState>>, channel: String, message: String) {
    let mut state = state.lock().unwrap();
    state
        .ps
        .publish(channel, message);
    let _ = write_message(&mut stream, "published");
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

