use log::*;
use std::cell::RefCell;
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::option::Option::Some;
use std::sync::Arc;
use tokio::net::{TcpListener, ToSocketAddrs};
use crate::peer::TCPPeer;
use std::marker::PhantomData;
use tokio::net::tcp::OwnedReadHalf;
use aqueue::Actor;

pub type ConnectEventType = fn(SocketAddr) -> bool;

pub struct TCPServer<I, R> {
    listener: RefCell<Option<TcpListener>>,
    connect_event: RefCell<Option<ConnectEventType>>,
    input_event: Arc<I>,
    phantom:PhantomData<R>
}

impl<I, R> TCPServer<I, R>
    where
        I: Fn(OwnedReadHalf,Arc<Actor<TCPPeer>>) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send,
{
    /// 创建一个新的TCP服务
    pub(crate) async fn new<T: ToSocketAddrs>(
        addr: T,
        input: I,
    ) -> Result<TCPServer<I, R>, Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        Ok(TCPServer {
            listener: RefCell::new(Some(listener)),
            connect_event: RefCell::new(None),
            input_event: Arc::new(input),
            phantom:PhantomData::default()
        })
    }

    /// 设置连接事件通知
    pub(crate) fn set_connection_event(&self, f: ConnectEventType) {
        self.connect_event.replace(Some(f));
    }

    /// 启动TCP服务
    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        if let Some(listener) = self.listener.borrow_mut().take() {
            loop {
                let (socket, addr) = listener.accept().await?;
                if let Some(connect_event) = *self.connect_event.borrow() {
                    if !connect_event(addr) {
                        warn!("addr:{} not connect", addr);
                        continue;
                    }
                }
                trace!("start read:{}", addr);
                let (reader, sender) = socket.into_split();
                let peer = TCPPeer::new(addr, sender);
                let input = self.input_event.clone();
                tokio::spawn(async move {
                    (*input)(reader, peer).await;
                });
            }
        }

        Err("not listener or repeat start".into())
    }
}
