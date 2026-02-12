use std::time::Duration;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Tabs},
    Frame,
};

use crate::app::App;

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Player tabs
        Constraint::Length(5), // Track info
        Constraint::Length(3), // Progress bar
        Constraint::Length(4), // Controls + volume
        Constraint::Length(3), // Help bar
    ])
    .split(frame.area());

    draw_player_tabs(frame, app, chunks[0]);
    draw_track_info(frame, app, chunks[1]);
    draw_progress(frame, app, chunks[2]);
    draw_controls(frame, app, chunks[3]);
    draw_help(frame, chunks[4]);
}

fn draw_player_tabs(frame: &mut Frame, app: &App, area: Rect) {
    if app.player_names.is_empty() {
        let block = Block::default()
            .title(" playerctl-tui ")
            .borders(Borders::ALL);
        let text = Paragraph::new("No players found").block(block);
        frame.render_widget(text, area);
        return;
    }
    let titles: Vec<Line> = app
        .player_names
        .iter()
        .map(|n| Line::from(n.as_str()))
        .collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(" playerctl-tui ")
                .borders(Borders::ALL),
        )
        .select(app.selected_player)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}

fn draw_track_info(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL);
    if app.player_names.is_empty() {
        let text = Paragraph::new("  Waiting for an MPRIS player...").block(block);
        frame.render_widget(text, area);
        return;
    }
    let text = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  Title:  ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.title),
        ]),
        Line::from(vec![
            Span::styled("  Artist: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.artist),
        ]),
        Line::from(vec![
            Span::styled("  Album:  ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.album),
        ]),
    ])
    .block(block);
    frame.render_widget(text, area);
}

fn draw_progress(frame: &mut Frame, app: &App, area: Rect) {
    let ratio = if app.duration.as_secs_f64() > 0.0 {
        (app.position.as_secs_f64() / app.duration.as_secs_f64()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let label = format!(
        " {} / {} ",
        format_duration(app.position),
        format_duration(app.duration)
    );
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(ratio)
        .label(label);
    frame.render_widget(gauge, area);
}

fn draw_controls(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL);

    let status_icon = match app.playback_status.as_str() {
        "Playing" => "\u{25b6}",
        "Paused" => "\u{23f8}",
        _ => "\u{25a0}",
    };

    let shuffle_str = if app.shuffle { "On" } else { "Off" };
    let vol_pct = (app.volume * 100.0).round() as u16;

    let vol_ratio = app.volume.clamp(0.0, 1.0);
    let bar_width = area.width.saturating_sub(2) as usize; // inside borders
    let filled = ((bar_width as f64) * vol_ratio).round() as usize;
    let vol_bar: String = format!(
        "{}{}",
        "=".repeat(filled.min(bar_width)),
        " ".repeat(bar_width.saturating_sub(filled))
    );

    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!("  {} {}", status_icon, app.playback_status),
                Style::default().fg(Color::Green),
            ),
            Span::raw(format!(
                "      Loop: {}    Shuffle: {}",
                app.loop_status, shuffle_str
            )),
        ]),
        Line::from(vec![
            Span::styled("  Volume: [", Style::default().fg(Color::White)),
            Span::styled(&vol_bar[..filled.min(bar_width)], Style::default().fg(Color::Magenta)),
            Span::styled(
                &vol_bar[filled.min(bar_width)..],
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("] {}%", vol_pct),
                Style::default().fg(Color::White),
            ),
        ]),
    ];
    let text = Paragraph::new(lines).block(block);
    frame.render_widget(text, area);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Cyan)),
        Span::raw(":Quit "),
        Span::styled("Space", Style::default().fg(Color::Cyan)),
        Span::raw(":Play/Pause "),
        Span::styled("n/p", Style::default().fg(Color::Cyan)),
        Span::raw(":Next/Prev "),
        Span::styled("+/-", Style::default().fg(Color::Cyan)),
        Span::raw(":Vol "),
        Span::styled("\u{2190}/\u{2192}", Style::default().fg(Color::Cyan)),
        Span::raw(":Seek "),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(":Player "),
        Span::styled("l", Style::default().fg(Color::Cyan)),
        Span::raw(":Loop "),
        Span::styled("s", Style::default().fg(Color::Cyan)),
        Span::raw(":Shuffle"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, area);
}
