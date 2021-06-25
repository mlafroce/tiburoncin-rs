extern crate mio;

pub mod net;

use crate::net::passthrough::Passtrough;
use std::io;
use std::net::{TcpListener, TcpStream};

fn main() -> io::Result<()> {
    let left_address = "0.0.0.0:8080";
    let right_address = "0.0.0.0:8081";
    run_mim(left_address, right_address)
}

fn run_mim(left_address: &str, right_address: &str) -> io::Result<()> {
    println!("Connecting to server");
    let right_con = TcpStream::connect(right_address)?;
    right_con.set_nonblocking(true)?;

    println!("Awaiting client");
    let listener = TcpListener::bind(left_address)?;
    let left_con = listener.accept()?;
    left_con.0.set_nonblocking(true)?;

    let left = mio::net::TcpStream::from_std(left_con.0);
    let right = mio::net::TcpStream::from_std(right_con);
    let mut passthrough = Passtrough::new(left, right);
    passthrough.run()
}
