use crate::{TCPPeer, ConnectEventType, TCPServer};
use std::future::Future;
use tokio::net::ToSocketAddrs;
use tokio::net::tcp::OwnedReadHalf;
use std::sync::Arc;
use aqueue::Actor;
use std::marker::PhantomData;

/// TCP server builder
pub struct  Builder<I,R,A,T>{
    input:Option<I>,
    connect_event:Option<ConnectEventType>,
    addr:A,
    _phantom1:PhantomData<R>,
    _phantom2:PhantomData<T>
}

impl<I, R,A,T> Builder<I,R,A,T>
    where
        I: Fn(OwnedReadHalf,Arc<Actor<TCPPeer>>,T) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send+'static,
        A: ToSocketAddrs,
        T: Clone+Send+'static{

    pub fn new(addr:A)->Builder<I, R,A,T>{
        Builder{
            input: None,
            connect_event: None,
            addr,
            _phantom1:PhantomData::default(),
            _phantom2:PhantomData::default()
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
    pub async fn build(mut self)->Arc<Actor<TCPServer<I,R,T>>>{
        if let Some(input)=self.input.take() {
            return if let Some(connect) = self.connect_event.take() {
                TCPServer::new(self.addr, input, Some(connect)).await.unwrap()
            } else {
                TCPServer::new(self.addr, input, None).await.unwrap()
            }
        }
        panic!("input event is no settings,please use set_input_event function set input event.");
    }
}