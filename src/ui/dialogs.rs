use chrono::{Datelike, Local, NaiveDate};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::{
    days_in_month, App, ChecklistEditorState, DatePickerState, DeleteTarget, LabelCreateField,
    LabelPickerState, ProjectEditState, SearchState, TaskEditState, TaskField,
};
use crate::model::{label_color_rgb, LABEL_COLOR_NAMES};

use super::centered_rect;

// --- Project Edit Dialog ---

pub fn draw_project_edit(f: &mut Frame, app: &App, state: &ProjectEditState) {
    let theme = app.theme;
    let area = centered_rect(46, 7, f.area());
    f.render_widget(Clear, area);

    let title = if state.editing_id.is_some() {
        " Edit Project "
    } else {
        " New Project "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.input_border))
        .title(title)
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    f.render_widget(
        Paragraph::new(Span::styled("  Name", Style::default().fg(theme.detail_label))),
        Rect::new(inner.x, inner.y + 1, inner.width, 1),
    );
    f.render_widget(
        Paragraph::new(Span::styled(
            format!("  {}", state.input.text),
            Style::default().fg(theme.detail_value),
        ))
        .style(Style::default().bg(theme.bg_selected)),
        Rect::new(inner.x, inner.y + 2, inner.width, 1),
    );
    f.set_cursor_position((inner.x + 2 + state.input.cursor as u16, inner.y + 2));

    let hints = Line::from(vec![
        Span::styled("  ^S", Style::default().fg(theme.key_hint)),
        Span::styled("/", Style::default().fg(theme.fg_dim)),
        Span::styled("\u{23ce}", Style::default().fg(theme.key_hint)),
        Span::styled(" save  ", Style::default().fg(theme.fg_dim)),
        Span::styled("Esc", Style::default().fg(theme.key_hint)),
        Span::styled(" cancel", Style::default().fg(theme.fg_dim)),
    ]);
    f.render_widget(
        Paragraph::new(hints),
        Rect::new(inner.x, inner.y + 4, inner.width, 1),
    );
}

// --- Task Edit Dialog ---


