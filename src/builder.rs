use crate::{TCPPeer, ConnectEventType, TCPServer};
use std::future::Future;
use tokio::net::ToSocketAddrs;
use tokio::net::tcp::OwnedReadHalf;
use std::sync::Arc;
use aqueue::Actor;
use std::marker::PhantomData;

/// TCP server builder
pub struct  Builder<I,R,T>{
    input:Option<I>,
    connect_event:Option<ConnectEventType>,
    addr:T,
    _mask:PhantomData<R>
}

impl<I, R,T> Builder<I, R,T>
    where
        I: Fn(OwnedReadHalf,Arc<Actor<TCPPeer>>) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send,
        T: ToSocketAddrs{

    pub fn new(addr:T)->Builder<I, R,T>{
        Builder{
            input: None,
            connect_event: None,
            addr,
            _mask:PhantomData::default()
        }
    }

    /// 设置TCP server 输入事件
    pub fn set_input_event(mut self,f:I)->Self{
        self.input=Some(f);
        self
    }

    /// 设置TCP server 连接事件
    pub fn set_connect_event(mut self,c:ConnectEventType)->Self{
        self.connect_event=Some(c);
        self
    }

    /// 生成TCPSERVER,如果没有设置 tcp input 将报错
    pub async fn build(mut self)->TCPServer<I,R>{
        if let Some(input)=self.input.take() {
            let tcp= TCPServer::new(self.addr, input).await.unwrap();
            if let Some(connect)=self.connect_event.take() {
                tcp.set_connection_event(connect);
            }
            return tcp;
        }
        panic!("input event is no settings,please use set_input_event function set input event.");
    }
}