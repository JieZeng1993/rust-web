use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::net::TcpListener as TokioTcpListener;
use tokio::sync::Notify;

mod server;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut listener = TokioTcpListener::bind("127.0.0.1:666").await?;
    println!("listener {listener:?}");
    let local_addr = listener.local_addr()?;
    println!("local_addr {local_addr:?}");

    //存活连接数
    //安全关机时，确保没有存在的连接
    let alive_connections = Arc::new(AtomicUsize::new(0));
    //超时同步器,暂时没有超时
    let timeout_notify = Arc::new(Notify::new());
    //同步器
    let notify = Arc::new(Notify::new());
    loop {
        tokio::select! {
            res = listener.accept()=>{
                println!("res: {:?}", res);
                if let Ok((tcp_stream, socket_addr)) = res {
                    let alive_connections = alive_connections.clone();
                    let timeout_notify = timeout_notify.clone();
                    let notify = notify.clone();
                    tokio::spawn( async move{
                        //线程数加1
                        alive_connections.fetch_add(1,Ordering::SeqCst);

                        if alive_connections.fetch_sub(1, Ordering::SeqCst) == 1 {
                            //最后一个连接，唤醒主线程
                            notify.notify_one();
                        }
                    });
                }
            }
        }
    }

    if alive_connections.load(Ordering::SeqCst) > 0 {
        //存在连接，等待连接全部关闭
        notify.notified().await;
    }

    OK(())
}
