use std::error::Error;
use std::net::SocketAddr;
use tokio::net::tcp::OwnedWriteHalf;

use tokio::io::AsyncWriteExt;
use std::io;
use std::ops::Deref;
use aqueue::Actor;
use aqueue::AError::Other;
use std::sync::Arc;


pub struct TCPPeer {
    pub addr: SocketAddr,
    pub sender: OwnedWriteHalf,
}


impl TCPPeer {
    /// 创建一个TCP PEER
    #[inline]
    pub fn new(addr: SocketAddr, sender: OwnedWriteHalf) -> Arc<Actor<TCPPeer>> {
       Arc::new(Actor::new(TCPPeer {
            addr,
            sender
        }))
    }

    /// 发送
    #[inline]
    pub async fn send(&mut self, buff: &[u8]) -> io::Result<usize> {
       self.sender.write(buff).await
    }

    /// 掐线
    #[inline]
    pub async fn disconnect(&mut self) ->io::Result<()> {
        self.sender.shutdown().await
    }
}

#[aqueue::aqueue_trait]
pub trait IPeer{
    fn addr(&self)->SocketAddr;
    async fn send<T:Deref<Target=[u8]>+Send+Sync+'static>(&self,buff:T)->Result<usize,Box<dyn Error>>;
    async fn disconnect(&self)->Result<(),Box<dyn Error>>;
}

#[aqueue::aqueue_trait]
impl IPeer for Actor<TCPPeer>{
    fn addr(&self) -> SocketAddr {
        unsafe {
            self.deref_inner().addr
        }
    }

    async fn send<T: Deref<Target=[u8]> + Send + Sync + 'static>(&self, buff: T) -> Result<usize, Box<dyn Error>> {
       let size= self.inner_call(async move|inner|{
           match inner.get_mut().send(&buff).await{
               Ok(size)=>Ok(size),
               Err(er)=>Err(Other(er.into()))
           }

        }).await?;
        Ok(size)
    }

    async fn disconnect(&self) -> Result<(), Box<dyn Error>> {
        self.inner_call(async move|inner|{
            match inner.get_mut().disconnect().await {
                Ok(_) => Ok(()),
                Err(er) => Err(Other(er.into()))
            }
        }).await?;
        Ok(())
    }
}
