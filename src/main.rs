use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{copy_bidirectional};
use anyhow::Result;

/// rustproxy - simple configurable TCP proxy with logging
#[derive(Parser)]
struct Args {
    /// Listen address, e.g., 127.0.0.1:12345
    #[arg(short, long)]
    listen: String,

    /// Target address to forward to, e.g., example.com:80
    #[arg(short, long)]
    target: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("rustproxy starting...");
    println!("Listening on: {}", args.listen);
    println!("Forwarding to target: {}", args.target);

    let listener = TcpListener::bind(&args.listen).await?;
    loop {
        let (mut inbound, addr) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[!] Failed to accept connection: {e}");
                continue;
            }
        };
        println!("[+] Accepted connection from {addr}");

        let target = args.target.clone();
        tokio::spawn(async move {
            println!("[{addr}] Connecting to target {target}...");
            let mut outbound = match TcpStream::connect(&target).await {
                Ok(s) => {
                    println!("[{addr}] Connected to target");
                    s
                },
                Err(e) => {
                    eprintln!("[{addr}] Failed to connect to target {target}: {e}");
                    return;
                }
            };

            match copy_bidirectional(&mut inbound, &mut outbound).await {
                Ok((from_client, from_server)) => {
                    println!(
                        "[{addr}] Closed. Bytes relayed: client→server {from_client}, server→client {from_server}"
                    );
                }
                Err(e) => {
                    eprintln!("[{addr}] Connection error during forwarding: {e}");
                }
            }
        });
    }
}
