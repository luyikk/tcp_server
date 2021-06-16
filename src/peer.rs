use anyhow::*;
use aqueue::Actor;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::WriteHalf;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

pub struct TCPPeer<T> {
    pub addr: SocketAddr,
    pub sender: Option<WriteHalf<T>>,
}

impl<T> TCPPeer<T>
where
    T: AsyncRead + AsyncWrite + Send + 'static,
{
    /// 创建一个TCP PEER
    #[inline]
    pub fn new(addr: SocketAddr, sender: WriteHalf<T>) -> Arc<Actor<TCPPeer<T>>> {
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
    pub async fn send<'a>(&'a mut self, buff: &'a [u8]) -> Result<usize> {
        if let Some(ref mut sender) = self.sender {
            Ok(sender.write(buff).await?)
        } else {
            bail!("ConnectionReset")
        }
    }

    /// 发送全部
    #[inline]
    pub async fn send_all<'a>(&'a mut self, buff: &'a [u8]) -> Result<()> {
        if let Some(ref mut sender) = self.sender {
            sender.write_all(buff).await?;
            Ok(())
        } else {
            bail!("ConnectionReset")
        }
    }

    /// flush
    #[inline]
    pub async fn flush(&mut self) -> Result<()> {
        if let Some(ref mut sender) = self.sender {
            sender.flush().await?;
            Ok(())
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
pub trait IPeer: Sync + Send {
    fn addr(&self) -> SocketAddr;
    async fn is_disconnect(&self) -> Result<bool>;
    async fn send<'a>(&'a self, buff: &'a [u8]) -> Result<usize>;
    async fn send_all<'a>(&'a self, buff: &'a [u8]) -> Result<()>;
    async fn flush(&mut self) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
}

#[async_trait::async_trait]
impl<T> IPeer for Actor<TCPPeer<T>>
where
    T: AsyncRead + AsyncWrite + Send + 'static,
{
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
    async fn send<'a>(&'a self, buff: &'a [u8]) -> Result<usize> {
        ensure!(!buff.is_empty(), "send buff is null");
        unsafe {
            self.inner_call_ref(async move |inner| inner.get_mut().send(buff).await)
                .await
        }
    }
    #[inline]
    async fn send_all<'a>(&'a self, buff: &'a [u8]) -> Result<()> {
        ensure!(!buff.is_empty(), "send buff is null");
        unsafe {
            self.inner_call_ref(async move |inner| inner.get_mut().send_all(buff).await)
                .await
        }
    }
    #[inline]
    async fn flush(&mut self) -> Result<()> {
        self.inner_call(async move |inner| inner.get_mut().flush().await)
            .await
    }

    #[inline]
    async fn disconnect(&self) -> Result<()> {
        self.inner_call(async move |inner| inner.get_mut().disconnect().await)
            .await
    }
}
