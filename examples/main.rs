#![feature(async_closure)]

use std::error::Error;
use std::sync::Arc;
use tcpserver::{Builder, IPeer, ITCPServer};
use tokio::io::AsyncReadExt;

#[global_allocator]
static  MIN:mimalloc::MiMalloc=mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
                peer.send(buff[..len].to_vec()).await.unwrap();
            }
            println!("{:?} disconnect", peer.addr());
        })
        .build()
        .await;

    tcpserver.start_block(()).await?;
    Ok(())
}
