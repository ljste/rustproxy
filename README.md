# rustproxy

**rustproxy** is a simple, async TCP proxy in Rust with colorful, timestamped traffic hex dumps and optional file-based logging.

---

## Features

- Async TCP proxy with configurable endpoints
- Colorized, readable, timestamped hex dump logging in both directions
- Shows line offsets and ASCII view just like `hexdump -C`
- Optional file logging, with automatic (simple) truncation if too big
- Connection banners for clarity
- Shows per-connection and global byte stat summaries
- Handles Ctrl+C cleanly – prints running totals
- Options to disable prefix or timestamps for scripting
- Built with Tokio for async I/O

---

## Usage

```bash
cargo run -- -l <LISTEN_ADDR> -t <TARGET_ADDR> [OPTIONS]
```

### Arguments

- `-l, --listen <ADDR>` – Local address to listen on (e.g., `127.0.0.1:8080`)
- `-t, --target <ADDR>` – Remote address to forward to (e.g., `httpbin.org:80`)
- `--dump-c2s` – Enable client to server traffic hex dumps
- `--dump-s2c` – Enable server to client traffic hex dumps
- `--dump-file <FILE>` – Save traffic dumps to file (optional)
- `--dump-file-max-mb <MB>` – Approximate max log file size in MB (auto-truncates if exceeded)
- `--no-prefix` – Omit direction prefix in dumps
- `--no-timestamp` – Omit timestamps from dumps

---

## Example: HTTP Proxy with Logging

```bash
cargo run -- -l 127.0.0.1:8080 -t httpbin.org:80 --dump-c2s --dump-s2c --dump-file traffic.log --dump-file-max-mb 10
```

Then test it:

```bash
curl -v -H "Host: httpbin.org" http://localhost:8080/get
```

---

## Output Example

```
========== Connection from 127.0.0.1:59460 to httpbin.org:80 STARTED ==========
[12:34:56.789] [CLIENT → SERVER] 0000  47 45 54 20 2f 67 65 74  20 48 54 54 50 2f 31 2e  |GET /get HTTP/1.|
[12:34:56.790] [CLIENT → SERVER] 0010  31 0d 0a 48 6f 73 74 3a  20 68 74 74 70 62 69 6e  |1..Host: httpbin|
...
[12:34:56.791] [SERVER ← CLIENT] 0000  48 54 54 50 2f 31 2e 31  20 32 30 30 20 4f 4b 0d  |HTTP/1.1 200 OK.|
...
[+] Closed. Bytes relayed: client→server 254, server→client 291
========== Connection from 127.0.0.1:59460 to httpbin.org:80 CLOSED ==========
```

---

## Notes

- Supports HTTP or raw TCP (unencrypted)
- HTTPS (TLS) not currently supported
- Dump file truncation is basic (zeroes file if too large)
- Color via `colored` crate, time via `chrono`
- You can disable colors/prefix/timestamps for scripting