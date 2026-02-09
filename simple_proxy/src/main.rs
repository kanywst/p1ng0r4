use async_trait::async_trait;
use pingora::prelude::*;
use log::info;

pub struct MyProxy;

#[async_trait]
impl ProxyHttp for MyProxy {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        // Proxy to Cloudflare's 1.1.1.1 over HTTPS
        let peer = Box::new(HttpPeer::new(
            "1.1.1.1:443",
            true,
            "one.one.one.one".to_string(),
        ));
        info!("Proxying to 1.1.1.1:443");
        Ok(peer)
    }
}

fn main() {
    env_logger::init();
    
    // Create a server with default options
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    // Create an HTTP proxy service
    let mut my_proxy_service = http_proxy_service(&my_server.configuration, MyProxy);
    
    // Bind to port 6188 (default Pingora example port)
    my_proxy_service.add_tcp("0.0.0.0:6188");
    info!("Simple proxy listening on 0.0.0.0:6188");

    my_server.add_service(my_proxy_service);
    my_server.run_forever();
}
