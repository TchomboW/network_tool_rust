# Network Diagnostic Tool (Rust Rewrite)

A high-performance network diagnostic toolkit rewritten from [TchomboW/Network-Diagnostic-tools](https://github.com/TchomboW/Network-Diagnostic-tools). This project converts the original Go implementation to idiomatic Rust, leveraging async runtimes and modern terminal UI frameworks.

## Features

- **ICMP Ping** — Full ping statistics (transmit/receive/loss, min/avg/max latency) via `surge-ping`
- **DNS Resolution** — Fast DNS lookups with TTL-based caching via `hickory-resolver`
- **TCP Latency** — Port connectivity testing (e.g., HTTPS on 443) with millisecond precision
- **HTTP TTFB** — Time-to-first-byte measurement via `reqwest` async HTTP client
- **TUI Dashboard** — Real-time terminal dashboard using Ratatui + Crossterm, press `q` to quit
- **Web UI** — Browser-based diagnostics dashboard with real-time API via Axum
- **Async Architecture** — All diagnostics run concurrently via Tokio async runtime

## Quick Start

```bash
# Build from source
cargo build --release

# CLI mode (single run)
./target/release/network_tool ping example.com
./target/release/network_tool dns example.com
./target/release/network_tool tcp example.com 443
./target/release/network_tool http https://example.com

# TUI mode (real-time dashboard, updates every 5s)
./target/release/network_tool tui example.com --interval 5

# Web UI mode (starts HTTP server at :3000)
./target/release/network_tool --web :3000

# Access in browser: http://localhost:3000
# API endpoint: GET /api/diagnostics (returns JSON diagnostics)
```

## Project Structure

```
src/
├── main.rs          — CLI/Web entry point (clap argument parsing)
├── models.rs        — Data structures: PingStats, DnsResult, TcpResult, HttpResult
├── web_ui.rs        — Axum web server with /api/diagnostics endpoint
├── modules/
│   ├── icmp.rs      — ICMP ping implementation (surge-ping)
│   ├── dns.rs       — DNS resolution with cache (hickory-resolver)
│   ├── tcp.rs       — TCP connect latency measurement (tokio)
│   ├── http.rs      — HTTP TTFB check (reqwest)
│   └── tui.rs       — Ratatui terminal UI dashboard
└── utils/
    ├── dns_cache.rs        — DNS result cache with TTL eviction
    └── retry_middleware.rs — Exponential backoff retry logic
```

## Dependencies

|| Crate | Purpose |
|-------|---------|
|| `surge-ping` | ICMP ping implementation |
|| `hickory-resolver` | DNS resolution (successor to trust-dns) |
|| `reqwest` | Async HTTP client for TTFB measurement |
|| `axum` + `tower-http` | Web server and HTTP utilities |
|| `ratatui` + `crossterm` | Terminal UI framework |
|| `clap` | Command-line argument parsing (derive API) |
|| `tokio` | Async runtime with multi-thread scheduler |
|| `serde` + `serde_json` | Serialization/deserialization |
|| `anyhow` | Error handling with context |

## API Reference

### `GET /api/health`

Health check endpoint.

**Response:**
```json
{
  "status": "ok",
  "timestamp": "2026-05-30T12:00:00.000Z"
}
```

### `GET /api/diagnostics`

Runs all four network diagnostics concurrently and returns real-time results.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `target` | string | `"8.8.8.8"` | Target host (IP, hostname, or URL) |
| `ping_count` | integer | `4` | Number of ICMP packets to send |
| `tcp_port` | integer | `443` | TCP port to test connectivity on |

**Example:**
```bash
# Default diagnostics against 8.8.8.8
curl http://localhost:3000/api/diagnostics

# Custom target, 10 pings, port 80
curl "http://localhost:3000/api/diagnostics?target=google.com&ping_count=10&tcp_port=80"
```

**Response:**
```json
{
  "timestamp": "2026-05-30T12:00:00.000Z",
  "target": "8.8.8.8",
  "ping": {
    "status": "healthy",
    "data": {
      "transmitted": 4,
      "received": 4,
      "loss_percent": 0.0,
      "min_latency_ms": 12.3,
      "avg_latency_ms": 14.5,
      "max_latency_ms": 18.2
    }
  },
  "dns": {
    "status": "healthy",
    "data": {
      "hostname": "8.8.8.8",
      "resolve_time_ms": 23.1,
      "ips": ["8.8.8.8"]
    }
  },
  "tcp": {
    "status": "healthy",
    "data": {
      "port": 443,
      "success": true,
      "duration_ms": 45.6
    }
  },
  "http": {
    "status": "healthy",
    "data": {
      "url": "https://8.8.8.8",
      "status_code": 200,
      "status_text": "200 OK",
      "ttfb_ms": 123.4
    }
  }
}
```

**Status values:** `healthy`, `degraded`, `unhealthy`, `error`

## Recent Changes

### v0.1.1 — Web API wired to real diagnostics
- ✅ `/api/diagnostics` now calls actual ICMP, DNS, TCP, and HTTP modules (was returning static placeholder data)
- ✅ Query parameter support: `?target=&ping_count=&tcp_port=` for flexible API usage
- ✅ Health check endpoint: `GET /api/health` returns server status with timestamp
- ✅ Diagnostics run concurrently via `tokio::join!` — same pattern as CLI mode
- ✅ Status classification: each check returns `healthy`, `degraded`, `unhealthy`, or `error`

### v0.1.2 — TCP connect timeout fix
- ✅ Added 10-second timeout to `tcp_connect()` — prevents hanging on unreachable hosts

## Build Requirements

- **Rust 1.70+** (Edition 2021)
- macOS, Linux, or Windows

```bash
cargo check          # Verify compilation (0 errors)
cargo build --release  # Optimized release build
```

## Original Go Source

This project is a Rust rewrite of [TchomboW/Network-Diagnostic-tools](https://github.com/TchomboW/Network-Diagnostic-tools). See `GORUST_DIFF.md` for a detailed comparison of architectural differences between the Go and Rust implementations.

## License

MIT