pub fn draw_task_edit(f: &mut Frame, app: &App, state: &TaskEditState) {
    let theme = app.theme;
    // Responsive: 72 % of terminal, clamped to sensible min/max
    let width = (f.area().width * 72 / 100).max(54).min(92);
    let height = (f.area().height * 72 / 100).max(21).min(40);
    let area = centered_rect(width, height, f.area());
    f.render_widget(Clear, area);

    let title = if state.editing_id.is_some() {
        " Edit Task "
    } else {
        " New Task "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.input_border))
        .title(title)
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let label_s = Style::default().fg(theme.detail_label);
    let active_s = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);
    let y = inner.y;

    // Fixed rows: 5 above textarea (unused + title label + title value + gap + desc label)
    //           + 1 gap after textarea
    //           + 2 (due date) + 2 (labels) + 2 (checklist) = 6
    //           + 1 hints (last row) + 1 breathing row before hints
    //           = 14 fixed rows; the rest goes to the description textarea.
    let desc_rows = inner.height.saturating_sub(14).max(4);

    // --- Title (single line) ---
    draw_field(
        f, inner, y + 1, "Title",
        &format!("  {}", state.title.text),
        state.active_field == TaskField::Title,
        label_s, active_s, theme,
    );

    // --- Description (multi-line textarea) ---
    let desc_active = state.active_field == TaskField::Description;
    let desc_label = if desc_active { active_s } else { label_s };
    f.render_widget(
        Paragraph::new(Span::styled("  Description", desc_label)),
        Rect::new(inner.x, y + 4, inner.width, 1),
    );

    let text_area_x = inner.x + 2;
    let text_area_w = inner.width.saturating_sub(3);
    let text_area_y = y + 5;
    let text_area = Rect::new(text_area_x, text_area_y, text_area_w, desc_rows);

    // Compute cursor position within the text area (logical lines, not soft-wrap)
    let (cx, cy) = textarea_cursor_pos(&state.description.text, state.description.cursor);
    let scroll_y = cy.saturating_sub(desc_rows.saturating_sub(1));
    let scroll_x = cx.saturating_sub(text_area_w.saturating_sub(1));

    // Render the background for the whole text area when active
    if desc_active {
        for row in 0..desc_rows {
            f.render_widget(
                Paragraph::new("").style(Style::default().bg(theme.bg_selected)),
                Rect::new(inner.x, text_area_y + row, inner.width, 1),
            );
        }
    }

    let display_text = if state.description.text.is_empty() && !desc_active {
        "Type a description..."
    } else {
        &state.description.text
    };
    let desc_fg = if state.description.text.is_empty() && !desc_active {
        theme.fg_dim
    } else {
        theme.detail_value
    };

    let desc_para = Paragraph::new(display_text)
        .style(Style::default().fg(desc_fg))
        .scroll((scroll_y, scroll_x));
    f.render_widget(desc_para, text_area);

    if desc_active {
        f.set_cursor_position((
            text_area_x + cx.saturating_sub(scroll_x),
            text_area_y + cy.saturating_sub(scroll_y),
        ));
    }

    // --- Due Date (follows immediately after textarea + 1 gap row) ---
    let row_due = y + 5 + desc_rows + 1;
    let date_text = match state.due_date {
        Some(d) => format!("  {}", d.format("%b %d, %Y")),
        None => "  No date (\u{23ce} to pick)".to_string(),
    };
    draw_field(
        f, inner, row_due, "Due Date", &date_text,
        state.active_field == TaskField::DueDate,
        label_s, active_s, theme,
    );

    // --- Labels ---
    let row_lbl = row_due + 2;
    let label_text = if state.label_ids.is_empty() {
        "  No labels (\u{23ce} to pick)".to_string()
    } else {
        let names: Vec<&str> = state
            .label_ids
            .iter()
            .filter_map(|id| {
                app.data
                    .labels
                    .iter()
                    .find(|l| l.id == *id)
                    .map(|l| l.name.as_str())
            })
            .collect();
        format!("  {}", names.join(", "))
    };
    draw_field(
        f, inner, row_lbl, "Labels", &label_text,
        state.active_field == TaskField::Labels,
        label_s, active_s, theme,
    );

    // --- Checklist ---
    let row_cl = row_lbl + 2;
    let cl_text = if state.checklist.is_empty() {
        "  No items (\u{23ce} to edit)".to_string()
    } else {
        let done = state.checklist.iter().filter(|i| i.done).count();
        format!("  {} items ({} done)", state.checklist.len(), done)
    };
    draw_field(
        f, inner, row_cl, "Checklist", &cl_text,
        state.active_field == TaskField::Checklist,
        label_s, active_s, theme,
    );

    // --- Cursor for Title ---
    if state.active_field == TaskField::Title {
        f.set_cursor_position((inner.x + 2 + state.title.cursor as u16, y + 2));
    }

    // --- Hints ---
    let hints = Line::from(vec![
        Span::styled("  ^S", Style::default().fg(theme.key_hint)),
        Span::styled(" save  ", Style::default().fg(theme.fg_dim)),
        Span::styled("Tab", Style::default().fg(theme.key_hint)),
        Span::styled(" next  ", Style::default().fg(theme.fg_dim)),
        Span::styled("Esc", Style::default().fg(theme.key_hint)),
        Span::styled(" cancel", Style::default().fg(theme.fg_dim)),
    ]);
    f.render_widget(
        Paragraph::new(hints),
        Rect::new(inner.x, inner.y + inner.height.saturating_sub(1), inner.width, 1),
    );
}

