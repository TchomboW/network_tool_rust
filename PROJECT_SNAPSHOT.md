# Network Diagnostic Tool - Rust Project Snapshot

## Project Structure (2026-05-29)
```
network_tool_rust/
├── Cargo.toml          (deps: surge-ping, hickory-resolver, reqwest, ratatui, crossterm, clap, tokio, anyhow, axum, serde)
├── Cargo.lock
└── src/
    ├── main.rs         (CLI/Web entry, module exports)
    ├── models.rs       (PingStats, DnsResult, TcpResult, HttpResult, DiagnosticResult)
    ├── web_ui.rs       (Axum web server with /api/diagnostics endpoint)
    ├── modules/
    │   ├── icmp.rs     (ICMP ping via surge-ping)
    │   ├── dns.rs      (DNS resolution via hickory-resolver)
    │   ├── tcp.rs      (TCP connect latency measurement)
    │   ├── http.rs     (HTTP TTFB via reqwest)
    │   └── tui.rs      (Ratatui CROSSTERM terminal UI)
    └── utils/
        ├── dns_cache.rs     (DNS result caching with TTL eviction)
        └── retry_middleware.rs  (Retry with exponential backoff)
```

## Build Status
- `cargo check`: ✅ PASS (0 errors, 9 warnings - unused imports/dead code only)
- Edition: Rust 2021

## Key Dependencies vs Go Original
|| Functionality | Go Library | Rust Crate |
|--------------|-----------|------------|
|| ICMP Ping | `github.com/go-ping/ping` | `surge-ping 0.8.4` |
|| DNS Resolution | `github.com/miekg/dns` + `net.LookupIP` | `hickory-resolver 0.24.x` |
|| TCP Latency | `net.DialTimeout` | `tokio::net::TcpStream::connect` |
|| HTTP TTFB | `http.Client.Get` + time.Now() | `reqwest::Client.get().send().await.elapsed()` |
|| TUI | `github.com/charmbracelet/bubbles` + `lipgloss` | `ratatui 0.26+` + `crossterm 0.27+` |
|| CLI Parser | `github.com/urfave/cli/v2` | `clap 4.x` |
|| Async Runtime | Go goroutines + channels | `tokio 1.x` |
|| Web Server | N/A | `axum 0.7` + `tower-http` |
