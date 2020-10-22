#![feature(async_closure)]
use tokio::io::AsyncReadExt;
use xbinary::XBWrite;
use tcpserver::Builder;

#[tokio::test]
async fn echo() {
    let tcpserver = Builder::new("0.0.0.0:5555")
        .set_connect_event(|addr| {
            println!("{:?} connect", addr);
            true
        }).set_input_event(async move |mut peer| {
            let mut buff = [0; 4096];

            while let Ok(len) = peer.reader.read(&mut buff).await {
                println!("{:?}",&buff[..len]);
                let mut writer = XBWrite::new();
                writer.write(&buff[..len]);
                peer.send_mut(writer).await.unwrap();
            }

            println!("{:?} disconnect",peer.addr);
        }).build().await;

    tcpserver.start().await.unwrap();
}