/// Compute (col, row) cursor position within explicit newline-separated text.
fn textarea_cursor_pos(text: &str, cursor: usize) -> (u16, u16) {
    let mut col = 0usize;
    let mut row = 0usize;

    for (i, ch) in text.char_indices() {
        if i >= cursor {
            break;
        }
        if ch == '\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (col as u16, row as u16)
}

fn draw_field(
    f: &mut Frame,
    inner: Rect,
    y: u16,
    label: &str,
    value: &str,
    is_active: bool,
    label_style: Style,
    active_style: Style,
    theme: &crate::theme::Theme,
) {
    let ls = if is_active { active_style } else { label_style };
    f.render_widget(
        Paragraph::new(Span::styled(format!("  {}", label), ls)),
        Rect::new(inner.x, y, inner.width, 1),
    );
    let vs = if is_active {
        Style::default()
            .bg(theme.bg_selected)
            .fg(theme.detail_value)
    } else {
        Style::default().fg(if value.contains("No ") {
            theme.fg_dim
        } else {
            theme.detail_value
        })
    };
    f.render_widget(
        Paragraph::new(Span::styled(value, vs)).style(if is_active {
            Style::default().bg(theme.bg_selected)
        } else {
            Style::default()
        }),
        Rect::new(inner.x, y + 1, inner.width, 1),
    );
}

// --- Date Picker Dialog ---

pub fn draw_date_picker(f: &mut Frame, app: &App, state: &DatePickerState) {
    let theme = app.theme;
    let area = centered_rect(36, 16, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.input_border))
        .title(" Due Date ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let today = Local::now().date_naive();

    let month_names = [
        "January", "February", "March", "April", "May", "June", "July", "August", "September",
        "October", "November", "December",
    ];
    let month_name = month_names[(state.month - 1) as usize];
    let month_str = format!("\u{25c0}  {} {}  \u{25b6}", month_name, state.year);
    let padding = (inner.width as usize).saturating_sub(month_str.len()) / 2;
    f.render_widget(
        Paragraph::new(Span::styled(
            format!("{}{}", " ".repeat(padding), month_str),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )),
        Rect::new(inner.x, inner.y + 1, inner.width, 1),
    );

    let headers = Line::from(vec![
        Span::styled(" Mo", Style::default().fg(theme.fg_dim)),
        Span::styled("  Tu", Style::default().fg(theme.fg_dim)),
        Span::styled("  We", Style::default().fg(theme.fg_dim)),
        Span::styled("  Th", Style::default().fg(theme.fg_dim)),
        Span::styled("  Fr", Style::default().fg(theme.fg_dim)),
        Span::styled("  Sa", Style::default().fg(theme.fg_dim)),
        Span::styled("  Su", Style::default().fg(theme.fg_dim)),
    ]);
    f.render_widget(
        Paragraph::new(headers),
        Rect::new(inner.x, inner.y + 3, inner.width, 1),
    );

    let first = NaiveDate::from_ymd_opt(state.year, state.month, 1).unwrap();
    let weekday_offset = first.weekday().num_days_from_monday() as usize;
    let dim = days_in_month(state.year, state.month);

    let mut day = 1u32;
    for week in 0..6u16 {
        if day > dim {
            break;
        }
        let mut spans = Vec::new();
        for wd in 0..7usize {
            if (week == 0 && wd < weekday_offset) || day > dim {
                spans.push(Span::raw("    "));
            } else {
                let date = NaiveDate::from_ymd_opt(state.year, state.month, day).unwrap();
                let is_selected = date == state.selected;
                let is_today = date == today;
                let text = format!("{:>4}", day);
                let style = if is_selected {
                    Style::default()
                        .bg(theme.accent)
                        .fg(theme.project_count_fg)
                        .add_modifier(Modifier::BOLD)
                } else if is_today {
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg)
                };
                spans.push(Span::styled(text, style));
                day += 1;
            }
        }
        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(inner.x, inner.y + 4 + week, inner.width, 1),
        );
    }

    let hint_y = inner.y + inner.height.saturating_sub(2);
    let hints = Line::from(vec![
        Span::styled(" t", Style::default().fg(theme.key_hint)),
        Span::styled(":Today ", Style::default().fg(theme.fg_dim)),
        Span::styled("\u{23ce}", Style::default().fg(theme.key_hint)),
        Span::styled(":Pick ", Style::default().fg(theme.fg_dim)),
        Span::styled("Bs", Style::default().fg(theme.key_hint)),
        Span::styled(":Clear ", Style::default().fg(theme.fg_dim)),
        Span::styled("Esc", Style::default().fg(theme.key_hint)),
        Span::styled(":Back", Style::default().fg(theme.fg_dim)),
    ]);
    f.render_widget(
        Paragraph::new(hints),
        Rect::new(inner.x, hint_y, inner.width, 1),
    );
}

