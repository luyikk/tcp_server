# tcpserver

[![Crates.io](https://img.shields.io/crates/v/tcpserver.svg)](https://crates.io/crates/tcpserver)
[![Docs.rs](https://docs.rs/tcpserver/badge.svg)](https://docs.rs/tcpserver)
[![License](https://img.shields.io/crates/l/tcpserver.svg)](LICENSE)

**[English](#english) | [中文](#chinese)**

---

## English

> A lightweight, extensible async TCP server framework built on Tokio — with TLS support, backpressure, and a clean builder API.

- **MSRV**: Rust 1.75+
- **Async runtime**: Tokio
- **Concurrency model**: Actor-based (via [aqueue](https://crates.io/crates/aqueue)) — lock-free interior mutability
- **Stream extensibility**: Plain TCP, TLS (OpenSSL), or any `AsyncRead + AsyncWrite` stream
- **Backpressure**: Optional connection limit via `Semaphore`
- **Related crate**: [tcpclient](https://crates.io/crates/tcpclient) — matching TCP client framework

---

### Features

| Feature | Description |
|---------|-------------|
| 🔌 **Builder API** | Chainable `Builder` / `FromStdBuilder` with compile-time type safety |
| 🔁 **Stream Init** | Transform raw `TcpStream` into any `AsyncRead + AsyncWrite` (TLS, GZIP, …) |
| 🔒 **TLS** | Optional OpenSSL-based TLS via `tls` feature flag |
| ⚡ **TCP_NODELAY** | Disable Nagle's algorithm for low-latency scenarios |
| 🚦 **Connection Limit** | `Semaphore`-based backpressure — block accept when at capacity |
| 🎯 **Connect Filter** | Per-connection callback to accept or reject by `SocketAddr` |
| 📦 **Zero-cost generics** | All type parameters resolved at compile time; use `Arc<dyn ITCPServer<T>>` for type erasure |

---

### Project Structure

```
src/
├── lib.rs            # Crate root, re-exports
├── tcpserver.rs      # TCPServer + ITCPServer trait
├── peer.rs           # TCPPeer + IPeer trait
├── builder.rs        # Builder (bind by address)
├── builder_std.rs    # FromStdBuilder (use existing TcpListener)
└── error.rs          # Error types
examples/
├── echo_server.rs           # Plain TCP echo server
├── echo_server_from_std.rs  # Echo server from std TcpListener
├── echo_client.rs           # Echo client (for testing)
├── ssl_server.rs            # TLS echo server (requires --features tls)
└── test_ssl_client.rs       # TLS client (requires --features tls)
```

---

### Quick Start

#### Add Dependencies

```toml
[dependencies]
tcpserver = "2"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

#### Optional Features

```toml
# TLS support
tcpserver = { version = "2", features = ["tls"] }
```

---

#### Echo Server

The input closure receives `(ReadHalf<C>, Arc<Actor<TCPPeer<C>>>, T)` where `T` is your per-connection token.

```rust
use anyhow::Result;
use std::sync::Arc;
use tcpserver::{Builder, IPeer, ITCPServer};
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> Result<()> {
    let server: Arc<dyn ITCPServer<()>> = Builder::new("0.0.0.0:5555")
        .set_connect_event(|addr| {
            println!("{:?} connect", addr);
            true // return false to reject
        })
        .set_stream_init(|tcp_stream| async move { Ok(tcp_stream) })
        .set_input_event(|mut reader, peer, _token| async move {
            let mut buf = [0u8; 4096];
            while let Ok(len) = reader.read(&mut buf).await {
                if len == 0 {
                    break;
                }
                // echo back
                peer.send_all(&buf[..len]).await?;
            }
            println!("{:?} disconnect", peer.addr());
            Ok(())
        })
        .build()
        .await;

    server.start_block(()).await?;
    Ok(())
}
```

Run it:

```bash
cargo run --example echo_server
```

---

#### With TCP_NODELAY + Connection Limit

```rust
let server = Builder::new("0.0.0.0:5555")
    .set_nodelay(true)           // disable Nagle's algorithm
    .set_max_connections(1000)   // max concurrent connections (0 = unlimited)
    .set_stream_init(|tcp_stream| async move { Ok(tcp_stream) })
    .set_input_event(|mut reader, peer, _token| async move { /* ... */ Ok(()) })
    .build()
    .await;
```

---

#### Using an Existing `TcpListener` (socket options)

When you need `SO_REUSEADDR` or other socket options before binding:

```rust
use socket2::{Domain, Type, Socket};
use std::net::SocketAddr;
use tcpserver::FromStdBuilder;

let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
socket.set_reuse_address(true)?;
let addr: SocketAddr = "0.0.0.0:5555".parse()?;
socket.bind(&addr.into())?;
socket.listen(128)?;

let server = FromStdBuilder::new(socket.into())
    .set_stream_init(|tcp_stream| async move { Ok(tcp_stream) })
    .set_input_event(|mut reader, peer, _| async move { /* ... */ Ok(()) })
    .build()
    .await;
```

Run:

```bash
cargo run --example echo_server_from_std
```

---

#### TLS Server

Requires `--features tls` and OpenSSL development libraries.

```rust
// See examples/ssl_server.rs for the full example
let acceptor = build_ssl_acceptor(); // SslAcceptor with cert + key

let server = Builder::new("0.0.0.0:5555")
    .set_stream_init(|tcp_stream| async move {
        let ssl = Ssl::new(acceptor.context())?;
        let mut stream = SslStream::new(ssl, tcp_stream)?;
        Pin::new(&mut stream).accept().await?;
        Ok(stream)
    })
    .set_input_event(|mut reader, peer, _| async move { /* ... */ Ok(()) })
    .build()
    .await;
```

Run:

```bash
cargo run --example ssl_server --features tls
```

The matching TLS client:

```bash
cargo run --example test_ssl_client --features tls
```

---

### API Reference

#### `Builder` / `FromStdBuilder`

| Method | Description |
|--------|-------------|
| `new(addr)` / `new(listener)` | Create a builder |
| `set_input_event(f)` | **Required.** Called for each connection — receives reader, peer handle, and token |
| `set_stream_init(f)` | **Required.** Transform `TcpStream` → target stream type |
| `set_connect_event(f)` | Optional. `fn(SocketAddr) -> bool` — return `false` to reject |
| `set_nodelay(bool)` | Enable `TCP_NODELAY`. Default: `false` |
| `set_max_connections(n)` | Max concurrent connections. `0` = unlimited. Default: `0` |
| `build()` | Finalize and create the server. Panics if required fields are missing |

#### `IPeer` Trait

The peer handle passed to your input handler. All methods are async and thread-safe (backed by `aqueue::Actor`).

| Method | Description |
|--------|-------------|
| `addr() -> SocketAddr` | Remote address |
| `is_disconnect() -> Result<bool>` | Check if the peer is disconnected |
| `send(buf) -> Result<usize>` | Send bytes, returns count written |
| `send_all(buf) -> Result<()>` | Send all bytes, then flush |
| `send_ref(&[u8]) -> Result<usize>` | Send from a byte slice reference |
| `send_all_ref(&[u8]) -> Result<()>` | Send-all from a byte slice reference |
| `flush() -> Result<()>` | Flush the write buffer |
| `disconnect() -> Result<()>` | Shutdown the write half |

#### `ITCPServer<T>` Trait

Type-erased server handle. `T` is your per-connection token type.

| Method | Description |
|--------|-------------|
| `start(token) -> Result<JoinHandle<...>>` | Start accepting, returns a handle |
| `start_block(token) -> Result<()>` | Start accepting and block until the server exits |

---

### Examples

```bash
# Plain TCP echo
cargo run --example echo_server

# From existing TcpListener (with socket options)
cargo run --example echo_server_from_std

# TLS server + client (requires OpenSSL)
cargo run --example ssl_server --features tls
cargo run --example test_ssl_client --features tls
```

---

### Related Projects

- [tcpclient](https://crates.io/crates/tcpclient) — Matching async TCP client framework

---

### License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.

---
---

<a name="chinese"></a>
## 中文

> 基于 Tokio 的轻量级异步 TCP 服务框架 — 支持 TLS、背压控制、链式 Builder API。

- **MSRV**: Rust 1.75+
- **异步运行时**: Tokio
- **并发模型**: Actor 模式（基于 [aqueue](https://crates.io/crates/aqueue)）— 无锁内部可变性
- **流扩展**: 纯 TCP、TLS（OpenSSL）、或任意 `AsyncRead + AsyncWrite` 流
- **背压**: 可选连接数限制（`Semaphore`）
- **相关库**: [tcpclient](https://crates.io/crates/tcpclient) — 配套 TCP 客户端框架

---

### 功能特性

| 特性 | 说明 |
|------|------|
| 🔌 **Builder API** | 链式 `Builder` / `FromStdBuilder`，编译期类型安全 |
| 🔁 **流初始化** | 将原始 `TcpStream` 转换为任意 `AsyncRead + AsyncWrite`（TLS、GZIP等） |
| 🔒 **TLS** | 可选 OpenSSL TLS，通过 `tls` feature 开启 |
| ⚡ **TCP_NODELAY** | 关闭 Nagle 算法，降低小包延迟 |
| 🚦 **连接数限制** | 基于 `Semaphore` 的背压 — 达到上限时阻塞 accept |
| 🎯 **连接过滤器** | 按 `SocketAddr` 决定接受或拒绝连接 |
| 📦 **零成本泛型** | 编译期解析所有类型参数；用 `Arc<dyn ITCPServer<T>>` 擦除类型 |

---

### 项目结构

```
src/
├── lib.rs            # crate 根，重导出
├── tcpserver.rs      # TCPServer + ITCPServer trait
├── peer.rs           # TCPPeer + IPeer trait
├── builder.rs        # Builder（按地址绑定）
├── builder_std.rs    # FromStdBuilder（使用已有 TcpListener）
└── error.rs          # 错误类型
examples/
├── echo_server.rs           # 纯 TCP echo 服务器
├── echo_server_from_std.rs  # 从 std TcpListener 构建
├── echo_client.rs           # Echo 客户端（测试用）
├── ssl_server.rs            # TLS echo 服务器（需 --features tls）
└── test_ssl_client.rs       # TLS 客户端（需 --features tls）
```

---

### 快速开始

#### 添加依赖

```toml
[dependencies]
tcpserver = "2"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

#### 可选功能

```toml
# TLS 支持
tcpserver = { version = "2", features = ["tls"] }
```

---

#### Echo 服务器

输入闭包签名为 `(ReadHalf<C>, Arc<Actor<TCPPeer<C>>>, T)`，其中 `T` 是自定义的每连接令牌。

```rust
use anyhow::Result;
use std::sync::Arc;
use tcpserver::{Builder, IPeer, ITCPServer};
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> Result<()> {
    let server: Arc<dyn ITCPServer<()>> = Builder::new("0.0.0.0:5555")
        .set_connect_event(|addr| {
            println!("{:?} 已连接", addr);
            true // 返回 false 则拒绝连接
        })
        .set_stream_init(|tcp_stream| async move { Ok(tcp_stream) })
        .set_input_event(|mut reader, peer, _token| async move {
            let mut buf = [0u8; 4096];
            while let Ok(len) = reader.read(&mut buf).await {
                if len == 0 {
                    break;
                }
                // 原样返回
                peer.send_all(&buf[..len]).await?;
            }
            println!("{:?} 已断开", peer.addr());
            Ok(())
        })
        .build()
        .await;

    server.start_block(()).await?;
    Ok(())
}
```

运行：

```bash
cargo run --example echo_server
```

---

#### 配置 TCP_NODELAY + 连接数限制

```rust
let server = Builder::new("0.0.0.0:5555")
    .set_nodelay(true)           // 关闭 Nagle 算法
    .set_max_connections(1000)   // 最大并发连接数（0 = 不限制）
    .set_stream_init(|tcp_stream| async move { Ok(tcp_stream) })
    .set_input_event(|mut reader, peer, _token| async move { /* ... */ Ok(()) })
    .build()
    .await;
```

---

#### 使用已有 `TcpListener`（设置 socket 选项）

当需要在 bind 之前设置 `SO_REUSEADDR` 等 socket 选项时：

```rust
use socket2::{Domain, Type, Socket};
use std::net::SocketAddr;
use tcpserver::FromStdBuilder;

let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
socket.set_reuse_address(true)?;
let addr: SocketAddr = "0.0.0.0:5555".parse()?;
socket.bind(&addr.into())?;
socket.listen(128)?;

let server = FromStdBuilder::new(socket.into())
    .set_stream_init(|tcp_stream| async move { Ok(tcp_stream) })
    .set_input_event(|mut reader, peer, _| async move { /* ... */ Ok(()) })
    .build()
    .await;
```

运行：

```bash
cargo run --example echo_server_from_std
```

---

#### TLS 服务器

需要 `--features tls` 和 OpenSSL 开发库。

```rust
// 完整示例见 examples/ssl_server.rs
let acceptor = build_ssl_acceptor(); // 配置了证书和密钥的 SslAcceptor

let server = Builder::new("0.0.0.0:5555")
    .set_stream_init(|tcp_stream| async move {
        let ssl = Ssl::new(acceptor.context())?;
        let mut stream = SslStream::new(ssl, tcp_stream)?;
        Pin::new(&mut stream).accept().await?;
        Ok(stream)
    })
    .set_input_event(|mut reader, peer, _| async move { /* ... */ Ok(()) })
    .build()
    .await;
```

运行：

```bash
cargo run --example ssl_server --features tls
```

配套 TLS 客户端：

```bash
cargo run --example test_ssl_client --features tls
```

---

### API 参考

#### `Builder` / `FromStdBuilder`

| 方法 | 说明 |
|------|------|
| `new(addr)` / `new(listener)` | 创建 Builder |
| `set_input_event(f)` | **必填。** 每连接调用 — 接收 reader、peer 句柄和 token |
| `set_stream_init(f)` | **必填。** 将 `TcpStream` 转换为目标流类型 |
| `set_connect_event(f)` | 可选。`fn(SocketAddr) -> bool` — 返回 `false` 拒绝连接 |
| `set_nodelay(bool)` | 启用 `TCP_NODELAY`。默认：`false` |
| `set_max_connections(n)` | 最大并发连接数。`0` = 不限制。默认：`0` |
| `build()` | 构建服务器。缺少必填字段会 panic |

#### `IPeer` Trait

传给输入处理器的 peer 句柄。所有方法均为异步且线程安全（基于 `aqueue::Actor`）。

| 方法 | 说明 |
|------|------|
| `addr() -> SocketAddr` | 远端地址 |
| `is_disconnect() -> Result<bool>` | 检查是否已断开 |
| `send(buf) -> Result<usize>` | 发送数据，返回已写入字节数 |
| `send_all(buf) -> Result<()>` | 发送全部数据并 flush |
| `send_ref(&[u8]) -> Result<usize>` | 从字节切片引用发送 |
| `send_all_ref(&[u8]) -> Result<()>` | 从字节切片引用发送全部 |
| `flush() -> Result<()>` | 刷新写缓冲区 |
| `disconnect() -> Result<()>` | 关闭写半部 |

#### `ITCPServer<T>` Trait

类型擦除的服务器句柄。`T` 是自定义的每连接令牌类型。

| 方法 | 说明 |
|------|------|
| `start(token) -> Result<JoinHandle<...>>` | 启动 accept，返回句柄 |
| `start_block(token) -> Result<()>` | 启动 accept 并阻塞直到服务器退出 |

---

### 示例

```bash
# 纯 TCP echo
cargo run --example echo_server

# 从已有 TcpListener（带 socket 选项）
cargo run --example echo_server_from_std

# TLS 服务器 + 客户端（需要 OpenSSL）
cargo run --example ssl_server --features tls
cargo run --example test_ssl_client --features tls
```

---

### 相关项目

- [tcpclient](https://crates.io/crates/tcpclient) — 配套异步 TCP 客户端框架

---

### 许可证

采用 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 双许可证，任选其一。
