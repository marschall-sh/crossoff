use chrono::{Datelike, Local, NaiveDate};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{ActivePane, App};
use crate::model::label_color_rgb;
use crate::theme::Theme;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let is_active = app.active_pane == ActivePane::Tasks;

    let border_color = if is_active {
        theme.border_active
    } else {
        theme.border
    };
    let project_name = app
        .selected_project()
        .map(|p| p.name.as_str())
        .unwrap_or("No Project");

    let title = format!(" {} ", project_name);

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
        .title(title)
        .title_style(title_style)
        .style(Style::default().bg(theme.bg));

    let tasks = app.tasks_for_selected_project();

    if tasks.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No tasks yet. Press 'n' to create one.",
            Style::default().fg(theme.fg_dim),
        ))
        .block(block);
        f.render_widget(empty, area);
        return;
    }

    let today = Local::now().date_naive();
    let inner_width = area.width.saturating_sub(2) as usize;

    // Find the boundary between undone and done tasks
    let first_done_idx = tasks.iter().position(|t| t.done);
    let has_both = first_done_idx.is_some()
        && first_done_idx.unwrap() > 0
        && first_done_idx.unwrap() < tasks.len();

    let mut items: Vec<ListItem> = Vec::new();
    let mut separator_at: Option<usize> = None;

    for (i, task) in tasks.iter().enumerate() {
        // Insert divider before the first done task
        if has_both && Some(i) == first_done_idx {
            separator_at = Some(items.len());
            let rule_width = inner_width.saturating_sub(4);
            let rule = "\u{2500}".repeat(rule_width.min(40));
            items.push(
                ListItem::new(Line::from(Span::styled(
                    format!("  {}", rule),
                    Style::default().fg(theme.border),
                )))
                .style(Style::default()),
            );
        }

        let is_selected = i == app.task_index;
        let marker = if is_selected && is_active {
            "\u{25b8} "
        } else {
            "  "
        };
        let checkbox = if task.done {
            "\u{25cf} "
        } else {
            "\u{25cb} "
        };

        let task_title_style = if task.done {
            Style::default()
                .fg(theme.fg_dim)
                .add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default().fg(theme.fg)
        };

        let checkbox_color = if task.done { theme.success } else { theme.fg_dim };

        // Manual position indicator — only on tasks the user explicitly moved
        let prio_indicator = if task.pinned && !task.done {
            "\u{2195}" // ↕
        } else {
            " "
        };
        let prio_color = if task.pinned && !task.done {
            theme.accent
        } else {
            theme.bg
        };

        let mut spans = vec![
            Span::styled(marker, Style::default().fg(theme.cursor_marker)),
            Span::styled(checkbox, Style::default().fg(checkbox_color)),
            Span::styled(prio_indicator, Style::default().fg(prio_color)),
            Span::styled(" ", Style::default()),
            Span::styled(task.title.as_str(), task_title_style),
        ];

        // Labels as colored pills
        let mut extra_width = 2usize; // account for prio indicator + space
        for label_id in &task.label_ids {
            if let Some(label) = app.data.labels.iter().find(|l| l.id == *label_id) {
                let (r, g, b) = label_color_rgb(&label.color);
                let bg = Color::Rgb(r, g, b);
                let fg = if (r as u16 + g as u16 + b as u16) > 384 {
                    Color::Rgb(0x1a, 0x1a, 0x1a)
                } else {
                    Color::Rgb(0xf0, 0xf0, 0xf0)
                };
                spans.push(Span::raw(" "));
                let pill = format!(" {} ", label.name);
                extra_width += 1 + pill.len();
                spans.push(Span::styled(pill, Style::default().bg(bg).fg(fg)));
            }
        }

        // Checklist progress
        if !task.checklist.is_empty() {
            let done_count = task.checklist.iter().filter(|c| c.done).count();
            let total = task.checklist.len();
            let progress = format!(" \u{2713}{}/{}", done_count, total);
            let color = if done_count == total {
                theme.success
            } else {
                theme.fg_dim
            };
            extra_width += progress.len();
            spans.push(Span::styled(progress, Style::default().fg(color)));
        }

        // Due date (right-aligned)
        if let Some(due) = task.due_date {
            let (date_str, date_color) = format_due_date(due, today, theme);
            let used = 2 + 2 + task.title.len() + extra_width;
            let date_len = date_str.len();
            let padding = if used + date_len + 2 < inner_width {
                inner_width - used - date_len - 1
            } else {
                1
            };
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::styled(date_str, Style::default().fg(date_color)));
        }

        let bg = if is_selected {
            Style::default().bg(theme.bg_selected)
        } else {
            Style::default()
        };

        items.push(ListItem::new(Line::from(spans)).style(bg));
    }

    // Compute display index (accounting for separator)
    let display_index = match separator_at {
        Some(sep) if app.task_index >= first_done_idx.unwrap() => app.task_index + 1,
        _ => app.task_index,
    };

    let list = List::new(items).block(block);
    let mut state = ListState::default();
    state.select(Some(display_index));
    f.render_stateful_widget(list, area, &mut state);

}

fn format_due_date(due: NaiveDate, today: NaiveDate, theme: &Theme) -> (String, Color) {
    let days = (due - today).num_days();

    if days < -1 {
        (due.format("%b %d").to_string(), theme.error)
    } else if days == -1 {
        ("Yesterday".to_string(), theme.error)
    } else if days == 0 {
        ("Today".to_string(), theme.warning)
    } else if days == 1 {
        ("Tomorrow".to_string(), theme.warning)
    } else if days < 7 {
        (due.format("%a").to_string(), theme.accent)
    } else if due.year() == today.year() {
        (due.format("%b %d").to_string(), theme.fg_dim)
    } else {
        (due.format("%b %d, %Y").to_string(), theme.fg_dim)
    }
}