// --- Label Picker Dialog ---

pub fn draw_label_picker(f: &mut Frame, app: &App, state: &LabelPickerState) {
    let theme = app.theme;

    if let Some(ref creating) = state.creating {
        draw_label_create(f, app, creating);
        return;
    }

    let item_count = app.data.labels.len() as u16;
    let height = (item_count + 6).clamp(8, 18);
    let area = centered_rect(44, height, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.input_border))
        .title(" Labels ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.data.labels.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(
                "  No labels yet. Press 'n' to create one.",
                Style::default().fg(theme.fg_dim),
            )),
            Rect::new(inner.x, inner.y + 1, inner.width, 1),
        );
    } else {
        for (i, label) in app.data.labels.iter().enumerate() {
            if i as u16 >= inner.height.saturating_sub(3) {
                break;
            }
            let is_selected = i == state.index;
            let is_assigned = state.assigned.contains(&label.id);
            let marker = if is_selected { "\u{25b8} " } else { "  " };
            let checkbox = if is_assigned { "\u{2611} " } else { "\u{2610} " };
            let (r, g, b) = label_color_rgb(&label.color);

            let line = Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.cursor_marker)),
                Span::styled(
                    checkbox,
                    Style::default().fg(if is_assigned {
                        theme.success
                    } else {
                        theme.fg_dim
                    }),
                ),
                Span::styled("\u{25cf} ", Style::default().fg(Color::Rgb(r, g, b))),
                Span::styled(label.name.as_str(), Style::default().fg(theme.fg)),
            ]);
            let bg = if is_selected {
                Style::default().bg(theme.bg_selected)
            } else {
                Style::default()
            };
            f.render_widget(
                Paragraph::new(line).style(bg),
                Rect::new(inner.x, inner.y + 1 + i as u16, inner.width, 1),
            );
        }
    }

    let hy = inner.y + inner.height.saturating_sub(2);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Sp", Style::default().fg(theme.key_hint)),
            Span::styled(" toggle  ", Style::default().fg(theme.fg_dim)),
            Span::styled("n", Style::default().fg(theme.key_hint)),
            Span::styled(" new  ", Style::default().fg(theme.fg_dim)),
            Span::styled("d", Style::default().fg(theme.key_hint)),
            Span::styled(" del  ", Style::default().fg(theme.fg_dim)),
            Span::styled("^S", Style::default().fg(theme.key_hint)),
            Span::styled("/", Style::default().fg(theme.fg_dim)),
            Span::styled("\u{23ce}", Style::default().fg(theme.key_hint)),
            Span::styled(" done", Style::default().fg(theme.fg_dim)),
        ])),
        Rect::new(inner.x, hy, inner.width, 1),
    );
}

