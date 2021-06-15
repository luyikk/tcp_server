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

lazy_static!{
    pub static ref SSL:SslAcceptor={
        let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        acceptor
            .set_private_key_file("tests/server_key.pem", SslFiletype::PEM)
            .unwrap();
        acceptor
            .set_certificate_chain_file("tests/server_crt.pem")
            .unwrap();
        acceptor.build()
    };
}


#[tokio::main]
async fn main()->Result<()> {
    env_logger::Builder::new().filter_level(LevelFilter::Debug).init();
    let tcpserver: Arc<dyn ITCPServer<()>> = Builder::new("0.0.0.0:5555")
        .set_connect_event(|addr| {
            println!("{:?} connect", addr);
            true
        })
        .set_stream_init(async move |tcp_stream|{
            let ssl = Ssl::new(SSL.context()).unwrap();
            let mut stream = SslStream::new(ssl, tcp_stream)?;
            Pin::new(&mut stream).accept().await?;
            Ok(stream)
        })
        .set_input_event(async move |mut reader, peer, _| {
            let mut buff = [0; 4096];
            while let Ok(len) = reader.read(&mut buff).await {
                if len == 0 {
                    break;
                }
                peer.send(&buff[..len]).await?;
            }
            println!("{:?} disconnect", peer.addr());
            Ok(())
        })
        .build()
        .await;

    tcpserver.start_block(()).await?;
    Ok(())
}
