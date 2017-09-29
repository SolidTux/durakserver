use std::net::{TcpListener, TcpStream};

fn run_server(address: &'static str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;

    for stream in listener.incoming() {
        handle_client(stream?);
    }

    Ok(())
}

fn handle_client(stream: TcpStream) {

}

fn main() {
    println!("Hello, world!");
    run_server("0.0.0.0:2342").unwrap();
}
