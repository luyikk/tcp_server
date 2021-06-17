#![feature(async_closure)]
use anyhow::*;
use lazy_static::lazy_static;
use log::LevelFilter;
use openssl::ssl::{Ssl, SslAcceptor, SslFiletype, SslMethod, SslVerifyMode};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tcpserver::{Builder, IPeer, ITCPServer};
use tokio::io::AsyncReadExt;
use tokio::time::sleep;
use tokio_openssl::SslStream;
lazy_static! {
    pub static ref SSL: SslAcceptor = {
        let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls_server()).unwrap();
        acceptor.set_ca_file("tests/ca-cert.pem").unwrap();
        acceptor
            .set_private_key_file("tests/server-key.pem", SslFiletype::PEM)
            .unwrap();
        acceptor
            .set_certificate_chain_file("tests/server-cert.pem")
            .unwrap();
        acceptor.set_verify_callback(SslVerifyMode::PEER|SslVerifyMode::FAIL_IF_NO_PEER_CERT,|ok,cert|{
            println!("pre verify ok {}",ok);
            if !ok{
                match cert.verify_cert(){
                    Ok(v)=>{
                        println!("verify {}",v)
                    },
                    Err(_)=>{
                        if let Some(cert)= cert.current_cert(){
                           println!("subject info {:?}",cert.subject_name());
                           println!("issuer info {:?}",cert.issuer_name());
                        }
                    }
                }
            }
            true
        });
        acceptor.check_private_key().unwrap();
        acceptor.build()
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();
    let tcpserver: Arc<dyn ITCPServer<()>> = Builder::new("0.0.0.0:5555")
        .set_connect_event(|addr| {
            println!("{:?} connect", addr);
            true
        })
        .set_stream_init(async move |tcp_stream| {
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
                println!("{}", std::str::from_utf8(&buff[..len])?);
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
