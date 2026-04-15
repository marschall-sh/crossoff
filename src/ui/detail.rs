use chrono::{Local, NaiveDate};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{ActivePane, App};
use crate::model::label_color_rgb;
use crate::theme::Theme;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let tasks = app.tasks_for_selected_project();
    let is_active = app.active_pane == ActivePane::Detail;

    let border_color = if is_active {
        theme.border_active
    } else {
        theme.border
    };
    let title_style = if is_active {
        Style::default()
            .bg(theme.accent)
            .fg(theme.project_count_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.fg_dim)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(" Details ")
        .title_style(title_style)
        .style(Style::default().bg(theme.bg));

    let Some(task) = tasks.get(app.task_index) else {
        f.render_widget(block, area);
        return;
    };

    let today = Local::now().date_naive();
    let mut lines: Vec<Line> = Vec::new();

    // Title with checkbox
    let checkbox = if task.done {
        "\u{25cf} "
    } else {
        "\u{25cb} "
    };
    let checkbox_color = if task.done {
        theme.success
    } else {
        theme.fg_dim
    };
    let title_text_style = if task.done {
        Style::default()
            .fg(theme.fg_dim)
            .add_modifier(Modifier::CROSSED_OUT)
    } else {
        Style::default()
            .fg(theme.fg)
            .add_modifier(Modifier::BOLD)
    };
    let mut title_spans = vec![
        Span::styled("  ", Style::default()),
        Span::styled(checkbox, Style::default().fg(checkbox_color)),
    ];
    if task.pinned && !task.done {
        title_spans.push(Span::styled(
            "\u{2195} ",
            Style::default().fg(theme.accent),
        ));
    }
    title_spans.push(Span::styled(task.title.as_str(), title_text_style));
    lines.push(Line::from(title_spans));

    // Labels
    if !task.label_ids.is_empty() {
        let mut label_spans = vec![Span::styled("  ", Style::default())];
        for (i, label_id) in task.label_ids.iter().enumerate() {
            if let Some(label) = app.data.labels.iter().find(|l| l.id == *label_id) {
                if i > 0 {
                    label_spans.push(Span::raw(" "));
                }
                let (r, g, b) = label_color_rgb(&label.color);
                let bg = Color::Rgb(r, g, b);
                let fg = if (r as u16 + g as u16 + b as u16) > 384 {
                    Color::Rgb(0x1a, 0x1a, 0x1a)
                } else {
                    Color::Rgb(0xf0, 0xf0, 0xf0)
                };
                label_spans.push(Span::styled(
                    format!(" {} ", label.name),
                    Style::default().bg(bg).fg(fg),
                ));
            }
        }
        if label_spans.len() > 1 {
            lines.push(Line::from(label_spans));
        }
    }

    // Due date
    if let Some(due) = task.due_date {
        let (friendly, color) = format_due_detail(due, today, theme);
        let full_date = due.format("%b %d, %Y").to_string();
        lines.push(Line::from(vec![
            Span::styled("  Due: ", Style::default().fg(theme.detail_label)),
            Span::styled(friendly, Style::default().fg(color)),
            Span::styled(" \u{00b7} ", Style::default().fg(theme.fg_dim)),
            Span::styled(full_date, Style::default().fg(theme.fg_dim)),
        ]));
    }

    // Description — rendered line by line, respecting newlines, * and - as bullets
    if !task.description.is_empty() {
        lines.push(Line::from(""));
        for text_line in task.description.split('\n') {
            let trimmed = text_line.trim_start();
            if trimmed.starts_with("* ") || trimmed.starts_with("- ") {
                let indent = text_line.len() - trimmed.len();
                let content = &trimmed[2..];
                let padding = " ".repeat(indent + 2);
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}\u{2022} ", padding),
                        Style::default().fg(theme.accent),
                    ),
                    Span::styled(content.to_string(), Style::default().fg(theme.fg)),
                ]));
            } else if text_line.is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(Span::styled(
                    format!("  {}", text_line),
                    Style::default().fg(theme.fg),
                )));
            }
        }
    }

    // Checklist
    if !task.checklist.is_empty() {
        lines.push(Line::from(""));
        let done_count = task.checklist.iter().filter(|i| i.done).count();
        let total = task.checklist.len();
        lines.push(Line::from(vec![
            Span::styled("  Checklist ", Style::default().fg(theme.detail_label)),
            Span::styled(
                format!("{}/{}", done_count, total),
                Style::default().fg(theme.fg_dim),
            ),
        ]));

        for item in &task.checklist {
            let check = if item.done {
                "\u{2611} "
            } else {
                "\u{2610} "
            };
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
                Span::styled("  ", Style::default()),
                Span::styled(check, Style::default().fg(check_color)),
                Span::styled(item.text.as_str(), text_style),
            ]));
        }
    }

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));
    f.render_widget(para, area);
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
