use chrono::{Local, NaiveDate};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{format_duration_compact, ActivePane, App};
use crate::model::label_color_rgb;
use crate::theme::Theme;

use super::centered_rect;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;

    // Keep details focused, but not as a harsh edge-to-edge fullscreen pane.
    let bg = Block::default().style(Style::default().bg(theme.bg));
    f.render_widget(bg, area);

    let width = (area.width * 78 / 100).max(64).min(108);
    let height = (area.height * 84 / 100).max(20).min(42);
    let panel = centered_rect(width, height, area);

    let is_active = app.active_pane == ActivePane::Detail;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if is_active {
            theme.border_active
        } else {
            theme.border
        }))
        .title(" Task Details ")
        .title_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .padding(Padding::new(3, 3, 1, 1))
        .style(Style::default().bg(theme.header_bg));

    let Some(task) = app.selected_task() else {
        f.render_widget(block, panel);
        return;
    };

    let today = Local::now().date_naive();
    let mut lines: Vec<Line> = Vec::new();

    // Header: status + priority + title. Leave breathing room below it.
    let status = if task.done { "Done" } else { "Open" };
    let status_color = if task.done {
        theme.success
    } else {
        theme.accent
    };
    let title_style = if task.done {
        Style::default()
            .fg(theme.fg_dim)
            .add_modifier(Modifier::BOLD | Modifier::CROSSED_OUT)
    } else {
        Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)
    };

    let mut title_spans = vec![
        Span::styled(
            format!(" {} ", status),
            Style::default().fg(theme.project_count_fg).bg(status_color),
        ),
        Span::raw("  "),
    ];
    if task.pinned && !task.done {
        title_spans.push(Span::styled("★ ", Style::default().fg(theme.warning)));
    }
    title_spans.push(Span::styled(task.title.clone(), title_style));
    lines.push(Line::from(title_spans));
    lines.push(Line::from(""));

    // Metadata chips.
    let mut meta_spans: Vec<Span> = Vec::new();
    if let Some(due) = task.due_date {
        let (friendly, color) = format_due_detail(due, today, theme);
        meta_spans.push(Span::styled(
            " Due ",
            Style::default().fg(theme.project_count_fg).bg(color),
        ));
        meta_spans.push(Span::raw(" "));
        meta_spans.push(Span::styled(
            due.format("%b %d, %Y").to_string(),
            Style::default().fg(theme.fg_dim),
        ));
        meta_spans.push(Span::styled(
            format!(" · {}", friendly),
            Style::default().fg(color),
        ));
        meta_spans.push(Span::raw("   "));
    }

    let tracked_seconds = app.task_tracked_seconds(task.id);
    if tracked_seconds > 0 || app.is_task_running(task.id) {
        let running = app.is_task_running(task.id);
        meta_spans.push(Span::styled(
            if running {
                " ● Tracking "
            } else {
                " ◷ Tracked "
            },
            Style::default().fg(theme.accent).bg(if running {
                theme.bg_running
            } else {
                theme.border
            }),
        ));
        meta_spans.push(Span::raw(" "));
        meta_spans.push(Span::styled(
            format_duration_compact(tracked_seconds),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if !meta_spans.is_empty() {
        lines.push(Line::from(meta_spans));
        lines.push(Line::from(""));
    }

    if !task.label_ids.is_empty() {
        let mut label_spans = vec![Span::styled("Labels  ", section_label_style(theme))];
        for label_id in &task.label_ids {
            if let Some(label) = app.data.labels.iter().find(|l| l.id == *label_id) {
                let (r, g, b) = label_color_rgb(&label.color);
                let bg = Color::Rgb(r, g, b);
                let fg = readable_fg(r, g, b);
                label_spans.push(Span::styled(
                    format!(" {} ", label.name),
                    Style::default().bg(bg).fg(fg),
                ));
                label_spans.push(Span::raw(" "));
            }
        }
        lines.push(Line::from(label_spans));
        lines.push(Line::from(""));
    }

    if !task.description.is_empty() {
        push_section_header(&mut lines, "Description", theme);
        for text_line in task.description.split('\n') {
            let trimmed = text_line.trim_start();
            if trimmed.starts_with("* ") || trimmed.starts_with("- ") {
                let indent = text_line.len() - trimmed.len();
                let content = &trimmed[2..];
                let padding = " ".repeat(indent + 1);
                lines.push(Line::from(vec![
                    Span::styled(format!("{}• ", padding), Style::default().fg(theme.accent)),
                    Span::styled(content.to_string(), Style::default().fg(theme.fg)),
                ]));
            } else if text_line.is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(Span::styled(
                    text_line.to_string(),
                    Style::default().fg(theme.fg),
                )));
            }
        }
        lines.push(Line::from(""));
    }

    if !task.checklist.is_empty() {
        let done_count = task.checklist.iter().filter(|i| i.done).count();
        let total = task.checklist.len();
        push_section_header(
            &mut lines,
            &format!("Checklist {}/{}", done_count, total),
            theme,
        );
        for item in &task.checklist {
            let check = if item.done { "☑ " } else { "☐ " };
            let check_color = if item.done {
                theme.success
            } else {
                theme.fg_dim
            };
            let text_style = if item.done {
                Style::default()
                    .fg(theme.fg_dim)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().fg(theme.fg)
            };
            lines.push(Line::from(vec![
                Span::styled(check, Style::default().fg(check_color)),
                Span::styled(item.text.clone(), text_style),
            ]));
        }
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![
        Span::styled(
            "e",
            Style::default()
                .fg(theme.key_hint)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" edit  ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            "q/Esc",
            Style::default()
                .fg(theme.key_hint)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" back  ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            "↑↓",
            Style::default()
                .fg(theme.key_hint)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" scroll", Style::default().fg(theme.fg_dim)),
    ]));

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));
    f.render_widget(para, panel);
}

fn push_section_header(lines: &mut Vec<Line>, title: &str, theme: &Theme) {
    lines.push(Line::from(vec![
        Span::styled("▍ ", Style::default().fg(theme.accent)),
        Span::styled(title.to_string(), section_label_style(theme)),
    ]));
    lines.push(Line::from(""));
}

fn section_label_style(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.detail_label)
        .add_modifier(Modifier::BOLD)
}

fn readable_fg(r: u8, g: u8, b: u8) -> Color {
    if (r as u16 + g as u16 + b as u16) > 384 {
        Color::Rgb(0x1a, 0x1a, 0x1a)
    } else {
        Color::Rgb(0xf0, 0xf0, 0xf0)
    }
}

fn format_due_detail(due: NaiveDate, today: NaiveDate, theme: &Theme) -> (String, Color) {
    let days = (due - today).num_days();
    if days < -1 {
        ("Overdue".to_string(), theme.error)
    } else if days == -1 {
        ("Yesterday".to_string(), theme.error)
    } else if days == 0 {
        ("Today".to_string(), theme.warning)
    } else if days == 1 {
        ("Tomorrow".to_string(), theme.warning)
    } else if days < 7 {
        (due.format("%A").to_string(), theme.accent)
    } else {
        (format!("In {} days", days), theme.fg_dim)
    }
}
