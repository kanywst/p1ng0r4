use async_trait::async_trait;
use pingora::prelude::*;

pub struct MyProxy;

#[async_trait]
impl ProxyHttp for MyProxy {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let peer = Box::new(HttpPeer::new(
            "1.1.1.1:443",
            true,
            "one.one.one.one".to_string(),
        ));
        Ok(peer)
    }
}

fn main() {
    env_logger::init();
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let mut my_proxy_service = http_proxy_service(&my_server.configuration, MyProxy);
    my_proxy_service.add_tcp("0.0.0.0:8080");

    my_server.add_service(my_proxy_service);
    my_server.run_forever();
}
