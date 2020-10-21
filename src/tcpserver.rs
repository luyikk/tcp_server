use log::*;
use std::cell::RefCell;
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::option::Option::Some;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use xbinary::XBWrite;
use crate::peer::TCPPeer;

pub type ConnectEventType = fn(SocketAddr) -> bool;

pub struct TCPServer<I, R>
    where
        I: Fn(TCPPeer) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send,
{
    listener: RefCell<Option<TcpListener>>,
    connect_event: RefCell<Option<ConnectEventType>>,
    input_event: Arc<I>,
}

impl<I, R> TCPServer<I, R>
    where
        I: Fn(TCPPeer) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send,
{
    /// 创建一个新的TCP服务
    pub async fn new<T: ToSocketAddrs>(
        addr: T,
        input: I,
    ) -> Result<TCPServer<I, R>, Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        Ok(TCPServer {
            listener: RefCell::new(Some(listener)),
            connect_event: RefCell::new(None),
            input_event: Arc::new(input),
        })
    }

    /// 设置连接事件通知
    pub fn set_connection_event(&self, f: ConnectEventType) {
        self.connect_event.replace(Some(f));
    }

    /// 启动TCP服务
    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        if let Some(mut listener) = self.listener.borrow_mut().take() {
            loop {
                let (socket, addr) = listener.accept().await?;
                if let Some(connect_event) = *self.connect_event.borrow() {
                    if !connect_event(addr) {
                        warn!("addr:{} not connect", addr);
                        continue;
                    }
                }
                trace!("start read:{}", addr);

                let (tx, mut rx): (Sender<XBWrite>, Receiver<XBWrite>) = channel(1024);
                let (reader, mut sender) = socket.into_split();
                tokio::spawn(async move {
                    while let Some(buff) = rx.recv().await {
                        if buff.is_empty() {
                            if let Err(er) = sender.shutdown().await {
                                error!("{} disconnect error:{}", addr, er);
                            }
                            break;
                        } else if let Err(er) = sender.write(&buff).await {
                            error!("{} send buffer error:{}", addr, er);
                        }
                    }
                });

                let peer = TCPPeer::new(addr, reader, tx);
                let input = self.input_event.clone();
                tokio::spawn(async move {
                    (*input)(peer).await;
                });
            }
        }

        Err("not listener or repeat start".into())
    }
}
