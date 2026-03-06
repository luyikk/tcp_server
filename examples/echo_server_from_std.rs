use anyhow::Result;
use socket2::{Domain, Type};
use std::net::SocketAddr;
use std::sync::Arc;
use tcpserver::{FromStdBuilder, IPeer, ITCPServer};
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = socket2::Socket::new(Domain::IPV4, Type::STREAM, None)?;
    listener.set_reuse_address(true)?;
    let address: SocketAddr = "0.0.0.0:5555".parse().unwrap();
    listener.bind(&address.into())?;
    listener.listen(128)?;

    let tcpserver: Arc<dyn ITCPServer<()>> = FromStdBuilder::new(listener.into())
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
