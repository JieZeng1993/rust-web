use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use hyper::http::uri::Scheme;
use hyper::server::conn::Http;
use tokio::io::{AsyncRead, AsyncWrite};
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
    // let ep = Arc::new(ep.into_endpoint().map_to_response());
    loop {
        tokio::select! {
            res = listener.accept()=>{
                println!("res: {:?}", res);
                if let Ok((tcp_stream, remote_addr)) = res {
                    let alive_connections = alive_connections.clone();
                    let timeout_notify = timeout_notify.clone();
                    let notify = notify.clone();
                    // let ep = ep.clone();
                    tokio::spawn( async move{
                        //线程数加1
                        alive_connections.fetch_add(1,Ordering::SeqCst);
                        serve_connection(tcp_stream, local_addr.clone(),    remote_addr.clone(), Scheme::HTTP).await;
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

    Ok(())
}

async fn serve_connection(
    socket: impl AsyncRead + AsyncWrite + Send + Unpin + 'static,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    scheme: Scheme,
    // ep: Arc<dyn Endpoint<Output=Response>>,
) {
    let service = hyper::service::service_fn({

    });


    let conn = Http::new()
        .serve_connection(socket, service)
        .with_upgrades();

    conn.await;
}