fn draw_label_create(f: &mut Frame, app: &App, state: &crate::app::LabelCreateState) {
    let theme = app.theme;
    let area = centered_rect(44, 11, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.input_border))
        .title(" New Label ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let label_s = Style::default().fg(theme.detail_label);
    let active_s = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);

    // Name
    let ns = if state.active_field == LabelCreateField::Name {
        active_s
    } else {
        label_s
    };
    f.render_widget(
        Paragraph::new(Span::styled("  Name", ns)),
        Rect::new(inner.x, inner.y + 1, inner.width, 1),
    );
    f.render_widget(
        Paragraph::new(Span::styled(
            format!("  {}", state.name.text),
            Style::default().fg(theme.detail_value),
        ))
        .style(if state.active_field == LabelCreateField::Name {
            Style::default().bg(theme.bg_selected)
        } else {
            Style::default()
        }),
        Rect::new(inner.x, inner.y + 2, inner.width, 1),
    );

    // Color
    let cs = if state.active_field == LabelCreateField::Color {
        active_s
    } else {
        label_s
    };
    f.render_widget(
        Paragraph::new(Span::styled("  Color", cs)),
        Rect::new(inner.x, inner.y + 4, inner.width, 1),
    );

    let mut color_spans = vec![Span::raw("  ")];
    for (i, &name) in LABEL_COLOR_NAMES.iter().enumerate() {
        let (r, g, b) = label_color_rgb(name);
        let sym = if i == state.color_index {
            "\u{25c6} "
        } else {
            "\u{25cf} "
        };
        let mut s = Style::default().fg(Color::Rgb(r, g, b));
        if i == state.color_index {
            s = s.add_modifier(Modifier::BOLD);
        }
        color_spans.push(Span::styled(sym, s));
    }
    f.render_widget(
        Paragraph::new(Line::from(color_spans)).style(
            if state.active_field == LabelCreateField::Color {
                Style::default().bg(theme.bg_selected)
            } else {
                Style::default()
            },
        ),
        Rect::new(inner.x, inner.y + 5, inner.width, 1),
    );
    f.render_widget(
        Paragraph::new(Span::styled(
            format!("  {}", LABEL_COLOR_NAMES[state.color_index]),
            Style::default().fg(theme.fg_dim),
        )),
        Rect::new(inner.x, inner.y + 6, inner.width, 1),
    );

    if state.active_field == LabelCreateField::Name {
        f.set_cursor_position((inner.x + 2 + state.name.cursor as u16, inner.y + 2));
    }

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  ^S", Style::default().fg(theme.key_hint)),
            Span::styled("/", Style::default().fg(theme.fg_dim)),
            Span::styled("\u{23ce}", Style::default().fg(theme.key_hint)),
            Span::styled(" save  ", Style::default().fg(theme.fg_dim)),
            Span::styled("Tab", Style::default().fg(theme.key_hint)),
            Span::styled(" next  ", Style::default().fg(theme.fg_dim)),
            Span::styled("Esc", Style::default().fg(theme.key_hint)),
            Span::styled(" cancel", Style::default().fg(theme.fg_dim)),
        ])),
        Rect::new(inner.x, inner.y + 8, inner.width, 1),
    );
}

// --- Checklist Editor Dialog ---

