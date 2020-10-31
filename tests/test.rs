#![feature(async_closure)]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use xbinary::XBWrite;
use tcpserver::Builder;
use std::error::Error;


#[tokio::test]
async fn echo_server() {
    let tcpserver = Builder::new("0.0.0.0:5555")
        .set_connect_event(|addr| {
            println!("{:?} connect", addr);
            true
        }).set_input_event(async move |mut peer| {
        let mut buff = [0; 4096];

        while let Ok(len) = peer.reader.read(&mut buff).await {
            if len == 0 {
                break;
            }
            println!("{:?}", &buff[..len]);
            let mut writer = XBWrite::new();
            writer.write(&buff[..len]);
            peer.send_mut(writer).await.unwrap();
        }

        println!("{:?} disconnect", peer.addr);

        panic!("close");

    }).build().await;

    tcpserver.start().await.unwrap();
}


#[tokio::test]
async fn echo_client()->Result<(),Box<dyn Error>> {
    let mut tcp_stream = tokio::net::TcpStream::connect("127.0.0.1:5555").await?;
    let data=b"12231222222221";
    let mut read =[0;14];
    for _ in 0..100 {
        tcp_stream.write(data).await?;
        let len= tcp_stream.read(&mut read).await?;
        if len !=0 {
            assert_eq!(*data, read);
        }
        else{
            println!("disconnect");
            break;
        }
    }

    Ok(())
}