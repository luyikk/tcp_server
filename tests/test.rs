#![feature(async_closure)]

use std::error::Error;
use std::sync::Arc;
use tcpserver::{Builder, IPeer, ITCPServer};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_builder() -> Result<(), Box<dyn Error>> {
    let tcpserver = Builder::new("0.0.0.0:8998")
        .set_connect_event(|addr| {
            println!("{:?} connect", addr);
            true
        })
        .set_input_event(async move |mut reader, peer, _| {
            let mut buff = [0; 4096];
            while let Ok(len) = reader.read(&mut buff).await {
                if len == 0 {
                    break;
                }
                println!("{:?}", &buff[..len]);
                peer.send(buff[..len].to_vec()).await.unwrap();
            }
            println!("{:?} disconnect", peer.addr());
        })
        .build()
        .await;

    tcpserver.start(()).await?;
    Ok(())
}

#[tokio::test]
async fn echo_server() -> Result<(), Box<dyn Error>> {
    struct Foo {
        serv: Arc<dyn ITCPServer<()>>,
    }

    unsafe impl Send for Foo {}
    unsafe impl Sync for Foo {}

    impl Foo {
        pub async fn start(&self) -> Result<(), Box<dyn Error>> {
            self.serv.start_block(()).await
        }
    }
    let tcpserver: Arc<dyn ITCPServer<()>> = Builder::new("0.0.0.0:5555")
        .set_connect_event(|addr| {
            println!("{:?} connect", addr);
            true
        })
        .set_input_event(async move |mut reader, peer, _| {
            let mut buff = [0; 4096];
            while let Ok(len) = reader.read(&mut buff).await {
                if len == 0 {
                    break;
                }
                println!("{:?}", &buff[..len]);
                peer.send(buff[..len].to_vec()).await.unwrap();
            }
            println!("{:?} disconnect", peer.addr());
        })
        .build()
        .await;

    let foo_server = Arc::new(Foo { serv: tcpserver });

    foo_server.start().await?;
    Ok(())
}

#[tokio::test]
async fn echo_client() -> Result<(), Box<dyn Error>> {
    let mut tcp_stream = tokio::net::TcpStream::connect("127.0.0.1:5555").await?;
    let data = b"12231222222221";
    let mut read = [0; 14];
    for _ in 0..100 {
        tcp_stream.write(data).await?;
        let len = tcp_stream.read(&mut read).await?;
        if len != 0 {
            assert_eq!(*data, read);
        } else {
            println!("disconnect");
            break;
        }
    }

    Ok(())
}