pub fn draw_checklist_editor(f: &mut Frame, app: &App, state: &ChecklistEditorState) {
    let theme = app.theme;
    let item_count = state.items.len() as u16;
    let height = (item_count + 6).clamp(8, 20);
    let area = centered_rect(50, height, f.area());
    f.render_widget(Clear, area);

    let done_count = state.items.iter().filter(|i| i.done).count();
    let total = state.items.len();
    let title_text = if total > 0 {
        format!(" Checklist ({}/{}) ", done_count, total)
    } else {
        " Checklist ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.input_border))
        .title(title_text)
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.items.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(
                "  No items yet. Press 'n' to add one.",
                Style::default().fg(theme.fg_dim),
            )),
            Rect::new(inner.x, inner.y + 1, inner.width, 1),
        );
    } else {
        for (i, item) in state.items.iter().enumerate() {
            if i as u16 >= inner.height.saturating_sub(3) {
                break;
            }
            let is_selected = i == state.index;
            let marker = if is_selected { "\u{25b8} " } else { "  " };
            let checkbox = if item.done { "\u{2611} " } else { "\u{2610} " };
            let check_color = if item.done { theme.success } else { theme.fg_dim };

            let is_editing = is_selected && state.editing.is_some();
            let display_text = if is_editing {
                state.editing.as_ref().unwrap().text.as_str()
            } else {
                item.text.as_str()
            };
            let text_style = if is_editing {
                Style::default().fg(theme.detail_value)
            } else if item.done {
                Style::default()
                    .fg(theme.fg_dim)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().fg(theme.fg)
            };

            let line = Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.cursor_marker)),
                Span::styled(checkbox, Style::default().fg(check_color)),
                Span::styled(display_text, text_style),
            ]);
            let bg = if is_selected {
                Style::default().bg(theme.bg_selected)
            } else {
                Style::default()
            };
            let row_y = inner.y + 1 + i as u16;
            f.render_widget(Paragraph::new(line).style(bg), Rect::new(inner.x, row_y, inner.width, 1));

            if is_editing {
                let cx = inner.x + 2 + 2 + state.editing.as_ref().unwrap().cursor as u16;
                f.set_cursor_position((cx, row_y));
            }
        }
    }

    let hy = inner.y + inner.height.saturating_sub(2);
    if state.editing.is_some() {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" \u{23ce}", Style::default().fg(theme.key_hint)),
                Span::styled(" confirm  ", Style::default().fg(theme.fg_dim)),
                Span::styled("Esc", Style::default().fg(theme.key_hint)),
                Span::styled(" cancel", Style::default().fg(theme.fg_dim)),
            ])),
            Rect::new(inner.x, hy, inner.width, 1),
        );
    } else {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" Sp", Style::default().fg(theme.key_hint)),
                Span::styled(" toggle  ", Style::default().fg(theme.fg_dim)),
                Span::styled("n", Style::default().fg(theme.key_hint)),
                Span::styled(" new  ", Style::default().fg(theme.fg_dim)),
                Span::styled("e", Style::default().fg(theme.key_hint)),
                Span::styled(" edit  ", Style::default().fg(theme.fg_dim)),
                Span::styled("d", Style::default().fg(theme.key_hint)),
                Span::styled(" del  ", Style::default().fg(theme.fg_dim)),
                Span::styled("^S", Style::default().fg(theme.key_hint)),
                Span::styled("/", Style::default().fg(theme.fg_dim)),
                Span::styled("Esc", Style::default().fg(theme.key_hint)),
                Span::styled(" done", Style::default().fg(theme.fg_dim)),
            ])),
            Rect::new(inner.x, hy, inner.width, 1),
        );
    }
}

// --- Search Dialog ---

