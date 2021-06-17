#![feature(async_closure)]
use anyhow::*;
use openssl::ssl::{SslConnector, SslMethod, SslFiletype};
use std::pin::Pin;
use tokio::io::{AsyncReadExt};
use tokio_openssl::SslStream;
use tcpclient::SocketClientTrait;
use tokio::sync::oneshot::channel;

#[tokio::main]
async fn main() -> Result<()> {
    // TLS TEST CLIENT
    let (tx,rx)=channel();
    let client=
        tcpclient::TcpClient::connect_stream_type("127.0.0.1:5555",
            async move|tcp_stream|{
                let mut connector = SslConnector::builder(SslMethod::tls())?;
                connector.set_ca_file("tests/server-cert.pem")?;
                connector.set_private_key_file("tests/client-key.pem", SslFiletype::PEM)?;
                connector.set_certificate_chain_file("tests/client-cert.pem")?;
                connector.check_private_key()?;
                let ssl = connector.build().configure()?.into_ssl("localhost")?;
                let mut stream = SslStream::new(ssl, tcp_stream)?;
                Pin::new(&mut stream).connect().await?;
                Ok(stream)
            },
            async move|tx,_client,mut stream|{
                let mut buf = [0; 5];
                stream.read_exact(&mut buf).await?;
                assert_eq!(&buf, b"200\r\n");
                tx.send(()).map_err(|_| anyhow!("rx is close"))?;
                Ok(true)
            },tx).await?;

    client.send_all(b"hello world\r\n").await?;
    rx.await?;
    println!("finish");
    Ok(())
}
