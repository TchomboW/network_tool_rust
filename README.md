# Network Diagnostic Tool (Rust Rewrite)

A high-performance network diagnostic toolkit rewritten from [TchomboW/Network-Diagnostic-tools](https://github.com/TchomboW/Network-Diagnostic-tools). This project converts the original Go implementation to idiomatic Rust, leveraging async runtimes and modern terminal UI frameworks.

## Features

- **ICMP Ping** — Full ping statistics (transmit/receive/loss, min/avg/max latency) via `surge-ping`
- **DNS Resolution** — Fast DNS lookups with TTL-based caching via `hickory-resolver`
- **TCP Latency** — Port connectivity testing (e.g., HTTPS on 443) with millisecond precision
- **HTTP TTFB** — Time-to-first-byte measurement via `reqwest` async HTTP client
- **TUI Dashboard** — Real-time terminal dashboard using Ratatui + Crossterm, press `q` to quit
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
```

## Project Structure

```
src/
├── main.rs          — CLI entry point (clap argument parsing)
├── models.rs        — Data structures: PingStats, DnsResult, TcpResult, HttpResult
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

| Crate | Purpose |
|-------|---------|
| `surge-ping` | ICMP ping implementation |
| `hickory-resolver` | DNS resolution (successor to trust-dns) |
| `reqwest` | Async HTTP client for TTFB measurement |
| `ratatui` + `crossterm` | Terminal UI framework |
| `clap` | Command-line argument parsing (derive API) |
| `tokio` | Async runtime with multi-thread scheduler |
| `anyhow` | Error handling with context |

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