pub fn draw_search(f: &mut Frame, app: &App, state: &SearchState) {
    let theme = app.theme;
    let today = Local::now().date_naive();

    let width = (f.area().width * 70 / 100).max(50).min(76);
    let height = (f.area().height * 70 / 100).max(12).min(30);
    let area = centered_rect(width, height, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .title(" Search ")
        .title_style(
            Style::default()
                .bg(theme.accent)
                .fg(theme.project_count_fg)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 4 || inner.width < 10 {
        return;
    }

    // Search input
    let input_line = Line::from(vec![
        Span::styled("  \u{25b8} ", Style::default().fg(theme.accent)),
        Span::styled(
            state.query.text.as_str(),
            Style::default().fg(theme.detail_value),
        ),
    ]);
    f.render_widget(
        Paragraph::new(input_line).style(Style::default().bg(theme.bg_selected)),
        Rect::new(inner.x, inner.y + 1, inner.width, 1),
    );
    f.set_cursor_position((inner.x + 4 + state.query.text.len() as u16, inner.y + 1));

    // Results
    let results_y = inner.y + 3;
    let results_height = inner.height.saturating_sub(5);

    if state.results.is_empty() && !state.query.text.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(
                "  No results found",
                Style::default().fg(theme.fg_dim),
            )),
            Rect::new(inner.x, results_y, inner.width, 1),
        );
    } else {
        let mut y = results_y;
        for (i, result) in state.results.iter().enumerate() {
            if y + 1 >= results_y + results_height {
                break;
            }

            let is_selected = i == state.selected;
            let marker = if is_selected { "\u{25b8} " } else { "  " };
            let row_bg = if is_selected {
                Style::default().bg(theme.bg_selected)
            } else {
                Style::default()
            };

            // Line 1: marker + highlighted title + labels
            let mut title_spans = vec![Span::styled(
                marker,
                Style::default().fg(theme.cursor_marker),
            )];

            // Render title with fuzzy match highlights
            if result.matched_indices.is_empty() {
                let ts = if result.done {
                    Style::default()
                        .fg(theme.fg_dim)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default().fg(theme.fg)
                };
                title_spans.push(Span::styled(result.title.as_str(), ts));
            } else {
                let chars: Vec<char> = result.title.chars().collect();
                let mut run = String::new();
                let mut in_match = false;

                let normal = if result.done {
                    Style::default()
                        .fg(theme.fg_dim)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default().fg(theme.fg)
                };
                let highlight = Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

                for (ci, &ch) in chars.iter().enumerate() {
                    let is_match = result.matched_indices.contains(&ci);
                    if is_match != in_match {
                        if !run.is_empty() {
                            title_spans.push(Span::styled(
                                run.clone(),
                                if in_match { highlight } else { normal },
                            ));
                            run.clear();
                        }
                        in_match = is_match;
                    }
                    run.push(ch);
                }
                if !run.is_empty() {
                    title_spans.push(Span::styled(
                        run,
                        if in_match { highlight } else { normal },
                    ));
                }
            }

            // Inline label pills
            for label_id in &result.label_ids {
                if let Some(label) = app.data.labels.iter().find(|l| l.id == *label_id) {
                    let (r, g, b) = label_color_rgb(&label.color);
                    let bg = Color::Rgb(r, g, b);
                    let fg = if (r as u16 + g as u16 + b as u16) > 384 {
                        Color::Rgb(0x1a, 0x1a, 0x1a)
                    } else {
                        Color::Rgb(0xf0, 0xf0, 0xf0)
                    };
                    title_spans.push(Span::raw(" "));
                    title_spans.push(Span::styled(
                        format!(" {} ", label.name),
                        Style::default().bg(bg).fg(fg),
                    ));
                }
            }

            f.render_widget(
                Paragraph::new(Line::from(title_spans)).style(row_bg),
                Rect::new(inner.x, y, inner.width, 1),
            );
            y += 1;

            // Line 2: project name + due date
            if y < results_y + results_height {
                let mut ctx_spans = vec![Span::styled(
                    format!("    {}", result.project_name),
                    Style::default().fg(theme.fg_dim),
                )];
                if let Some(due) = result.due_date {
                    let days = (due - today).num_days();
                    let (date_str, date_color) = if days < 0 {
                        ("Overdue".to_string(), theme.error)
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
                    };
                    ctx_spans.push(Span::styled(
                        " \u{00b7} ",
                        Style::default().fg(theme.border),
                    ));
                    ctx_spans.push(Span::styled(date_str, Style::default().fg(date_color)));
                }
                f.render_widget(
                    Paragraph::new(Line::from(ctx_spans)).style(row_bg),
                    Rect::new(inner.x, y, inner.width, 1),
                );
                y += 1;
            }

            y += 1; // gap between results
        }
    }

    // Hints
    let hy = inner.y + inner.height.saturating_sub(1);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  \u{2191}\u{2193}", Style::default().fg(theme.key_hint)),
            Span::styled(" navigate  ", Style::default().fg(theme.fg_dim)),
            Span::styled("\u{23ce}", Style::default().fg(theme.key_hint)),
            Span::styled(" jump to  ", Style::default().fg(theme.fg_dim)),
            Span::styled("Esc", Style::default().fg(theme.key_hint)),
            Span::styled(" close", Style::default().fg(theme.fg_dim)),
        ])),
        Rect::new(inner.x, hy, inner.width, 1),
    );
}

