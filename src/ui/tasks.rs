use chrono::{Datelike, Local, NaiveDate};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};
use ratatui::Frame;

use crate::app::{format_duration_compact, ActivePane, App};
use crate::model::{label_color_rgb, Task, TaskLane};
use crate::theme::Theme;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let bg = Block::default().style(Style::default().bg(theme.bg));
    f.render_widget(bg, area);

    if area.width < 24 || area.height < 5 {
        return;
    }

    let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(3)]).split(area);
    draw_board_header(f, app, chunks[0]);

    let board = chunks[1];
    let lanes = if board.width >= 90 {
        Layout::horizontal([
            Constraint::Percentage(33),
            Constraint::Length(2),
            Constraint::Percentage(34),
            Constraint::Length(2),
            Constraint::Percentage(33),
        ])
        .split(board)
    } else {
        Layout::horizontal([
            Constraint::Percentage(33),
            Constraint::Length(1),
            Constraint::Percentage(34),
            Constraint::Length(1),
            Constraint::Percentage(33),
        ])
        .split(board)
    };

    draw_lane(f, app, lanes[0], TaskLane::Inbox);
    draw_lane(f, app, lanes[2], TaskLane::Todo);
    draw_lane(f, app, lanes[4], TaskLane::Done);
}

fn draw_board_header(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let tasks = app.tasks_for_selected_project();
    let active = tasks.iter().filter(|t| !t.done).count();
    let done = tasks.iter().filter(|t| t.done).count();

    let line = Line::from(vec![
        Span::styled(
            "  crossoff",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ·  ", Style::default().fg(theme.fg_dim)),
        Span::styled(
            format!("{} active", active),
            Style::default().fg(theme.fg_dim),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(format!("{} done", done), Style::default().fg(theme.success)),
    ]);
    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(theme.bg)),
        Rect::new(area.x, area.y, area.width, 1),
    );
}

fn draw_lane(f: &mut Frame, app: &App, area: Rect, lane: TaskLane) {
    let theme = app.theme;
    let tasks = app.tasks_for_lane(lane);
    let focused = app.active_pane == ActivePane::Tasks && app.task_lane == lane;
    let lane_color = lane_indicator_color(lane, theme);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if focused {
            theme.border_active
        } else {
            theme.border
        }))
        .padding(Padding::new(1, 1, 0, 1))
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    draw_lane_header(f, app, inner, lane, tasks.len(), lane_color);

    let cards_area = Rect::new(
        inner.x,
        inner.y.saturating_add(3),
        inner.width,
        inner.height.saturating_sub(3),
    );

    if tasks.is_empty() {
        let msg = match lane {
            TaskLane::Inbox => "Add a task with n",
            TaskLane::Todo => "Move tasks here with L",
            TaskLane::Done => "Completed tasks stay here for 24h",
        };
        let empty = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border));
        let empty_inner = empty.inner(Rect::new(cards_area.x, cards_area.y, cards_area.width, 4));
        f.render_widget(
            empty,
            Rect::new(cards_area.x, cards_area.y, cards_area.width, 4),
        );
        f.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(theme.fg_dim))),
            Rect::new(
                empty_inner.x + 1,
                empty_inner.y + 1,
                empty_inner.width.saturating_sub(2),
                1,
            ),
        );
        return;
    }

    let card_height = 5;
    let gap = 1;
    let stride = card_height + gap;
    let visible = ((cards_area.height + gap) / stride).max(1) as usize;
    let start = if focused && app.task_index >= visible {
        app.task_index + 1 - visible
    } else {
        0
    };

    for (visible_i, (i, task)) in tasks
        .iter()
        .enumerate()
        .skip(start)
        .take(visible)
        .enumerate()
    {
        let y = cards_area.y + visible_i as u16 * stride;
        let selected = focused && i == app.task_index;
        render_task_card(
            f,
            app,
            task,
            Rect::new(cards_area.x, y, cards_area.width, card_height),
            selected,
        );
    }
}

fn draw_lane_header(
    f: &mut Frame,
    app: &App,
    area: Rect,
    lane: TaskLane,
    count: usize,
    lane_color: Color,
) {
    let theme = app.theme;
    let title = Line::from(vec![
        Span::styled("● ", Style::default().fg(lane_color)),
        Span::styled(
            App::lane_title(lane),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!(" {} ", count),
            Style::default()
                .fg(theme.project_count_fg)
                .bg(theme.project_count_bg),
        ),
    ]);
    f.render_widget(
        Paragraph::new(title).style(Style::default().bg(theme.bg)),
        Rect::new(area.x, area.y, area.width, 1),
    );

    let subtitle = match lane {
        TaskLane::Inbox => "Ready to be picked up",
        TaskLane::Todo => "Currently being worked on",
        TaskLane::Done => "Auto-clears after 24h",
    };
    f.render_widget(
        Paragraph::new(Span::styled(subtitle, Style::default().fg(theme.fg_dim)))
            .style(Style::default().bg(theme.bg)),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );
}

