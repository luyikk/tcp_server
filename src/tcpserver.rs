use crate::peer::TCPPeer;
use crate::IPeer;
use aqueue::{AError, AResult, Actor};
use log::*;
use std::error::Error;
use std::future::Future;
use std::io;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::task::JoinHandle;

pub type ConnectEventType = fn(SocketAddr) -> bool;

pub struct TCPServer<I, R, T> {
    listener: Option<TcpListener>,
    connect_event: Option<ConnectEventType>,
    input_event: Arc<I>,
    _phantom: PhantomData<R>,
    __phantom: PhantomData<T>,
}

unsafe impl<I, R, T> Send for TCPServer<I, R, T> {}
unsafe impl<I, R, T> Sync for TCPServer<I, R, T> {}

impl<I, R, T> TCPServer<I, R, T>
where
    I: Fn(OwnedReadHalf, Arc<Actor<TCPPeer>>, T) -> R + Send + Sync + 'static,
    R: Future<Output = ()> + Send + 'static,
    T: Clone + Send + 'static,
{
    /// 创建一个新的TCP服务
    pub(crate) async fn new<A: ToSocketAddrs>(
        addr: A,
        input: I,
        connect_event: Option<ConnectEventType>,
    ) -> Result<Arc<Actor<TCPServer<I, R, T>>>, Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Arc::new(Actor::new(TCPServer {
            listener: Some(listener),
            connect_event,
            input_event: Arc::new(input),
            _phantom: PhantomData::default(),
            __phantom: PhantomData::default(),
        })))
    }

    /// 启动TCP服务
    pub async fn start(&mut self, token: T) -> AResult<JoinHandle<io::Result<()>>> {
        if let Some(listener) = self.listener.take() {
            let connect_event = self.connect_event.take();
            let input_event = self.input_event.clone();
            let join: JoinHandle<io::Result<()>> = tokio::spawn(async move {
                loop {
                    let (socket, addr) = listener.accept().await?;
                    if let Some(ref connect_event) = connect_event {
                        if !connect_event(addr) {
                            warn!("addr:{} not connect", addr);
                            continue;
                        }
                    }
                    trace!("start read:{}", addr);
                    let (reader, sender) = socket.into_split();
                    //let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = bounded(1024);

                    // tokio::spawn(async move {
                    //     while let Ok(buff) = rx.recv().await {
                    //         if buff.is_empty() {
                    //             break;
                    //         } else if let Err(er) = sender.write(&buff).await {
                    //             error!("{} send buffer error:{}", addr, er);
                    //             break;
                    //         }
                    //     }
                    //
                    //     if let Err(er) = sender.shutdown().await {
                    //         trace!("{} disconnect error:{}", addr, er);
                    //     }
                    // });

                    let peer = TCPPeer::new(addr, sender);
                    let input = input_event.clone();
                    let peer_token = token.clone();
                    tokio::spawn(async move {
                        (*input)(reader, peer.clone(), peer_token).await;
                        if let Err(er) = peer.disconnect().await {
                            error!("disconnect client:{:?} err:{}", peer.addr(), er);
                        } else {
                            debug!("{} disconnect", peer.addr())
                        }
                    });
                }
            });

            return Ok(join);
        }

        Err(AError::StrErr("not listener or repeat start".into()))
    }
}

#[aqueue::aqueue_trait]
pub trait ITCPServer<T> {
    async fn start(&self, token: T) -> AResult<JoinHandle<io::Result<()>>>;
    async fn start_block(&self, token: T) -> Result<(), Box<dyn Error>>;
}

#[aqueue::aqueue_trait]
impl<I, R, T> ITCPServer<T> for Actor<TCPServer<I, R, T>>
where
    I: Fn(OwnedReadHalf, Arc<Actor<TCPPeer>>, T) -> R + Send + Sync + 'static,
    R: Future<Output = ()> + Send + 'static,
    T: Clone + Send + Sync + 'static,
{
    async fn start(&self, token: T) -> AResult<JoinHandle<io::Result<()>>> {
        self.inner_call(async move |inner| inner.get_mut().start(token).await)
            .await
    }

    async fn start_block(&self, token: T) -> Result<(), Box<dyn Error>> {
        Self::start(self, token).await?.await??;
        Ok(())
    }
}
