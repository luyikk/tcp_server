#![feature(async_closure)]
use openssl::ssl::{SslAcceptor, SslMethod, SslFiletype, Ssl};
use lazy_static::lazy_static;
use std::sync::Arc;
use tcpserver::{Builder, IPeer, ITCPServer};
use tokio::io::AsyncReadExt;
use anyhow::*;
use tokio_openssl::SslStream;
use std::pin::Pin;
use log::LevelFilter;
use std::time::Duration;
use tokio::time::sleep;
lazy_static!{
    pub static ref SSL:SslAcceptor={

        let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
         acceptor
            .set_private_key_file("tests/key.pem", SslFiletype::PEM)
            .unwrap();
         acceptor
            .set_certificate_chain_file("tests/cert.pem")
            .unwrap();
        acceptor.build()
    };
}

#[tokio::main]
async fn main()->Result<()> {
    SSL.context();
    env_logger::Builder::new().filter_level(LevelFilter::Debug).init();
    let tcpserver: Arc<dyn ITCPServer<()>> = Builder::new("0.0.0.0:5555")
        .set_connect_event(|addr| {
            println!("{:?} connect", addr);
            true
        })
        .set_stream_init(async move |tcp_stream|{
            let ssl = Ssl::new(SSL.context())?;
            let mut stream = SslStream::new(ssl, tcp_stream)?;
            sleep(Duration::from_millis(200)).await;
            Pin::new(&mut stream).accept().await?;
            Ok(stream)
        })
        .set_input_event(async move |mut reader, peer, _| {
            let mut buff = [0; 4096];
            while let Ok(len) = reader.read(&mut buff).await {
                if len == 0 {
                    break;
                }
                println!("{}",std::str::from_utf8(&buff[..len])?);
                peer.send(b"200\r\n").await?;
            }
            println!("{:?} disconnect", peer.addr());
            Ok(())
        })
        .build()
        .await;

    tcpserver.start_block(()).await?;
    Ok(())
}