fn render_task_card(f: &mut Frame, app: &App, task: &Task, area: Rect, selected: bool) {
    let theme = app.theme;
    let border = if selected { theme.accent } else { theme.border };
    let bg = if selected {
        theme.bg_selected
    } else {
        theme.header_bg
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if selected && inner.height > 0 {
        let rail_lines: Vec<Line> = (0..inner.height)
            .map(|_| Line::from(Span::styled("┃", Style::default().fg(theme.accent))))
            .collect();
        f.render_widget(
            Paragraph::new(rail_lines).style(Style::default().bg(bg)),
            Rect::new(inner.x, inner.y, 1, inner.height),
        );
    }

    let content_x = inner.x + 2;
    let content_width = inner.width.saturating_sub(3);
    let priority_prefix = if task.pinned && !task.done {
        "★ "
    } else {
        ""
    };
    let title = truncate(
        &format!("{}{}", priority_prefix, task.title),
        content_width as usize,
    );
    let title_color = if task.pinned && !task.done {
        theme.warning
    } else {
        theme.fg
    };
    let title_style = if task.done {
        Style::default()
            .fg(theme.fg_dim)
            .add_modifier(Modifier::BOLD | Modifier::CROSSED_OUT)
    } else {
        Style::default()
            .fg(title_color)
            .add_modifier(Modifier::BOLD)
    };
    f.render_widget(
        Paragraph::new(Span::styled(title, title_style)).style(Style::default().bg(bg)),
        Rect::new(content_x, inner.y, content_width, 1),
    );

    let meta = task_meta_spans(app, task, content_width as usize);
    f.render_widget(
        Paragraph::new(Line::from(meta)).style(Style::default().bg(bg)),
        Rect::new(content_x, inner.y + 2, content_width, 1),
    );
}

fn task_meta_spans<'a>(app: &'a App, task: &'a Task, max_width: usize) -> Vec<Span<'a>> {
    let theme = app.theme;
    let today = Local::now().date_naive();
    let mut spans = Vec::new();
    let mut used = 0usize;

    if let Some(due) = task.due_date {
        let (date, color) = format_due_date(due, today, theme);
        let chip = format!(" {} ", date);
        used += chip.len() + 1;
        spans.push(Span::styled(
            chip,
            Style::default().fg(color).bg(theme.border),
        ));
        spans.push(Span::raw(" "));
    }

    for label_id in &task.label_ids {
        if let Some(label) = app.data.labels.iter().find(|l| l.id == *label_id) {
            let pill = format!(" {} ", label.name);
            if used + pill.len() > max_width.saturating_sub(8) {
                break;
            }
            let (r, g, b) = label_color_rgb(&label.color);
            let bg = Color::Rgb(r, g, b);
            let fg = if (r as u16 + g as u16 + b as u16) > 384 {
                Color::Rgb(0x1a, 0x1a, 0x1a)
            } else {
                Color::Rgb(0xf0, 0xf0, 0xf0)
            };
            used += pill.len() + 1;
            spans.push(Span::styled(pill, Style::default().bg(bg).fg(fg)));
            spans.push(Span::raw(" "));
        }
    }

    if !task.checklist.is_empty() {
        let done_count = task.checklist.iter().filter(|item| item.done).count();
        let total = task.checklist.len();
        let checklist = format!(" ✓ {}/{} ", done_count, total);
        if used + checklist.len() <= max_width.saturating_sub(8) {
            let complete = done_count == total;
            used += checklist.len() + 1;
            spans.push(Span::styled(
                checklist,
                Style::default()
                    .fg(if complete {
                        theme.success
                    } else {
                        theme.fg_dim
                    })
                    .bg(theme.border),
            ));
            spans.push(Span::raw(" "));
        }
    }

    let tracked_seconds = app.task_tracked_seconds(task.id);
    if tracked_seconds > 0 || app.is_task_running(task.id) {
        let timer = format!(
            " {}{} ",
            if app.is_task_running(task.id) {
                "● "
            } else {
                "◷ "
            },
            format_duration_compact(tracked_seconds)
        );
        if used + timer.len() <= max_width {
            let timer_bg = if app.is_task_running(task.id) {
                theme.bg_running
            } else {
                theme.border
            };
            spans.push(Span::styled(
                timer,
                Style::default().fg(theme.accent).bg(timer_bg),
            ));
        }
    }

    spans
}

fn lane_indicator_color(lane: TaskLane, theme: &Theme) -> Color {
    match lane {
        TaskLane::Inbox => theme.warning,
        TaskLane::Todo => theme.accent,
        TaskLane::Done => theme.success,
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let take = max.saturating_sub(1);
        format!("{}…", text.chars().take(take).collect::<String>())
    }
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
