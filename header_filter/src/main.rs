use async_trait::async_trait;
use pingora::prelude::*;
use log::info;

pub struct FilterProxy;

#[async_trait]
impl ProxyHttp for FilterProxy {
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

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Add a custom header to the upstream request
        upstream_request.insert_header("X-Pingora-Sample", "FilterExample").unwrap();
        // Also ensure Host is set correctly
        upstream_request.insert_header("Host", "one.one.one.one").unwrap();
        info!("Modified upstream request headers");
        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Add a custom header to the response sent back to the client
        upstream_response.insert_header("X-Hello-From-Pingora", "v0.7.0").unwrap();
        
        // You can also remove headers
        upstream_response.remove_header("Server");
        
        info!("Modified response headers");
        Ok(())
    }
}

fn main() {
    env_logger::init();
    
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let mut my_proxy_service = http_proxy_service(&my_server.configuration, FilterProxy);
    
    my_proxy_service.add_tcp("0.0.0.0:6188");
    info!("Filter proxy listening on 0.0.0.0:6188");

    my_server.add_service(my_proxy_service);
    my_server.run_forever();
}
