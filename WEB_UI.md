# Web UI — API-First Diagnostics Server

## Overview

The web UI provides a browser-accessible diagnostics server built on **Axum** (the ergonomic Rust web framework). It exposes a JSON API that can be consumed by any frontend or tooling, enabling remote network diagnostics without terminal access.

## Architecture

```
┌─────────────┐     HTTP/JSON      ┌──────────────┐
│  Browser    │ ◄─────────────────► │   Axum Router│
│  / Client   │                    │              │
└─────────────┘                    │  /api/       │
                                   │   diagnostics│
                                   └──────┬───────┘
                                          │
                              Placeholder handler (TODO)
```

**Current state**: The diagnostics endpoint returns static/placeholder JSON. Integration with the actual diagnostic modules (ICMP, DNS, TCP, HTTP) is pending — see "Future Work" below.

## Launch

```bash
# Start web server on default port 3000
./target/release/network_tool --web :3000

# Custom port
./target/release/network_tool --web 127.0.0.1:8080

# Custom host and port
./target/release/network_tool --web 0.0.0.0:3001
```

### Arguments

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--web` | String (optional) | N/A | Bind address in `<host>:<port>` format. If omitted, CLI mode runs instead. |

### Mode Priority

1. `--web` flag → starts web server, exits immediately
2. `--tui` flag → starts terminal dashboard
3. Default → runs CLI diagnostics loop

## API Endpoints

### `GET /api/diagnostics`

Returns a JSON snapshot of system diagnostics.

**Response:**

```json
{
  "status": "ok",
  "timestamp": "2026-05-23T11:44:00.000Z",
  "checks": {
    "api": "healthy",
    "database": "connected"
  }
}
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `status` | String | Overall system status (`ok`, `degraded`, `error`) |
| `timestamp` | String (RFC3339) | Server-side timestamp of the diagnostics snapshot |
| `checks` | Object | Individual health check results (key-value pairs) |

**Example curl:**

```bash
curl http://localhost:3000/api/diagnostics | jq .
```

## Implementation Details

### Module: `src/web_ui.rs`

| Component | Type/Role |
|-----------|-----------|
| `create_router()` | Returns an Axum `Router` mounted on `/api/diagnostics` |
| `diagnostics_handler()` | Async handler returning JSON via `axum::response::Json` |

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `axum` | 0.7 | Web server framework (routing, handlers) |
| `serde_json` | 1 | JSON serialization for responses |

### Code Structure

```rust
// Router setup — single endpoint mounted at /api/diagnostics
pub fn create_router() -> Router {
    Router::new()
        .route("/api/diagnostics", get(diagnostics_handler))
}

// Handler — returns static JSON payload
pub async fn diagnostics_handler() -> Json<Value> {
    let diagnostics = json!({ ... });
    Json(diagnostics)
}
```

## Current Limitations

1. **Placeholder data** — The diagnostics handler returns static JSON, not real network diagnostic results from the ICMP/DNS/TCP/HTTP modules.
2. **Single endpoint** — Only `/api/diagnostics` is implemented; no dedicated endpoints per diagnostic type.
3. **No authentication** — The API has no auth guard; it's intended for local/Trusted network use only.
4. **No static frontend** — The server serves JSON API responses only; no HTML/JS frontend files are served.
5. **No CORS headers** — Cross-origin requests from external frontends are not configured.

## Future Work

### Phase 1 — Real Diagnostics Integration

Replace the placeholder handler with actual diagnostic calls:

```rust
pub async fn diagnostics_handler() -> Json<Value> {
    let target = /* configurable via query param or global */;

    // Run diagnostics concurrently (mirrors main.rs CLI logic)
    let (ping, dns, tcp, http) = tokio::join!(
        modules::icmp::ping(&target, 4),
        modules::dns::resolve_dns(&target),
        modules::tcp::tcp_connect(&target, 443),
        modules::http::check_http(&target)
    );

    // Aggregate results into JSON payload
    Json(json!({ ... }))
}
```

### Phase 2 — Enhanced API

| Feature | Description | Priority |
|---------|-------------|----------|
| Per-type endpoints | `GET /api/ping`, `GET /api/dns`, etc. | High |
| Query parameters | `?target=example.com&interval=10` on API calls | High |
| Streaming/WS | WebSocket endpoint for real-time streaming diagnostics | Medium |
| Authentication | API key or basic auth guard on all endpoints | Medium |

### Phase 3 — Static Frontend

Serve a minimal HTML/JS dashboard that:
- Calls `/api/diagnostics` on load and polling interval
- Renders results in a browser-friendly layout (tables, charts)
- Allows target/interval configuration via UI

### Phase 4 — Production Hardening

| Concern | Action |
|---------|--------|
| CORS headers | Add `tower-http` CORS middleware for cross-origin frontend access |
| Request timeout | Set per-request timeouts to prevent hanging diagnostics |
| Structured logging | Add tracing/layer for request/response logging |
| Graceful shutdown | Signal handling (SIGTERM) to stop server cleanly |

## Relationship to Other Modes

| Mode | Entry Point | Use Case |
|------|-------------|----------|
| CLI (`./network_tool target`) | `main()` → diagnostics loop | Quick terminal-based checks, scripts, CI/CD |
| TUI (`./network_tool --tui target`) | `modules::tui::run_tui()` | Real-time terminal dashboard, ops monitoring |
| Web (`./network_tool --web :3000`) | `web_ui::create_router()` → Axum serve | Remote access, API integration, browser dashboard (future) |

## Notes

- The web server runs on a single thread via `tokio::main` — fine for API-only use, but consider multi-thread runtime features (`--features full`) if adding CPU-heavy static file serving.
- The `axum` + `tower-http` stack is already in `Cargo.toml`, providing a foundation for middleware (CORS, compression, rate limiting) when Phase 4 hardening begins.
