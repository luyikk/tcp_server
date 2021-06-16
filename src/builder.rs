use crate::{ConnectEventType, TCPServer, StreamInitType, IPeer};
use aqueue::Actor;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::net::ToSocketAddrs;
use tokio::io::{ReadHalf, AsyncRead, AsyncWrite};
use anyhow::*;

/// TCP server builder
pub struct Builder<I, R, A, T,B,C> {
    input: Option<I>,
    connect_event: Option<ConnectEventType>,
    stream_init:Option<StreamInitType<B>>,
    addr: A,
    _phantom1: PhantomData<R>,
    _phantom2: PhantomData<T>,
    _phantom3: PhantomData<C>,
}

impl<I, R, A, T, B,C> Builder<I, R, A, T, B,C>
where
    I: Fn(ReadHalf<C>, Arc<dyn IPeer>, T) -> R + Send + Sync + 'static,
    R: Future<Output = Result<()>> + Send + 'static,
    A: ToSocketAddrs,
    T: Clone + Send + 'static,
    B: Future<Output = Result<C>> + Send + 'static,
    C: AsyncRead + AsyncWrite + Send +'static
{
    pub fn new(addr: A) -> Builder<I, R, A, T,B,C> {
        Builder {
            input: None,
            connect_event: None,
            stream_init:None,
            addr,
            _phantom1: PhantomData::default(),
            _phantom2: PhantomData::default(),
            _phantom3: PhantomData::default()
        }
    }

    /// 设置TCP server 输入事件
    pub fn set_input_event(mut self, f: I) -> Self {
        self.input = Some(f);
        self
    }

    /// 设置TCP server 连接事件
    pub fn set_connect_event(mut self, c: ConnectEventType) -> Self {
        self.connect_event = Some(c);
        self
    }

    /// 设置输入流类型,例如TCPStream,SSLStream or GZIPStream
    pub fn set_stream_init(mut self,c:StreamInitType<B>)->Self{
        self.stream_init=Some(c);
        self
    }

    /// 生成TCPSERVER,如果没有设置 tcp input 将报错
    pub async fn build(mut self) -> Arc<Actor<TCPServer<I, R, T,B,C>>> {
        if let Some(input) = self.input.take() {
            if let Some(stream_init)=self.stream_init.take() {
                return if let Some(connect) = self.connect_event.take() {
                    TCPServer::new(self.addr,stream_init ,input, Some(connect))
                        .await
                        .unwrap()
                } else {
                    TCPServer::new(self.addr, stream_init,input, None).await.unwrap()
                };
            }
            panic!("stream_init is no settings,please use set_stream_init function.");
        }
        panic!("input event is no settings,please use set_input_event function set input event.");
    }
}