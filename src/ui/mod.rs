mod detail;
mod dialogs;
mod projects;
mod tasks;

use chrono::Local;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::app::{ActivePane, App, InputMode};

pub fn draw(f: &mut Frame, app: &App) {
    let theme = app.theme;

    let bg = Block::default().style(Style::default().bg(theme.bg));
    f.render_widget(bg, f.area());

    let chunks = Layout::vertical([
        Constraint::Length(1), // top breathing room
        Constraint::Min(3),    // content
        Constraint::Length(1), // status bar
    ])
    .split(f.area());

    draw_content(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);

    match &app.input_mode {
        InputMode::ProjectEdit(state) => dialogs::draw_project_edit(f, app, state),
        InputMode::TaskEdit(state) => dialogs::draw_task_edit(f, app, state),
        InputMode::DatePicker(state) => dialogs::draw_date_picker(f, app, state),
        InputMode::LabelPicker(state) => dialogs::draw_label_picker(f, app, state),
        InputMode::ChecklistEditor(state) => dialogs::draw_checklist_editor(f, app, state),
        InputMode::Search(state) => dialogs::draw_search(f, app, state),
        InputMode::ConfirmDelete(target) => dialogs::draw_confirm_delete(f, app, target),
        InputMode::Help => dialogs::draw_help(f, app),
        InputMode::Normal => {}
    }
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::horizontal([
        Constraint::Length(22),
        Constraint::Min(30),
    ])
    .split(area);

    projects::draw(f, app, chunks[0]);
    draw_right_pane(f, app, chunks[1]);
}

fn draw_right_pane(f: &mut Frame, app: &App, area: Rect) {
    let has_tasks = app
        .selected_project()
        .map(|p| app.data.tasks.iter().any(|t| t.project_id == p.id))
        .unwrap_or(false);

    if has_tasks {
        let chunks = Layout::vertical([
            Constraint::Percentage(58),
            Constraint::Percentage(42),
        ])
        .split(area);

        tasks::draw(f, app, chunks[0]);
        detail::draw(f, app, chunks[1]);
    } else {
        tasks::draw(f, app, area);
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let today = Local::now().date_naive();

    let bg = Block::default().style(Style::default().bg(theme.header_bg));
    f.render_widget(bg, area);

    // Branding
    let brand_spans = vec![
        Span::styled(" \u{25a2} ", Style::default().fg(theme.accent)),
        Span::styled(
            "crossoff",
            Style::default()
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    // Task stats for selected project
    let tasks = app.tasks_for_selected_project();
    let active_count = tasks.iter().filter(|t| !t.done).count();
    let overdue_count = tasks
        .iter()
        .filter(|t| !t.done && t.due_date.map(|d| d < today).unwrap_or(false))
        .count();

    let mut stats_spans: Vec<Span> = vec![Span::styled("  \u{2502} ", Style::default().fg(theme.border))];
    stats_spans.push(Span::styled(
        format!("{}", active_count),
        Style::default()
            .fg(theme.fg)
            .add_modifier(Modifier::BOLD),
    ));
    stats_spans.push(Span::styled(
        if active_count == 1 { " task" } else { " tasks" },
        Style::default().fg(theme.fg_dim),
    ));
    if overdue_count > 0 {
        stats_spans.push(Span::styled("  ", Style::default()));
        stats_spans.push(Span::styled(
            format!("{}", overdue_count),
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ));
        stats_spans.push(Span::styled(" overdue", Style::default().fg(theme.error)));
    }

    // Key hints
    let hint_spans = match app.active_pane {
        ActivePane::Projects => build_hints(
            &[
                ("n", "new"),
                ("e", "edit"),
                ("d", "del"),
                ("/", "search"),
                ("?", "help"),
                ("q", "quit"),
            ],
            theme,
        ),
        ActivePane::Tasks => build_hints(
            &[
                ("n", "new"),
                ("e", "edit"),
                ("d", "del"),
                ("p", "pin"),
                ("/", "search"),
                ("?", "help"),
            ],
            theme,
        ),
        ActivePane::Detail => build_hints(
            &[
                ("\u{2191}\u{2193}", "scroll"),
                ("Tab", "back"),
                ("/", "search"),
                ("?", "help"),
            ],
            theme,
        ),
    };

    // Calculate widths and padding
    let brand_width: usize = 12;
    let stats_width: usize = stats_spans.iter().map(|s| s.content.len()).sum();
    let hint_width: usize = hint_spans.iter().map(|s| s.content.len()).sum();
    let used = brand_width + stats_width + hint_width;
    let padding = (area.width as usize).saturating_sub(used + 1);

    let mut spans = brand_spans;
    spans.extend(stats_spans);
    spans.push(Span::raw(" ".repeat(padding)));
    spans.extend(hint_spans);

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.header_bg));
    f.render_widget(bar, area);
}

fn build_hints<'a>(hints: &[(&'a str, &'a str)], theme: &crate::theme::Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default().fg(theme.fg_dim)));
        }
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(theme.key_hint)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(theme.fg_dim),
        ));
    }
    spans
}

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
