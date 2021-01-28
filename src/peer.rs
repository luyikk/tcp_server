use std::net::SocketAddr;
use aqueue::{Actor, AResult};
use aqueue::AError::Other;
use std::sync::{Arc};
use tokio::sync::mpsc::Sender;
use std::error::Error;


pub struct TCPPeer {
    pub addr: SocketAddr,
    pub sender:Option<Sender<Vec<u8>>>,
}


impl TCPPeer {
    /// 创建一个TCP PEER
    #[inline]
    pub fn new(addr: SocketAddr, sender: Sender<Vec<u8>>) -> Arc<Actor<TCPPeer>> {
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
    pub async fn send(&self, buff: Vec<u8>) ->Result<(),Box<dyn Error+ Send + Sync>> {
       if let Some(sender)=self.sender.clone(){
           sender.send(buff).await?;
           Ok(())
       }else {
           Err("ConnectionReset".into())
       }
    }

    /// 掐线
    #[inline]
    pub async fn disconnect(&mut self) ->Result<(),Box<dyn Error+ Send + Sync>> {
        if let Some(sender)=self.sender.take() {
            sender.send(Vec::default()).await?;
            Ok(())
        }
        else{
           Ok(())
        }
    }
}

#[aqueue::aqueue_trait]
pub trait IPeer{
    fn addr(&self)->SocketAddr;
    async fn is_disconnect(&self)-> AResult<bool> ;
    async fn send(&self,buff:Vec<u8>)->AResult<()>;
    async fn disconnect(&self)->AResult<()>;
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
    async fn is_disconnect(&self) -> AResult<bool> {
        self.inner_call(async move |inner| {
            Ok(inner.get().is_disconnect())
        }).await
    }

    #[inline]
    async fn send(&self, buff: Vec<u8>) -> AResult<()> {
        unsafe {
            match self.deref_inner().send(buff).await {
                Ok(_) => Ok(()),
                Err(er) => Err(Other(er.into()))
            }
        }
    }

    #[inline]
    async fn disconnect(&self) -> AResult<()> {
        self.inner_call(async move|inner|{
            match inner.get_mut().disconnect().await {
                Ok(_) => Ok(()),
                Err(er) => Err(Other(er.into()))
            }
        }).await
    }
}
