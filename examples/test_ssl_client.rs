use anyhow::*;
use tokio_openssl::SslStream;
use openssl::ssl::{SslConnector, SslMethod};
use tokio::net::TcpStream;
use std::pin::Pin;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[cfg(tls)]
#[tokio::main]
async fn main()->Result<()> {
    let mut connector = SslConnector::builder(SslMethod::tls()).unwrap();
    connector.set_ca_file("tests/server_crt.pem").unwrap();
    let ssl = connector
        .build()
        .configure()?
        .into_ssl("localhost")?;

    let stream = TcpStream::connect("127.0.0.1:5555").await.unwrap();
    let mut stream = SslStream::new(ssl, stream).unwrap();
    Pin::new(&mut stream).connect().await.unwrap();
    stream.write_all(b"asdf").await.unwrap();
    let mut buf = [0;4];
    stream.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"asdf");
    println!("finish");
    Ok(())
}