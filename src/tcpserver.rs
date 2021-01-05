use log::*;
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::{Arc};
use tokio::net::{TcpListener, ToSocketAddrs};
use crate::peer::TCPPeer;
use std::marker::PhantomData;
use tokio::net::tcp::OwnedReadHalf;
use aqueue::{Actor, AError, AResult};
use crate::IPeer;
use std::io;
use tokio::task::JoinHandle;

pub type ConnectEventType = fn(SocketAddr) -> bool;

pub struct TCPServer<I, R> {
    listener: Option<TcpListener>,
    connect_event: Option<ConnectEventType>,
    input_event: Arc<I>,
    phantom:PhantomData<R>
}

unsafe impl <I,R> Send for TCPServer<I, R>{}
unsafe impl <I,R> Sync for TCPServer<I, R>{}

impl<I, R> TCPServer<I, R>
    where
        I: Fn(OwnedReadHalf,Arc<Actor<TCPPeer>>) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send+'static,
{
    /// 创建一个新的TCP服务
    pub(crate) async fn new<T: ToSocketAddrs>(
        addr: T,
        input: I,
        connect_event:Option<ConnectEventType>
    ) -> Result<Arc<Actor<TCPServer<I, R>>>, Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Arc::new(Actor::new(TCPServer {
            listener:Some(listener),
            connect_event,
            input_event: Arc::new(input),
            phantom:PhantomData::default()
        })))
    }

    /// 启动TCP服务
    pub async fn start(&mut self) -> AResult<JoinHandle<io::Result<()>>> {
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
                    let peer = TCPPeer::new(addr, sender);
                    let input = input_event.clone();
                    tokio::spawn(async move {
                        (*input)(reader, peer.clone()).await;
                        if let Err(er) = peer.disconnect().await {
                            error!("disconnect client:{:?} err:{}", peer.addr(), er);
                        }
                    });
                }

            });

            return Ok(join)
        }

        Err(AError::StrErr("not listener or repeat start".into()))
    }
}

#[aqueue::aqueue_trait]
pub trait ITCPServer{
    async fn start(&self)->AResult<JoinHandle<io::Result<()>>>;
    async fn start_block(&self)->Result<(),Box<dyn Error>>;
}

#[aqueue::aqueue_trait]
impl <I, R> ITCPServer  for Actor<TCPServer<I, R>>
    where
        I: Fn(OwnedReadHalf,Arc<Actor<TCPPeer>>) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send+'static{

   async fn start(&self)->AResult<JoinHandle<io::Result<()>>> {
        self.inner_call(async move |inner| {
            inner.get_mut().start().await
        }).await
    }

    async fn start_block(&self) -> Result<(), Box<dyn Error>> {
        Self::start(self).await?.await??;
        Ok(())
    }
}