use anyhow::*;
use aqueue::Actor;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;

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
    pub async fn send<T: Deref<Target = [u8]> + Send + Sync + 'static>(
        &mut self,
        buff: T,
    ) -> Result<usize> {
        if let Some(ref mut sender) = self.sender {
            Ok(sender.write(&buff).await?)
        } else {
            bail!("ConnectionReset")
        }
    }

    /// 掐线
    #[inline]
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut sender) = self.sender.take() {
            Ok(sender.shutdown().await?)
        } else {
            Ok(())
        }
    }
}

#[async_trait::async_trait]
pub trait IPeer {
    fn addr(&self) -> SocketAddr;
    async fn is_disconnect(&self) -> Result<bool>;
    async fn send<T: Deref<Target = [u8]> + Send + Sync + 'static>(&self, buff: T)
        -> Result<usize>;
    async fn disconnect(&self) -> Result<()>;
}

#[async_trait::async_trait]
impl IPeer for Actor<TCPPeer> {
    #[inline]
    fn addr(&self) -> SocketAddr {
        unsafe { self.deref_inner().addr }
    }

    #[inline]
    async fn is_disconnect(&self) -> Result<bool> {
        self.inner_call(async move |inner| Ok(inner.get().is_disconnect()))
            .await
    }

    #[inline]
    async fn send<T: Deref<Target = [u8]> + Send + Sync + 'static>(
        &self,
        buff: T,
    ) -> Result<usize> {
        ensure!(!buff.is_empty(), "send buff is null");
        self.inner_call(async move |inner| inner.get_mut().send(buff).await)
            .await
    }

    #[inline]
    async fn disconnect(&self) -> Result<()> {
        self.inner_call(async move |inner| inner.get_mut().disconnect().await)
            .await
    }
}
