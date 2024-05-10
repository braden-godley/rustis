use rustis::pubsub::PubSub;
use rustis::threadpool::ThreadPool;
use rustis::packetreader::RequestPacket;
use std::io::{prelude::*, self};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::str;

struct ServerState {
    ps: PubSub,
}

pub fn start_server(threads: usize) -> io::Result<()> {
    println!("Starting server with {threads} threads!");

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let ps = PubSub::new();
    let tcp_pool = ThreadPool::new(threads);

    let state = ServerState { ps };
    let state = Arc::new(Mutex::new(state));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let state = Arc::clone(&state);

        tcp_pool.execute(move || {
            handle_connection(stream, state)
                .expect("Can't handle!");
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<ServerState>>) -> io::Result<()> {
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
            process_packet(&mut stream, &state, packet)?;

            data.drain(..=index+1);
        }
    }

    Ok(())
}

fn process_packet(stream: &mut TcpStream, state: &Arc<Mutex<ServerState>>, packet: &[u8]) -> io::Result<()> {
    let packet_str = String::from_utf8_lossy(packet);
    let packet = RequestPacket::new(packet_str.to_string());

    dbg!(&packet);

    match packet {
        RequestPacket::Subscribe { channel } => handle_subscribe(stream, state, channel),
        RequestPacket::Publish { channel, message } => handle_publish(stream, state, channel, message),
        RequestPacket::Invalid { error } => {
            write_message(stream, &error)
        },
        RequestPacket::Unknown => {
            write_message(stream, "unknown")
        },
    }

    Ok(())
}

fn handle_subscribe(stream: &mut TcpStream, state: &Arc<Mutex<ServerState>>, channel: String) {
    let mut state = state.lock().unwrap();
    let receiver = state.ps.subscribe(channel);
    drop(state);

    match receiver {
        Ok(receiver) => {
            loop {
                let message = receiver.recv().unwrap();

                let _ = write_message(stream, &message[..]);
            }
        }
        Err(_) => (),
    }
}

fn handle_publish(stream: &mut TcpStream, state: &Arc<Mutex<ServerState>>, channel: String, message: String) {
    let mut state = state.lock().unwrap();
    state
        .ps
        .publish(channel, message);
    let _ = write_message(stream, "published");
}

fn write_message(stream: &mut TcpStream, message: &str) {
    let message = format!("{}\n\n", message);
    let result = stream.write_all(message.as_bytes());

    match result {
        Ok(_) => {
            println!("messaged");
        },
        Err(e) => {
            println!("error: {}", e.to_string());
        }
    }
}

