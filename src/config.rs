use std::net::SocketAddr;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: SocketAddr,
    pub tunnel: SocketAddr,
    pub upsteams: Vec<Upstream>,
}

impl Config {

}

#[derive(Debug, Deserialize)]
struct Upstream {
    host: String,
    target: String,
    uri: String,
}

impl Upstream {

}