// --- Confirm Delete Dialog ---

pub fn draw_confirm_delete(f: &mut Frame, app: &App, target: &DeleteTarget) {
    let theme = app.theme;
    let area = centered_rect(46, 7, f.area());
    f.render_widget(Clear, area);

    let (title, msg1, msg2) = match target {
        DeleteTarget::Project {
            name, task_count, ..
        } => {
            let line2 = if *task_count > 0 {
                format!("  This will also delete {} task(s).", task_count)
            } else {
                String::new()
            };
            (" Delete Project ", format!("  Delete \"{}\"?", name), line2)
        }
        DeleteTarget::Task { title, .. } => {
            let display = if title.len() > 30 {
                format!("{}...", &title[..27])
            } else {
                title.clone()
            };
            (
                " Delete Task ",
                format!("  Delete \"{}\"?", display),
                String::new(),
            )
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.error))
        .title(title)
        .title_style(Style::default().fg(theme.error).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    f.render_widget(
        Paragraph::new(Span::styled(&msg1, Style::default().fg(theme.fg))),
        Rect::new(inner.x, inner.y + 1, inner.width, 1),
    );
    if !msg2.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(&msg2, Style::default().fg(theme.warning))),
            Rect::new(inner.x, inner.y + 2, inner.width, 1),
        );
    }

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  y", Style::default().fg(theme.key_hint)),
            Span::styled(" confirm  ", Style::default().fg(theme.fg_dim)),
            Span::styled("n", Style::default().fg(theme.key_hint)),
            Span::styled(" cancel", Style::default().fg(theme.fg_dim)),
        ])),
        Rect::new(
            inner.x,
            inner.y + inner.height.saturating_sub(1),
            inner.width,
            1,
        ),
    );
}

// --- Help Dialog ---

pub fn draw_help(f: &mut Frame, app: &App) {
    let theme = app.theme;
    let area = centered_rect(56, 26, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .title(" Help ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.bg));

    let section = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);
    let key = Style::default().fg(theme.key_hint);
    let desc = Style::default().fg(theme.fg);
    let dim = Style::default().fg(theme.fg_dim);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("  Navigation", section)),
        help_line("    \u{2191}/\u{2193} or j/k", "Navigate items", key, desc),
        help_line("    Tab", "Switch between panes", key, desc),
        help_line("    Enter/Space", "Toggle task done", key, desc),
        help_line("    p", "Pin/unpin task to top", key, desc),
        Line::from(""),
        Line::from(Span::styled("  Actions", section)),
        help_line("    n", "New project or task", key, desc),
        help_line("    e", "Edit selected item", key, desc),
        help_line("    d", "Delete selected item", key, desc),
        help_line("    /", "Fuzzy search all tasks", key, desc),
        Line::from(""),
        Line::from(Span::styled("  Editors", section)),
        help_line("    Ctrl+S", "Save (works everywhere)", key, desc),
        help_line("    Tab / Shift+Tab", "Navigate fields", key, desc),
        help_line("    Enter", "Confirm / Open sub-editor", key, desc),
        help_line("    Esc", "Cancel / Go back", key, desc),
        Line::from(""),
        Line::from(Span::styled("  Date Picker", section)),
        help_line("    \u{2190}/\u{2192}", "Previous/next day", key, desc),
        help_line("    \u{2191}/\u{2193}", "Previous/next week", key, desc),
        help_line("    </> or PgUp/Dn", "Previous/next month", key, desc),
        help_line("    t", "Jump to today", key, desc),
        Line::from(""),
        Line::from(Span::styled("          Press any key to close", dim)),
    ];

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn help_line<'a>(k: &'a str, d: &'a str, key_style: Style, desc_style: Style) -> Line<'a> {
    let padding = 24usize.saturating_sub(k.len());
    Line::from(vec![
        Span::styled(k, key_style),
        Span::raw(" ".repeat(padding)),
        Span::styled(d, desc_style),
    ])
}
