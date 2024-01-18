use log::LevelFilter;
use std::error::Error;
use tcpclient::{SocketClientTrait, TcpClient};
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // set logger out
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    // connect echo server
    let client = TcpClient::connect(
        "127.0.0.1:5555",
        |_, client, mut reader| async move {
            // read buff from target server
            let mut buff = [0; 7];
            while let Ok(len) = reader.read_exact(&mut buff).await {
                // send buff to target server
                println!("{}", std::str::from_utf8(&buff[..len])?);
                client.send_ref(&buff[..len]).await?;
            }
            // return true need disconnect,false not disconnect
            // if true and the current state is disconnected, it will be ignored.
            Ok(true)
        },
        (),
    )
    .await?;

    // connect ok send buff to target server
    client.send_ref(b"1234567").await?;

    // test disconnect readline
    let mut str = "".into();
    std::io::stdin().read_line(&mut str)?;

    // disconnect target server
    client.disconnect().await?;
    // wait env logger out show
    std::io::stdin().read_line(&mut str)?;
    Ok(())
}
