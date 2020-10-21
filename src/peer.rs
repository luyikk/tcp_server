use std::error::Error;
use std::net::SocketAddr;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;
use xbinary::XBWrite;

pub struct TCPPeer {
    pub addr: SocketAddr,
    pub reader: OwnedReadHalf,
    pub sender: Sender<XBWrite>,
}

impl TCPPeer {
    /// 创建一个TCP PEER
    pub fn new(addr: SocketAddr, reader: OwnedReadHalf, sender: Sender<XBWrite>) -> TCPPeer {
        TCPPeer {
            addr,
            reader,
            sender,
        }
    }

    /// 获取发送句柄
    pub fn get_sender(&self) -> Sender<XBWrite> {
        self.sender.clone()
    }

    /// 发送
    pub async fn send(&self, buff: XBWrite) -> Result<(), Box<dyn Error>> {
        self.get_sender().send(buff).await?;
        Ok(())
    }

    /// 发送 mut 版
    pub async fn send_mut(&mut self, buff: XBWrite) -> Result<(), Box<dyn Error>> {
        self.sender.send(buff).await?;
        Ok(())
    }

    /// 掐线
    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        self.send(XBWrite::new()).await
    }
}
