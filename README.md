#rust tcp server frame.

# Examples Echo
``` rust
#![feature(async_closure)]
use anyhow::*;
use std::sync::Arc;
use tcpserver::{Builder, IPeer, ITCPServer};
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> Result<()> {
    let tcpserver: Arc<dyn ITCPServer<()>> = Builder::new("0.0.0.0:5555")
        .set_connect_event(|addr| {
            println!("{:?} connect", addr);
            true
        })
        .set_stream_init(async move |tcp_stream| Ok(tcp_stream))
        .set_input_event(async move |mut reader, peer, _token| {
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

```
