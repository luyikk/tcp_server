use aqueue::AError::Other;
use aqueue::{AResult, Actor};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::io::AsyncWriteExt;

pub struct TCPPeer {
    pub addr: SocketAddr,
    pub sender: Option<OwnedWriteHalf>,

}

impl TCPPeer {
    /// 创建一个TCP PEER
    #[inline]
    pub fn new(addr: SocketAddr, sender: OwnedWriteHalf) -> Arc<Actor<TCPPeer>> {
        Arc::new(Actor::new(TCPPeer {
            addr,
            sender: Some(sender),
        }))
    }
    /// 是否断线
    #[inline]
    pub fn is_disconnect(&self) -> bool {
        self.sender.is_none()
    }

    /// 发送
    #[inline]
    pub async fn send(&mut self, buff:Vec<u8>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if let Some(ref mut sender) = self.sender {
            Ok(sender.write(&buff).await?)
        } else {
            Err("ConnectionReset".into())
        }
    }

    /// 掐线
    #[inline]
    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(mut sender) = self.sender.take() {
            Ok(sender.shutdown().await?)
        } else {
            Ok(())
        }
    }
}

#[aqueue::aqueue_trait]
pub trait IPeer {
    fn addr(&self) -> SocketAddr;
    async fn is_disconnect(&self) -> AResult<bool>;
    async fn send(&self, buff:Vec<u8>) -> AResult<usize>;
    async fn disconnect(&self) -> AResult<()>;
}

#[aqueue::aqueue_trait]
impl IPeer for Actor<TCPPeer> {
    #[inline]
    fn addr(&self) -> SocketAddr {
        unsafe { self.deref_inner().addr }
    }

    #[inline]
    async fn is_disconnect(&self) -> AResult<bool> {
        self.inner_call(async move |inner| Ok(inner.get().is_disconnect()))
            .await
    }

    #[inline]
    async fn send(&self, buff:Vec<u8>) -> AResult<usize> {
        self.inner_call(async move|inner|{
            match inner.get_mut().send(buff).await{
                Ok(size)=>Ok(size),
                Err(er)=>Err(Other(er.into()))
            }
        }).await
    }

    #[inline]
    async fn disconnect(&self) -> AResult<()> {
        self.inner_call(
            async move |inner| match inner.get_mut().disconnect().await {
                Ok(_) => Ok(()),
                Err(er) => Err(Other(er.into())),
            },
        )
        .await
    }
}
