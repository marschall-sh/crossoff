use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::{ActivePane, App};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme;
    let is_active = app.active_pane == ActivePane::Projects;

    let border_color = if is_active {
        theme.border_active
    } else {
        theme.border
    };
    let (title, title_style) = if is_active {
        (
            " Projects ",
            Style::default()
                .bg(theme.accent)
                .fg(theme.project_count_fg)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            " Projects ",
            Style::default().fg(theme.fg_dim),
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(title)
        .title_style(title_style)
        .style(Style::default().bg(theme.bg));

    let items: Vec<ListItem> = app
        .data
        .projects
        .iter()
        .enumerate()
        .map(|(i, project)| {
            let task_count = app
                .data
                .tasks
                .iter()
                .filter(|t| t.project_id == project.id && !t.done)
                .count();

            let is_selected = i == app.project_index;
            let marker = if is_selected && is_active {
                "\u{25b8} "
            } else {
                "  "
            };

            // Inbox gets filled dot, others get outline
            let icon = if project.is_inbox { "\u{25cf} " } else { "\u{00b7} " };
            let icon_color = if project.is_inbox {
                theme.accent
            } else {
                theme.fg_dim
            };

            let name_style = if is_selected {
                Style::default()
                    .fg(theme.fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            let mut spans = vec![
                Span::styled(marker, Style::default().fg(theme.cursor_marker)),
                Span::styled(icon, Style::default().fg(icon_color)),
                Span::styled(project.name.as_str(), name_style),
            ];

            // Only show count when > 0
            if task_count > 0 {
                spans.push(Span::styled(
                    format!(" {}", task_count),
                    Style::default().fg(theme.fg_dim),
                ));
            }

            let bg = if is_selected {
                Style::default().bg(theme.bg_selected)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(spans)).style(bg)
        })
        .collect();

    let list = List::new(items).block(block);
    let mut state = ListState::default();
    state.select(Some(app.project_index));
    f.render_stateful_widget(list, area, &mut state);

}
