use crate::peer::TCPPeer;
use crate::IPeer;
use anyhow::{bail, Result};
use aqueue::Actor;
use log::*;
use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, ReadHalf};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::task::JoinHandle;

pub type ConnectEventType = fn(SocketAddr) -> bool;

pub struct TCPServer<I, R, T, B, C, IST> {
    listener: Option<TcpListener>,
    connect_event: Option<ConnectEventType>,
    stream_init: Arc<IST>,
    input_event: Arc<I>,
    _phantom1: PhantomData<R>,
    _phantom2: PhantomData<T>,
    _phantom3: PhantomData<C>,
    _phantom4: PhantomData<B>,
}

unsafe impl<I, R, T, B, C, IST> Send for TCPServer<I, R, T, B, C, IST> {}
unsafe impl<I, R, T, B, C, IST> Sync for TCPServer<I, R, T, B, C, IST> {}

impl<I, R, T, B, C, IST> TCPServer<I, R, T, B, C, IST>
where
    I: Fn(ReadHalf<C>, Arc<Actor<TCPPeer<C>>>, T) -> R + Send + Sync + 'static,
    R: Future<Output = Result<()>> + Send + 'static,
    T: Clone + Send + 'static,
    B: Future<Output = Result<C>> + Send + 'static,
    C: AsyncRead + AsyncWrite + Send + 'static,
    IST: Fn(TcpStream) -> B + Send + Sync + 'static,
{
    /// 创建一个新的TCP服务
    pub(crate) async fn new<A: ToSocketAddrs>(
        addr: A,
        stream_init: IST,
        input: I,
        connect_event: Option<ConnectEventType>,
    ) -> Result<Arc<Actor<TCPServer<I, R, T, B, C, IST>>>, Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Arc::new(Actor::new(TCPServer {
            listener: Some(listener),
            connect_event,
            stream_init: Arc::new(stream_init),
            input_event: Arc::new(input),
            _phantom1: Default::default(),
            _phantom2: Default::default(),
            _phantom3: Default::default(),
            _phantom4: Default::default(),
        })))
    }

    /// 启动TCP服务
    pub async fn start(&mut self, token: T) -> Result<JoinHandle<Result<()>>> {
        if let Some(listener) = self.listener.take() {
            let connect_event = self.connect_event.take();
            let input_event = self.input_event.clone();
            let stream_init = self.stream_init.clone();
            let join: JoinHandle<Result<()>> = tokio::spawn(async move {
                loop {
                    let (socket, addr) = listener.accept().await?;
                    if let Some(ref connect_event) = connect_event {
                        if !connect_event(addr) {
                            warn!("addr:{} not connect", addr);
                            continue;
                        }
                    }
                    trace!("start read:{}", addr);
                    let input = input_event.clone();
                    let peer_token = token.clone();
                    let stream_init = stream_init.clone();
                    tokio::spawn(async move {
                        match (*stream_init)(socket).await {
                            Ok(socket) => {
                                let (reader, sender) = tokio::io::split(socket);
                                let peer = TCPPeer::new(addr, sender);
                                if let Err(err) = (*input)(reader, peer.clone(), peer_token).await {
                                    error!("input data error:{}", err);
                                }
                                if let Err(er) = peer.disconnect().await {
                                    debug!("disconnect client:{:?} err:{}", peer.addr(), er);
                                } else {
                                    debug!("{} disconnect", peer.addr())
                                }
                            }
                            Err(err) => {
                                warn!("init stream err:{}", err);
                            }
                        }
                    });
                }
            });

            return Ok(join);
        }

        bail!("not listener or repeat start")
    }
}

#[async_trait::async_trait]
pub trait ITCPServer<T> {
    async fn start(&self, token: T) -> Result<JoinHandle<Result<()>>>;
    async fn start_block(&self, token: T) -> Result<()>;
}

#[async_trait::async_trait]
impl<I, R, T, B, C, IST> ITCPServer<T> for Actor<TCPServer<I, R, T, B, C, IST>>
where
    I: Fn(ReadHalf<C>, Arc<Actor<TCPPeer<C>>>, T) -> R + Send + Sync + 'static,
    R: Future<Output = Result<()>> + Send + 'static,
    T: Clone + Send + Sync + 'static,
    B: Future<Output = Result<C>> + Send + 'static,
    C: AsyncRead + AsyncWrite + Send + 'static,
    IST: Fn(TcpStream) -> B + Send + Sync + 'static,
{
    async fn start(&self, token: T) -> Result<JoinHandle<Result<()>>> {
        self.inner_call(|inner| async move { inner.get_mut().start(token).await })
            .await
    }

    async fn start_block(&self, token: T) -> Result<()> {
        Self::start(self, token).await?.await??;
        Ok(())
    }
}
