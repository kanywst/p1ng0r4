use async_trait::async_trait;
use pingora::prelude::*;
use pingora::lb::LoadBalancer;
use pingora::lb::selection::RoundRobin;
use pingora::lb::health_check::TcpHealthCheck;
use log::info;
use std::sync::Arc;
use std::time::Duration;

pub struct MyLB {
    lb: Arc<LoadBalancer<RoundRobin>>,
}

#[async_trait]
impl ProxyHttp for MyLB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        // Select an upstream using Round Robin
        let upstream = self.lb.select(b"", 256).unwrap();
        info!("Selected upstream: {:?}", upstream);

        // For this example, we assume upstreams are HTTPS and use one.one.one.one as SNI
        // In a real scenario, you'd map the selected upstream to its SNI/TLS settings
        let peer = Box::new(HttpPeer::new(upstream, true, "one.one.one.one".to_string()));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Set the Host header to match what the upstream expects
        upstream_request.insert_header("Host", "one.one.one.one").unwrap();
        Ok(())
    }
}

fn main() {
    env_logger::init();

    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    // Define multiple upstreams
    let upstreams = vec![
        "1.1.1.1:443",
        "1.0.0.1:443",
        "127.0.0.1:80", // This one will likely fail health check
    ];

    let mut lb = LoadBalancer::try_from_iter(upstreams).unwrap();
    
    // Add health check
    let hc = TcpHealthCheck::new();
    lb.set_health_check(hc);
    lb.health_check_frequency = Some(Duration::from_secs(5));

    // Background service for health check
    let background = background_service("health check", lb);
    let lb_task = background.task();

    let mut my_proxy_service = http_proxy_service(
        &my_server.configuration,
        MyLB { lb: lb_task },
    );
    
    my_proxy_service.add_tcp("0.0.0.0:6188");
    info!("Load balanced proxy listening on 0.0.0.0:6188");

    my_server.add_service(my_proxy_service);
    my_server.add_service(background);
    my_server.run_forever();
}
