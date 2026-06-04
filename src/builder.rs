use crate::{ConnectEventType, TCPPeer, TCPServer};

use aqueue::Actor;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, ReadHalf};
use tokio::net::{TcpStream, ToSocketAddrs};

/// TCP server builder
pub struct Builder<I, R, A, T, B, C, IST> {
    input: Option<I>,
    connect_event: Option<ConnectEventType>,
    stream_init: Option<IST>,
    addr: A,
    _phantom1: PhantomData<R>,
    _phantom2: PhantomData<T>,
    _phantom3: PhantomData<C>,
    _phantom4: PhantomData<B>,
}

impl<I, R, A, T, B, C, IST> Builder<I, R, A, T, B, C, IST>
where
    I: Fn(ReadHalf<C>, Arc<Actor<TCPPeer<C>>>, T) -> R + Send + Sync + 'static,
    R: Future<Output = anyhow::Result<()>> + Send + 'static,
    A: ToSocketAddrs,
    T: Clone + Send + 'static,
    B: Future<Output = anyhow::Result<C>> + Send + 'static,
    C: AsyncRead + AsyncWrite + Send + 'static,
    IST: Fn(TcpStream) -> B + Send + Sync + 'static,
{
    pub fn new(addr: A) -> Builder<I, R, A, T, B, C, IST> {
        Builder {
            input: None,
            connect_event: None,
            stream_init: None,
            addr,
            _phantom1: Default::default(),
            _phantom2: Default::default(),
            _phantom3: Default::default(),
            _phantom4: Default::default(),
        }
    }

    /// Set the input event handler — called for each connection to read and respond.
    ///
    /// 设置TCP server 输入事件
    pub fn set_input_event(mut self, f: I) -> Self {
        self.input = Some(f);
        self
    }

    /// Set the connect event filter — return `false` to reject a connection.
    ///
    /// 设置TCP server 连接事件
    pub fn set_connect_event(mut self, c: ConnectEventType) -> Self {
        self.connect_event = Some(c);
        self
    }

    /// Set the stream initializer — transforms a raw TcpStream (e.g. into SslStream).
    /// Examples: TcpStream (passthrough), SslStream, or GZIPStream.
    ///
    /// 设置输入流类型,例如TCPStream,SSLStream or GZIPStream
    pub fn set_stream_init(mut self, c: IST) -> Self {
        self.stream_init = Some(c);
        self
    }

    /// Build the TCPServer. Panics if `set_input_event` or `set_stream_init` was not called.
    ///
    /// 生成TCPSERVER,如果没有设置 tcp input 将报错
    pub async fn build(mut self) -> Arc<Actor<TCPServer<I, R, T, B, C, IST>>> {
        let input = self.input.take().unwrap_or_else(|| {
            panic!(
                "input event is no settings,please use set_input_event function set input event."
            )
        });
        let stream_init = self.stream_init.take().unwrap_or_else(|| {
            panic!("stream_init is no settings,please use set_stream_init function.")
        });
        let connect = self.connect_event.take();
        TCPServer::new(self.addr, stream_init, input, connect)
            .await
            .unwrap()
    }
}
