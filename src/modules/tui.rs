use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::EnterAlternateScreen;
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use crossterm::terminal::LeaveAlternateScreen;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

use crate::models::*;

pub async fn run_tui(target: &str, interval_secs: u64) -> Result<(), anyhow::Error> {
    // Setup terminal
    let mut stdout = std::io::stdout();

    crossterm::terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut run_number: u32 = 0;
    let mut results: Vec<DiagnosticResult> = Vec::new();

    loop {
        run_number += 1;

        // Run diagnostics concurrently
        let ping_fut = super::icmp::ping(target, 4);
        let dns_fut = super::dns::resolve_dns(target);
        let tcp_fut = super::tcp::tcp_connect(target, 443);
        let http_fut = super::http::check_http(target);

        let (ping, dns, tcp, http) = tokio::join!(ping_fut, dns_fut, tcp_fut, http_fut);

        let result = DiagnosticResult {
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            run_number,
            ping: ping.ok(),
            dns: dns.ok(),
            tcp: tcp.ok(),
            http: http.ok(),
        };

        results.push(result.clone());
        if results.len() > 20 {
            results.remove(0);
        }

        // Render UI
        terminal.draw(|frame| {
            let chunks = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(10),
            ]).split(frame.area());

            // Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled(" Network Diagnostic Tool ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("Target: {}", target), Style::default().fg(Color::White)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Header "));

            // Content
            let content = build_tui_content(&result);
            let content_widget = Paragraph::new(content).block(
                Block::default().borders(Borders::ALL).title(" Diagnostics ")
            );

            frame.render_widget(header, chunks[0]);
            frame.render_widget(content_widget, chunks[1]);
        })?;

        // Check for quit key
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Wait for next interval
        tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
    }

    // Cleanup terminal
    drop(terminal);
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn build_tui_content(result: &DiagnosticResult) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Run info
    lines.push(Line::from(Span::styled(
        format!("Run #{} at {}", result.run_number, result.timestamp),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    lines.push(Line::from(""));

    // Ping results
    if let Some(ping) = &result.ping {
        lines.push(Line::from(Span::styled(
            "[PING]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        )));

        if ping.received > 0 {
            lines.push(Line::from(format!(
                "  Packets: {}/{} transmitted, {:.1}% loss",
                ping.received, ping.transmitted, ping.loss_percent
            )));

            if let Some(avg) = ping.avg_latency_ms {
                lines.push(Line::from(format!(
                    "  Latency: min={:.1}ms avg={:.1}ms max={:.1}ms",
                    ping.min_latency_ms.unwrap_or(0.0),
                    avg,
                    ping.max_latency_ms.unwrap_or(0.0)
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "  Latency: N/A (all packets lost)", Style::default().fg(Color::Red)
                )));
            }
        } else {
            lines.push(Line::from(Span::styled(
                "  All packets lost", Style::default().fg(Color::Red)
            )));
        }

        lines.push(Line::from(""));
    }

    // DNS results
    if let Some(dns) = &result.dns {
        lines.push(Line::from(Span::styled(
            "[DNS]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        )));

        if !dns.ips.is_empty() {
            lines.push(Line::from(format!(
                "  Time: {:.1}ms", dns.resolve_time_ms
            )));
            lines.push(Line::from(format!(
                "  IPs: {}", dns.ips.join(", ")
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("  Failed to resolve {}: {:.1}ms", dns.hostname, dns.resolve_time_ms),
                Style::default().fg(Color::Red)
            )));
        }

        lines.push(Line::from(""));
    }

    // TCP results
    if let Some(tcp) = &result.tcp {
        lines.push(Line::from(Span::styled(
            "[TCP]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        )));

        let status = if tcp.success {
            Span::styled(format!("OK ({:.1}ms)", tcp.duration_ms), Style::default().fg(Color::Green))
        } else {
            Span::styled(format!("FAIL ({:.1}ms)", tcp.duration_ms), Style::default().fg(Color::Red))
        };

        lines.push(Line::from(format!(
            "  Port {} (HTTPS): {}", tcp.port, status
        )));

        lines.push(Line::from(""));
    }

    // HTTP results
    if let Some(http) = &result.http {
        lines.push(Line::from(Span::styled(
            "[HTTP]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        )));

        let status_style = match http.status_code {
            Some(code) if code >= 200 && code < 300 => Style::default().fg(Color::Green),
            Some(code) if code >= 400 => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::Yellow),
        };

        lines.push(Line::from(Span::styled(
            format!("  {}: {:.1}ms", http.status_text, http.ttfb_ms),
            status_style
        )));

        lines.push(Line::from(""));
    }

    // Hint
    lines.push(Line::from(Span::styled(
        "[Press 'q' to quit]", Style::default().fg(Color::DarkGray)
    )));

    lines
}
