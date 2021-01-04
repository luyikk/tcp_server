use std::error::Error;
use std::net::SocketAddr;
use tokio::net::tcp::OwnedWriteHalf;

use tokio::io::{AsyncWriteExt, ErrorKind};
use std::io;
use std::ops::Deref;
use aqueue::Actor;
use aqueue::AError::Other;
use std::sync::Arc;



pub struct TCPPeer {
    pub addr: SocketAddr,
    pub sender:Option<OwnedWriteHalf>,
}


impl TCPPeer {
    /// 创建一个TCP PEER
    #[inline]
    pub fn new(addr: SocketAddr, sender: OwnedWriteHalf) -> Arc<Actor<TCPPeer>> {
       Arc::new(Actor::new(TCPPeer {
            addr,
            sender:Some(sender)
        }))
    }
    /// 是否断线
    #[inline]
    pub fn is_disconnect(&self)->bool{
        self.sender.is_none()
    }

    /// 发送
    #[inline]
    pub async fn send(&mut self, buff: &[u8]) -> io::Result<usize> {
       if let Some(ref mut sender)=self.sender{
           sender.write(buff).await
       }else {
           Err(io::Error::new(ErrorKind::ConnectionAborted,"ConnectionAborted"))
       }
    }

    /// 掐线
    #[inline]
    pub async fn disconnect(&mut self) ->io::Result<()> {
        if let Some(mut sender)=self.sender.take() {
            sender.shutdown().await
        }
        else{
            Err(io::Error::new(ErrorKind::ConnectionAborted,"ConnectionAborted"))
        }
    }
}

#[aqueue::aqueue_trait]
pub trait IPeer{
    fn addr(&self)->SocketAddr;
    async fn is_disconnect(&self)-> Result<bool,Box<dyn Error>> ;
    async fn send<T:Deref<Target=[u8]>+Send+Sync+'static>(&self,buff:T)->Result<usize,Box<dyn Error>>;
    async fn disconnect(&self)->Result<(),Box<dyn Error>>;
}

#[aqueue::aqueue_trait]
impl IPeer for Actor<TCPPeer>{
    #[inline]
    fn addr(&self) -> SocketAddr {
        unsafe {
            self.deref_inner().addr
        }
    }

    #[inline]
    async fn is_disconnect(&self) -> Result<bool,Box<dyn Error>> {
        Ok(self.inner_call(async move |inner| {
            Ok(inner.get().is_disconnect())
        }).await?)
    }

    #[inline]
    async fn send<T: Deref<Target=[u8]> + Send + Sync + 'static>(&self, buff: T) -> Result<usize, Box<dyn Error>> {
       let size= self.inner_call(async move|inner|{
           match inner.get_mut().send(&buff).await{
               Ok(size)=>Ok(size),
               Err(er)=>Err(Other(er.into()))
           }

        }).await?;
        Ok(size)
    }

    #[inline]
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
