mod sample_structs;

use cachem::Server;
use cachem_example::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cnc_rec, mut server) = Server::new("0.0.0.0:55555".into());

    server.add(CacheName::A, Arc::new(Box::new(ACache::new(cnc_rec))));

    server.listen_cnc();
    server.listen_tcp().await;

    Ok(())
}
