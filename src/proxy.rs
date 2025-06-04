use pingora::upstreams::peer::Proxy;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::config::Config;
use tokio_socks::tcp::Socks5Stream;
use async_trait::async_trait;
use bytes::Bytes;
use log::info;


use pingora_core::upstreams::peer::HttpPeer;
use pingora_core::Result;
use pingora_http::ResponseHeader;
use pingora_proxy::{ProxyHttp, Session};

pub struct ProxyContext {
    
}

impl Default for ProxyContext {
    fn default() -> Self {
        Self {
            
        }
    }
}

pub async fn connect_via_socks5(addr: (&str, u16), proxy: (&str, u16)) -> std::io::Result<TcpStream> {
    let stream = Socks5Stream::connect(proxy, addr)
                                .await
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(stream.into_inner())
}

pub struct HttpService {
    //req_metric: prometheus::IntCounter,
}

impl HttpService {
    
}  


#[async_trait]
impl ProxyHttp for HttpService {
    type CTX = ProxyContext;

    fn new_ctx(&self) -> Self::CTX {
        ProxyContext::default()
    }

    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        // if session.req_header().uri.path().starts_with("/login")
        //     && !check_login(session.req_header())
        // {
        //     let _ = session
        //         .respond_error_with_body(403, Bytes::from_static(b"no way!"))
        //         .await;
        //     // true: early return as the response is already written
        //     return Ok(true);
        // }
        Ok(false)
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let addr = if session.req_header().uri.path().starts_with("/family") {
            ("1.0.0.1", 443)
        } else {
            ("192.168.50.42", 80)
        };

        // 使用 SOCKS5 代理（假设代理监听在 127.0.0.1:1080）
        let stream = connect_via_socks5(("192.168.50.42", 80), ("127.0.0.1", 7890)).await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)).unwrap();

        info!("connecting to {addr:?}");

        let peer = Box::new(HttpPeer::new(addr, false, "bilibili.com".to_string()));

        Ok(peer)
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // replace existing header if any
        upstream_response
            .insert_header("Server", "MyGateway")
            .unwrap();
        // because we don't support h3
        upstream_response.remove_header("alt-svc");

        Ok(())
    }


}



#[cfg(test)]
mod test {
    use socks5_proxy::server::new as socks5_new;
    use tokio::time::sleep;
    use tokio::net::TcpListener;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn start_listener() {
        let listener = TcpListener::bind("127.0.0.1:7891").await.unwrap();

        loop {
            let (mut socket, _) = listener.accept().await.unwrap();

            tokio::spawn(async move {
                let mut buffer = [0; 1024];
                match socket.read(&mut buffer).await {
                    Ok(_) => {
                        socket
                        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\na")
                        .await
                        .unwrap();
                    }
                    Err(e) => eprintln!("Failed to read from socket: {}", e),
                }
            });
        }
    }

    async fn start_socks5_server() {
        let socks5_server = 
            socks5_new("127.0.0.1:7890".parse().unwrap(), None).unwrap();
        socks5_server.run().await.unwrap();
    }

    #[tokio::test]
    async fn test_connect_via_socks5() {
        let proxy_addr = "localhost:7890";
        tokio::spawn(start_socks5_server());
        tokio::spawn(start_listener());

        //sleep(std::time::Duration::from_millis(100)).await;

        tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl_c");
    }


}
