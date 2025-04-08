# rustproxy

**rustproxy** is a simple, async TCP proxy written in Rust.

It listens on a specified local address, accepts incoming TCP connections, and transparently forwards all data to a target server — logging connection details and traffic metrics.

---

## Features

- Lightweight configurable TCP proxy  
- Logs connection attempts, successes, failures, and traffic counts  
- Built with asynchronous I/O (Tokio) for efficiency

---

## Usage

```bash
cargo run -- --listen <LISTEN_ADDR> --target <TARGET_ADDR>
```

where:

- `<LISTEN_ADDR>` – local address and port to listen on (e.g., `127.0.0.1:1080`)  
- `<TARGET_ADDR>` – remote address and port to forward to (e.g., `example.com:80`)

---

### Example: Forward local port 1080 to example.com port 80

```bash
cargo run -- --listen 127.0.0.1:1080 --target example.com:80
```

Now, connect **through your proxy**:

```bash
curl -x 127.0.0.1:1080 http://example.com
```

Your terminal will show connection logs and bytes relayed.
