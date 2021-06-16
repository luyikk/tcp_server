use anyhow::*;
use openssl::ssl::{SslConnector, SslMethod};
use std::pin::Pin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_openssl::SslStream;

#[tokio::main]
async fn main() -> Result<()> {
    let mut connector = SslConnector::builder(SslMethod::tls())?;
    connector.set_ca_file("tests/cert.pem")?;
    let ssl = connector.build().configure()?.into_ssl("localhost")?;

    let stream = TcpStream::connect("127.0.0.1:1002").await.unwrap();
    let mut stream = SslStream::new(ssl, stream).unwrap();
    Pin::new(&mut stream).connect().await.unwrap();
    stream.write_all(b"asdf\r\n").await.unwrap();
    let mut buf = [0; 5];
    stream.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"200\r\n");
    println!("finish");
    Ok(())
}
