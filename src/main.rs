mod models;
pub mod modules {
    pub mod icmp;
    pub mod dns;
    pub mod tcp;
    pub mod http;
    pub mod tui;
}
mod utils {
    pub mod dns_cache;
    pub mod retry_middleware;
}
mod web_ui;

use clap::Parser;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "network_tool", version, about = "Network Diagnostic Tool (Rust rewrite)")]
struct Args {
    /// Target host to monitor (IP, hostname, or URL)
    #[arg(short, long, default_value = "8.8.8.8")]
    target: String,

    /// Monitoring interval in seconds
    #[arg(short, long, default_value_t = 5)]
    interval: u64,

    /// Run for duration in seconds then exit (0 = infinite)
    #[arg(short, long, default_value_t = 0)]
    duration: u64,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Run TUI mode instead of CLI
    #[arg(short, long)]
    tui: bool,

    /// Web server address to start (e.g., :8080)
    #[arg(long)]
    web: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    if let Some(web_addr) = &args.web {
        println!("Starting web server at {}", web_addr);
        let app = web_ui::create_router();
        let listener = tokio::net::TcpListener::bind(web_addr).await?;
        axum::serve(listener, app).await?;
        return Ok(());
    }

    if args.tui {
        println!("Starting TUI mode...");
        return modules::tui::run_tui(&args.target, args.interval).await;
    }

    println!("=== Network Diagnostic Tool (Rust) ===");
    println!("Target:    {}", args.target);
    println!("Interval:  {}s", args.interval);

    let start_time = std::time::Instant::now();
    let mut run_number: u32 = 0;

    loop {
        // Check duration limit
        if args.duration > 0 && start_time.elapsed() >= Duration::from_secs(args.duration) {
            println!("\nDuration {}s reached. Exiting.", args.duration);
            break;
        }

        run_number += 1;
        println!("\n--- Run #{} ({}) ---", run_number, chrono::Local::now().format("%H:%M:%S"));

        // Run diagnostics concurrently
        let ping_fut = modules::icmp::ping(&args.target, 4);
        let dns_fut = modules::dns::resolve_dns(&args.target);
        let tcp_fut = modules::tcp::tcp_connect(&args.target, 443);
        let http_fut = modules::http::check_http(&args.target);

        let (ping, dns, tcp, http) = tokio::join!(ping_fut, dns_fut, tcp_fut, http_fut);

        // Print ping results
        match &ping {
            Ok(ping) => {
                println!("\n[PING] Testing connectivity...");
                if ping.received > 0 {
                    println!(
                        "  Packets: {}/{} transmitted, {:.1}% loss",
                        ping.received, ping.transmitted, ping.loss_percent
                    );
                    if let Some(avg) = ping.avg_latency_ms {
                        println!(
                            "  Latency: min={:.1}ms avg={:.1}ms max={:.1}ms",
                            ping.min_latency_ms.unwrap_or(0.0),
                            avg,
                            ping.max_latency_ms.unwrap_or(0.0)
                        );
                    } else {
                        println!("  Latency: N/A (all packets lost)");
                    }
                } else {
                    println!("  All {} packets lost", ping.transmitted);
                }
            }
            Err(e) => {
                println!("\n[PING] Error: {}", e);
            }
        }

        // Print DNS results
        match &dns {
            Ok(dns) => {
                println!("\n[DNS] Resolving hostname...");
                if !dns.ips.is_empty() {
                    println!("  Time: {:.1}ms", dns.resolve_time_ms);
                    println!("  IPs: {}", dns.ips.join(", "));
                } else {
                    println!(
                        "[DNS] Failed to resolve {}: {:.1}ms",
                        dns.hostname, dns.resolve_time_ms
                    );
                }
            }
            Err(e) => {
                println!("\n[DNS] Error: {}", e);
            }
        }

        // Print TCP results
        match &tcp {
            Ok(tcp) => {
                println!("\n[TCP] Testing port connectivity...");
                let status = if tcp.success { "OK" } else { "FAIL" };
                println!(
                    "  Port {} (HTTPS): {} ({:.1}ms)",
                    tcp.port, status, tcp.duration_ms
                );
            }
            Err(e) => {
                println!("\n[TCP] Error: {}", e);
            }
        }

        // Print HTTP results
        match &http {
            Ok(http) => {
                println!("\n[HTTP] Checking HTTP response...");
                println!("  {}: {:.1}ms", http.status_text, http.ttfb_ms);
            }
            Err(e) => {
                println!("\n[HTTP] Error: {}", e);
            }
        }

        // Verbose output
        if args.verbose {
            println!("\n[VERBOSE] Details:");
            println!("  OS: {}", std::env::consts::OS);
            println!("  Arch: {}", std::env::consts::ARCH);
        }

        // Wait for next interval or signal
        tokio::time::sleep(Duration::from_secs(args.interval)).await;
    }

    Ok(())
}
