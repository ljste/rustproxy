use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, split};
use std::sync::Arc;
use anyhow::Result;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use futures::future::join;
use std::sync::atomic::{AtomicU64, Ordering};

mod hexdump;

use hexdump::{HexdumpConfig, Color};

/// rustproxy - enhanced async TCP proxy with logging
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

    /// Disable prefix (direction info) in hex dump
    #[arg(long)]
    no_prefix: bool,

    /// Limit dump file size in MB (approx), if set
    #[arg(long)]
    dump_file_max_mb: Option<u64>,

    /// Disable timestamps in hex dump
    #[arg(long)]
    no_timestamp: bool,
}

struct GlobalStats {
    client_to_server: AtomicU64,
    server_to_client: AtomicU64,
}

impl GlobalStats {
    fn new() -> Self {
        Self {
            client_to_server: AtomicU64::new(0),
            server_to_client: AtomicU64::new(0),
        }
    }
    fn inc_c2s(&self, n: u64) {
        self.client_to_server.fetch_add(n, Ordering::Relaxed);
    }
    fn inc_s2c(&self, n: u64) {
        self.server_to_client.fetch_add(n, Ordering::Relaxed);
    }
    fn summarize(&self) {
        let c2s = self.client_to_server.load(Ordering::Relaxed);
        let s2c = self.server_to_client.load(Ordering::Relaxed);
        println!("\n=== Proxy totals: client→server {} bytes, server→client {} bytes ===", c2s, s2c);
    }
}

async fn check_truncate_file(
    file: &tokio::fs::File,
    max_size_bytes: u64
) -> Result<()> {
    let metadata = file.metadata().await?;
    if metadata.len() > max_size_bytes {
        // Truncate by reopening in write-truncate mode
        let fd = file.try_clone().await?;
        fd.set_len(0).await?;
        println!("[*] Dump file exceeded {max_size_bytes} bytes, truncated.");
    }
    Ok(())
}

async fn copy_and_log<'a, R, W>(
    direction_conf: HexdumpConfig,
    reader: &'a mut R,
    writer: &'a mut W,
    dump: bool,
    dump_file: Option<Arc<tokio::sync::Mutex<tokio::fs::File>>>,
    dump_file_max: Option<u64>,
    global_stats: Arc<GlobalStats>
) -> Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = [0u8; 8192];
    let mut total = 0;
    let mut offset = 0;

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        if dump {
            let dump_text = hexdump::format_hex_dump(
                &buf[..n],
                offset,
                &direction_conf
            );
            print!("{dump_text}");

            if let Some(file) = &dump_file {
                let mut file = file.lock().await;
                if let Some(max_bytes) = dump_file_max {
                    check_truncate_file(&*file, max_bytes).await.ok();
                }
                file.write_all(dump_text.as_bytes()).await?;
            }
        }
        writer.write_all(&buf[..n]).await?;
        total += n as u64;
        offset += n;

        if direction_conf.prefix.contains("CLIENT") {
            global_stats.inc_c2s(n as u64);
        } else {
            global_stats.inc_s2c(n as u64);
        }
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
    let dump_file = if let Some(path) = args.dump_file.clone() {
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
    let dump_file_max_bytes = args.dump_file_max_mb.map(|mb| mb * 1024 * 1024);

    let global_stats = Arc::new(GlobalStats::new());

    // spawn graceful shutdown handler for Ctrl+C
    {
        let global_stats = global_stats.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            println!("\n=== Proxy interrupted by user ===");
            global_stats.summarize();
            std::process::exit(0);
        });
    }

    loop {
        let (inbound, addr) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[!] Failed to accept connection: {e}");
                continue;
            }
        };
        println!("\n========== Connection from {addr} to {} STARTED ==========", args.target);

        let target = args.target.clone();
        let dump_file = dump_file.clone();
        let dump_file_max_bytes = dump_file_max_bytes.clone();
        let dump_c2s = args.dump_c2s;
        let dump_s2c = args.dump_s2c;
        let global_stats = global_stats.clone();

        let show_prefix = !args.no_prefix;
        let show_timestamps = !args.no_timestamp;

        tokio::spawn(async move {
            println!("[{addr}] Connecting to target {target}...");

            let outbound = match TcpStream::connect(&target).await {
                Ok(s) => {
                    println!("[{addr}] Connected to target");
                    s
                },
                Err(e) => {
                    eprintln!("[{addr}] Failed to connect to target {target}: {e}");
                    println!("========== Connection from {addr} to {target} CLOSED ==========");
                    return;
                }
            };

            let (mut inbound_read, mut inbound_write) = split(inbound);
            let (mut outbound_read, mut outbound_write) = split(outbound);

            let c2s_conf = HexdumpConfig {
                prefix: format!("[CLIENT \u{2192} SERVER] "), // nice arrow
                color: Some(Color::Blue),
                show_prefix,
                timestamp: show_timestamps,
                show_end_offset: true,
            };
            let s2c_conf = HexdumpConfig {
                prefix: format!("[SERVER \u{2190} CLIENT] "),
                color: Some(Color::Green),
                show_prefix,
                timestamp: show_timestamps,
                show_end_offset: true,
            };

            let (from_client, from_server) = match join(
                copy_and_log(
                    c2s_conf,
                    &mut inbound_read,
                    &mut outbound_write,
                    dump_c2s,
                    dump_file.clone(),
                    dump_file_max_bytes,
                    global_stats.clone(),
                ),
                copy_and_log(
                    s2c_conf,
                    &mut outbound_read,
                    &mut inbound_write,
                    dump_s2c,
                    dump_file,
                    dump_file_max_bytes,
                    global_stats.clone(),
                ),
            ).await
            {
                (Ok(c), Ok(s)) => (c, s),
                (Err(e), _) | (_, Err(e)) => {
                    eprintln!("[{addr}] Connection error during forwarding: {e}");
                    println!("========== Connection from {addr} to {target} CLOSED ==========");
                    return;
                }
            };

            println!(
                "[{addr}] Closed. Bytes relayed: client→server {}, server→client {}",
                from_client, from_server
            );
            println!("========== Connection from {addr} to {target} CLOSED ==========");
        });
    }
}
