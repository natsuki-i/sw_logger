use clap::Parser;
use std::net::Ipv4Addr;

#[derive(Debug, Parser)]
#[command(author, version)]
pub struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    pub ip: Ipv4Addr,
    #[arg(short, long, default_value = "8080")]
    pub port: u16,
}
