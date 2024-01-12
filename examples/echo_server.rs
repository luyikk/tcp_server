use anyhow::Result;
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
        .set_stream_init(|tcp_stream| async move { Ok(tcp_stream) })
        .set_input_event(|mut reader, peer, _| async move {
            let mut buff = [0; 4096];
            while let Ok(len) = reader.read(&mut buff).await {
                if len == 0 {
                    break;
                }
                peer.send(buff[..len].to_vec()).await?;
            }
            println!("{:?} disconnect", peer.addr());
            Ok(())
        })
        .build()
        .await;
    tcpserver.start_block(()).await?;
    Ok(())
}
