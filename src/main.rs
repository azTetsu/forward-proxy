mod proxy;
mod config;

use pingora::server::Server;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut server = Server::new(None)?;
    server.bootstrap();

    //server.add_listener("0.0.0.0:8080", proxy).unwrap();

    server.run_forever();
}