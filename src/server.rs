use rustis::pubsub::PubSub;
use rustis::threadpool::ThreadPool;
use rustis::packetreader::RequestPacket;
use rustis::kvstore::KvStore;

use std::io::{prelude::*, self};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::str;

use socket2::{Socket, Domain, Type, TcpKeepalive};

struct ServerState {
    ps: PubSub,
    kv: KvStore,
}

pub fn start_server(threads: usize) -> io::Result<()> {
    println!("Starting server with {threads} threads!");

    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;

    let address: SocketAddr = "127.0.0.1:7878".parse().unwrap();

    socket.bind(&address.into())?;
    socket.listen(128)?;

    let keepalive = TcpKeepalive::new()
        .with_time(Duration::from_secs(30))
        .with_interval(Duration::from_secs(3))
        .with_retries(4);

    socket.set_tcp_keepalive(&keepalive)?;

    let listener: TcpListener = socket.into();
    let ps = PubSub::new();
    let kv = KvStore::new();
    let tcp_pool = ThreadPool::new(threads);

    let state = ServerState { ps, kv };
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
        RequestPacket::Set { key, value } => handle_set(stream, state, key, value),
        RequestPacket::SetEx { key, ttl, value } => handle_setex(stream, state, key, ttl, value),
        RequestPacket::Get { key } => handle_get(stream, state, key),
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

fn handle_set(stream: &mut TcpStream, state: &Arc<Mutex<ServerState>>, key: String, value: String) {
    let mut state = state.lock().unwrap();
    state.kv.set(&key[..], &value[..]);
    let _ = write_message(stream, "set");
}

fn handle_setex(stream: &mut TcpStream, state: &Arc<Mutex<ServerState>>, key: String, ttl: u64, value: String) {
    let mut state = state.lock().unwrap();
    state.kv.setex(&key[..], &value[..], ttl);
    let _ = write_message(stream, "setex");
}

fn handle_get(stream: &mut TcpStream, state: &Arc<Mutex<ServerState>>, key: String) {
    let state = state.lock().unwrap();
    let val = state.kv.get(&key[..]);
    if let Some(val) = val {
        write_message(stream, &format!("1\n{}", &val));
    } else {
        write_message(stream, "0");
    }
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

