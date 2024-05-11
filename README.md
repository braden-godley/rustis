# Rustis

[![CI](https://github.com/braden-godley/rustis/actions/workflows/test.yml/badge.svg)](https://github.com/braden-godley/rustis/actions/workflows/test.yml)

Rustis is a Rust-based implementation of a subset of Redis's functionality. It provides an executable that can be used to start a server or issue commands to a server as a client.

## Installation

You can install Rustis by downloading this repository, then running the following:

```sh
cargo install --path .
```

This will install the `rustis` binary on your PATH

## Features

Right now there are two core services implemented in Rustis's client and server:
- Publisher / Subscriber
- Key / value storage

## How it works

The Rustis server listens for TCP connections and parses/validates the packets they send into commands. It uses a pool of threads to handle TCP connections and another pool of threads to handle the publisher/subscriber logic.

## How to use

### Starting a server

To start a server that clients can send commands to, run the following:

```sh
rustis server
```

Rustis binds to `127.0.0.1:7878` by default, but you can also specify a host and/or port:

```sh
rustis server --host 0.0.0.0 --port 3000
```

### Running client commands

All `client` commands assume the Rustis server is listening on `127.0.0.1:7878` but you can also specify a host and/or port:

```sh
rustis client --host 0.0.0.0 --port 3000 <client command here>
```

### Publishing a message to a channel

```sh
rustis client publish 'channel' 'message'
```

### Subscribing to messages on a channel

```sh
rustis client subscribe 'channel'
```

### Setting a value

```sh
rustis client set 'key' 'value'
```

### Setting a value with an expiration TTL

```sh 
rustis client setex 'key' <ttl in seconds> 'value'
```

### Getting a value

```sh 
rustis client get 'key'
```

### Getting the remaining time of a key

```sh 
rustis client ttl 'key'
```

