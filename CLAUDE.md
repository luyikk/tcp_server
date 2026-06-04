# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build (without TLS)
cargo build

# Build with TLS support
cargo build --features tls

# Run all tests
cargo test

# Run a single test
cargo test -- test_builder

# Run the echo server example
cargo run --example echo_server

# Run the TLS server example
cargo run --example ssl_server --features tls
```

MSRV is Rust 1.75.

## Architecture

This is an async TCP server framework (crate name `tcpserver`) built on Tokio, using the [aqueue](https://crates.io/crates/aqueue) actor crate for interior mutability instead of `Arc<Mutex<T>>`.

### Core Actors

Both `TCPServer` and `TCPPeer` are wrapped in `aqueue::Actor`. Rather than using locks, callers invoke methods on `Actor<T>` which queues closures to execute against `&mut T` sequentially. This means `Actor<TCPServer<...>>` and `Arc<Actor<TCPPeer<C>>>` are the primary handle types throughout the API.

### Key Types

- **`TCPServer<I, R, T, B, C, IST>`** (`src/tcpserver.rs`) — The server. Generic over the input handler `I`, its future return type `R`, a user token `T` (cloned per connection), the stream-init future `B`, the stream type `C` (e.g., `TcpStream` or `SslStream`), and the stream-init function `IST`. `start()` takes the listener, spawns an accept loop, and for each accepted connection spawns a per-connection task that calls `stream_init`, splits the stream, creates a `TCPPeer`, and invokes the `input_event` closure.

- **`TCPPeer<C>`** (`src/peer.rs`) — Represents a connected client. Owns the write half (`WriteHalf<C>`) and the remote `SocketAddr`. Provides `send`, `send_all`, `flush`, and `disconnect` (shutdown). Wrapped in `Actor` and exposed via `Arc<Actor<TCPPeer<C>>>`.

- **`IPeer` trait** (`src/peer.rs`) — Trait implemented for `Actor<TCPPeer<C>>`. All methods are async and proxy through `inner_call` to the actor. This is the interface users program against for sending data.

- **`ITCPServer<T>` trait** (`src/tcpserver.rs`) — Trait implemented for `Actor<TCPServer<...>>`. Has `start(token)` (returns `JoinHandle`) and `start_block(token)` (awaits the handle). Exists to enable `Arc<dyn ITCPServer<T>>` for type-erased usage (see examples).

### Builder Pattern

Two builders, both using a required-fields-by-panic pattern:

- **`Builder`** (`src/builder.rs`) — Takes an address (`ToSocketAddrs`), binds internally. This is the common case.
- **`FromStdBuilder`** (`src/builder_std.rs`) — Takes an existing `std::net::TcpListener`. Use this when you need to set socket options (e.g., `SO_REUSEADDR` via `socket2`) before the server takes ownership.

Both require `.set_input_event()` and `.set_stream_init()` before `.build()`. Missing either panics at runtime.

### Stream Init Extension Point

The `set_stream_init` closure transforms a raw `TcpStream` into any type that implements `AsyncRead + AsyncWrite`. For plain TCP, the closure is `|tcp_stream| async move { Ok(tcp_stream) }`. For TLS, it wraps the stream with `tokio_openssl::SslStream` (see `examples/ssl_server.rs`). The `tls` feature flag pulls in `openssl`, `openssl-sys`, and `tokio-openssl`.

### Input Event Signature

The input closure signature is:
```
Fn(ReadHalf<C>, Arc<Actor<TCPPeer<C>>>, T) -> Future<Output = anyhow::Result<()>>
```
It receives the read half (split from the stream), a handle to the peer for sending responses, and a clone of the user token.

### Error Type

`src/error.rs` — A `thiserror`-derived enum with three variants: `IOError` (from `std::io::Error`), `JoinError` (from `tokio::task::JoinError`), and `NotListenerError` (returned if `start()` is called twice or the listener was already consumed). The crate re-exports a `Result<T>` alias using this error type.

## README.md
Add bilingual instructions in English and Chinese, referring to: https://github.com/luyikk/rust_netx/blob/master/README.md