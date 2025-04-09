# rustproxy

**rustproxy** is a simple, async TCP proxy written in Rust that supports traffic logging with hex dumps.

---

## Features

- Async TCP proxy with configurable endpoints
- Hex dump logging of traffic in both directions
- Optional file-based logging
- Built with Tokio for async I/O

## Usage

```bash
cargo run -- -l <LISTEN_ADDR> -t <TARGET_ADDR> [OPTIONS]
```

### Arguments

- `-l, --listen <ADDR>` - Local address to listen on (e.g., `127.0.0.1:8080`)
- `-t, --target <ADDR>` - Remote address to forward to (e.g., `httpbin.org:80`)
- `--dump-c2s` - Enable client to server traffic hex dumps
- `--dump-s2c` - Enable server to client traffic hex dumps
- `--dump-file <FILE>` - Save traffic dumps to file (optional)

### Example: HTTP Proxy with Logging

1. Start the proxy:
```bash
cargo run -- -l 127.0.0.1:8080 -t httpbin.org:80 --dump-c2s --dump-s2c --dump-file traffic.log
```

2. Test with curl:
```bash
curl -v -H "Host: httpbin.org" http://localhost:8080/get
```

The proxy will:
- Display hex dumps of traffic in the terminal
- Save dumps to traffic.log if enabled
- Show connection status and bytes transferred

### Notes

- Currently supports HTTP proxying
- Requires explicit Host header for proper forwarding
- Does not support HTTPS (TLS) connections