use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, split};
use std::sync::Arc;
use anyhow::Result;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use futures::future::join;

mod hexdump;

/// rustproxy - simple configurable TCP proxy with logging
#[derive(Parser)]
struct Args {
    /// Listen address
    #[arg(short, long)]
    listen: String,

    /// Target address to forward to
    #[arg(short, long)]
    target: String,

    /// Enable client to server traffic dump
    #[arg(long)]
    dump_c2s: bool,

    /// Enable server to client traffic dump
    #[arg(long)]
    dump_s2c: bool,

    /// Output file for traffic dump (optional)
    #[arg(long)]
    dump_file: Option<String>,
}

async fn copy_and_log<'a, R, W>(
    from: &str,
    to: &str,
    reader: &'a mut R,
    writer: &'a mut W,
    dump: bool,
    dump_file: Option<Arc<tokio::sync::Mutex<tokio::fs::File>>>,
) -> Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = [0; 8192];
    let mut total = 0;

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        
        if dump {
            let dump_text = hexdump::format_hex_dump(&buf[..n], &format!("[{} → {}] ", from, to));
            print!("{}", dump_text);
            
            if let Some(file) = &dump_file {
                let mut file = file.lock().await;
                file.write_all(dump_text.as_bytes()).await?;
            }
        }

        writer.write_all(&buf[..n]).await?;
        total += n as u64;
    }
    Ok(total)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("rustproxy starting...");
    println!("Listening on: {}", args.listen);
    println!("Forwarding to target: {}", args.target);

    let listener = TcpListener::bind(&args.listen).await?;
    let dump_file = if let Some(path) = args.dump_file {
        Some(Arc::new(tokio::sync::Mutex::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await?,
        )))
    } else {
        None
    };

    loop {
        let (inbound, addr) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[!] Failed to accept connection: {e}");
                continue;
            }
        };
        println!("[+] Accepted connection from {addr}");

        let target = args.target.clone();
        let dump_file = dump_file.clone();
        let dump_c2s = args.dump_c2s;
        let dump_s2c = args.dump_s2c;

        tokio::spawn(async move {
            println!("[{addr}] Connecting to target {target}...");
            let outbound = match TcpStream::connect(&target).await {
                Ok(s) => {
                    println!("[{addr}] Connected to target");
                    s
                },
                Err(e) => {
                    eprintln!("[{addr}] Failed to connect to target {target}: {e}");
                    return;
                }
            };

            let (mut inbound_read, mut inbound_write) = split(inbound);
            let (mut outbound_read, mut outbound_write) = split(outbound);

            let (from_client, from_server) = match join(
                copy_and_log(
                    "CLIENT",
                    "SERVER",
                    &mut inbound_read,
                    &mut outbound_write,
                    dump_c2s,
                    dump_file.clone(),
                ),
                copy_and_log(
                    "SERVER",
                    "CLIENT",
                    &mut outbound_read,
                    &mut inbound_write,
                    dump_s2c,
                    dump_file,
                ),
            ).await
            {
                (Ok(c), Ok(s)) => (c, s),
                (Err(e), _) | (_, Err(e)) => {
                    eprintln!("[{addr}] Connection error during forwarding: {e}");
                    return;
                }
            };

            println!(
                "[{addr}] Closed. Bytes relayed: client→server {from_client}, server→client {from_server}"
            );
        });
    }
}