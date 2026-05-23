# Go vs Rust 差異總結報告

1. **執行模型**：Go 使用 goroutine + channel 實現併發診斷，Rust 改用 tokio async/await + `tokio::join!` 實現非同步併發，兩者都能有效利用 I/O 等待時間但 Rust 的 borrow checker 強制更嚴謹的所有權管理。

2. **ICMP 實現**：Go 的 `go-ping/ping` 庫提供高層級 `ping.Pinger()` API，Rust 的 `surge-ping` 則需手動構建 `Client → Pinger → ping(seq, payload)` 低層級呼叫鏈，增加了 `PingIdentifier`、`PingSequence` 等類型管理的複雜度。

3. **TUI 框架**：Go 使用 `charmbracelet/bubbles` + `lipgloss` 的組件式模型（如 `table.Model`, `list.Model`），Rust 改用 `ratatui` + `crossterm` 的函數式渲染模型（`terminal.draw(|frame| { ... })`），兩者都是終端 UI 的主流選擇但 API 設計哲學截然不同。

4. **DNS 解析**：Go 原生 `net.LookupIP()` 已內建 DNS 快取機制，Rust 的 `hickory-resolver` 需額外實作 `DnsCache` 結構體（含 TTL 過期、容量淘汰邏輯），但換來更精細的快取控制權。

5. **生態映射**：Go 的 `urfave/cli/v2` CLI 框架在 Rust 中對應 `clap 4.x`（從 `App`/`Command` 改為 `#[derive(Parser)]` derive macro），Go 的 `time.Now()`/`.Sub()` 時間計算在 Rust 中對應 `std::time::Instant::now()`/`.elapsed().as_millis()`，整體功能對等但 Rust 的類型系統在編譯期就強制處理了更多邊界條件。
