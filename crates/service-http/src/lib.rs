use std::net::Ipv4Addr;

pub mod middleware;

#[derive(Clone, Debug)]
pub struct ClientIp(pub Ipv4Addr);

#[derive(Clone, Debug)]
pub struct ApiKey(pub Option<String>);
