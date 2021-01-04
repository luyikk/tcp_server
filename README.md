#rust tcp server frame.

# Examples Echo
``` rust
#![feature(async_closure)]
use tcpserver::tokio;
use tokio::io::AsyncReadExt;
use tcpserver::XBWrite;
use tcpserver::Builder;

#[tokio::main]
async fn main() {
    let tcpserver = Builder::new("0.0.0.0:5555")
        .set_connect_event(|addr| {
            println!("{:?} connect", addr);
            true
        }).set_input_event(async move |mut reader, peer| {
        let mut buff = [0; 4096];
        while let Ok(len) = peer.reader.read(&mut buff).await  {
            if len==0{
                break;
            }
            println!("{:?}",&buff[..len]);
            peer.send(buff[..len].to_vec()).await.unwrap();
        }
        println!("{:?} disconnect",peer.addr);
    }).build().await;

    tcpserver.start().await.unwrap();
}
```
