use std::cmp::Ordering;

use chrono::{Datelike, Local, NaiveDate, Utc};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

use crate::config;
use crate::model::{
    AppData, ChecklistItem, Label, Project, Task, TimeSession, LABEL_COLOR_NAMES,
};
use crate::store;
use crate::theme::{self, Theme};

// --- Text Input Helper ---

#[derive(Clone, Debug)]
pub struct TextInput {
    pub text: String,
    pub cursor: usize,
}

impl TextInput {
    pub fn new(text: String) -> Self {
        let cursor = text.len();
        Self { text, cursor }
    }

    pub fn empty() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let new_cursor = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.text.drain(new_cursor..self.cursor);
            self.cursor = new_cursor;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.text.len() {
            let next = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
            self.text.drain(self.cursor..next);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
        }
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.text.len();
    }
}

fn apply_text_input(input: &mut TextInput, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            input.insert_char(c);
        }
        KeyCode::Backspace => input.backspace(),
        KeyCode::Delete => input.delete(),
        KeyCode::Left => input.move_left(),
        KeyCode::Right => input.move_right(),
        KeyCode::Home => input.home(),
        KeyCode::End => input.end(),
        _ => {}
    }
}

fn is_save(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL)
}

fn paste_into(input: &mut TextInput, text: &str, allow_newlines: bool) {
    for c in text.chars() {
        if c == '\r' {
            continue;
        }
        if c == '\n' && !allow_newlines {
            continue;
        }
        input.insert_char(c);
    }
}

fn move_cursor_up(input: &mut TextInput) {
    let text = &input.text;
    let cursor = input.cursor;
    let before = &text[..cursor];
    let cur_line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    if cur_line_start == 0 {
        return;
    }
    let col = text[cur_line_start..cursor].chars().count();
    let prev_line_start = text[..cur_line_start - 1]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let prev_line = &text[prev_line_start..cur_line_start - 1];
    let target_col = col.min(prev_line.chars().count());
    let byte_offset = prev_line
        .char_indices()
        .nth(target_col)
        .map(|(i, _)| i)
        .unwrap_or(prev_line.len());
    input.cursor = prev_line_start + byte_offset;
}

fn move_cursor_down(input: &mut TextInput) {
    let text = &input.text;
    let cursor = input.cursor;
    let before = &text[..cursor];
    let cur_line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let col = text[cur_line_start..cursor].chars().count();
    let Some(newline_pos) = text[cursor..].find('\n').map(|i| cursor + i) else {
        return;
    };
    let next_line_start = newline_pos + 1;
    let next_line_end = text[next_line_start..]
        .find('\n')
        .map(|i| next_line_start + i)
        .unwrap_or(text.len());
    let next_line = &text[next_line_start..next_line_end];
    let target_col = col.min(next_line.chars().count());
    let byte_offset = next_line
        .char_indices()
        .nth(target_col)
        .map(|(i, _)| i)
        .unwrap_or(next_line.len());
    input.cursor = next_line_start + byte_offset;
}

// --- App State Enums ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePane {
    Projects,
    Tasks,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskField {
    Title,
    Description,
    DueDate,
    Labels,
    Checklist,
}

#[derive(Clone, Debug)]
pub struct ProjectEditState {
    pub input: TextInput,
    pub editing_id: Option<Uuid>,
}

#[derive(Clone, Debug)]
pub struct TaskEditState {
    pub title: TextInput,
    pub description: TextInput,
    pub due_date: Option<NaiveDate>,
    pub label_ids: Vec<Uuid>,
    pub checklist: Vec<ChecklistItem>,
    pub active_field: TaskField,
    pub editing_id: Option<Uuid>,
}

pub struct DatePickerState {
    pub selected: NaiveDate,
    pub year: i32,
    pub month: u32,
    pub task_state: TaskEditState,
}

pub struct LabelPickerState {
    pub index: usize,
    pub assigned: Vec<Uuid>,
    pub task_state: TaskEditState,
    pub creating: Option<LabelCreateState>,
}

pub struct LabelCreateState {
    pub name: TextInput,
    pub color_index: usize,
    pub active_field: LabelCreateField,
}

#[derive(PartialEq)]
pub enum LabelCreateField {
    Name,
    Color,
}

pub struct ChecklistEditorState {
    pub index: usize,
    pub items: Vec<ChecklistItem>,
    pub task_state: TaskEditState,
    pub editing: Option<TextInput>,
}

pub struct SearchState {
    pub query: TextInput,
    pub results: Vec<SearchResult>,
    pub selected: usize,
}

pub struct MoveTaskState {
    pub task_id: Uuid,
    pub index: usize,
}

pub struct SearchResult {
    pub task_id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub project_name: String,
    pub due_date: Option<NaiveDate>,
    pub done: bool,
    pub label_ids: Vec<Uuid>,
    pub matched_indices: Vec<usize>,
}

pub enum DeleteTarget {
    Project {
        id: Uuid,
        name: String,
        task_count: usize,
    },
    Task {
        id: Uuid,
        title: String,
    },
}

pub enum InputMode {
    Normal,
    ProjectEdit(ProjectEditState),
    TaskEdit(TaskEditState),
    DatePicker(DatePickerState),
    LabelPicker(LabelPickerState),
    ChecklistEditor(ChecklistEditorState),
    Search(SearchState),
    MoveTask(MoveTaskState),
    ConfirmDelete(DeleteTarget),
    Help,
}

// --- App ---

pub struct App {
    pub data: AppData,
    pub active_pane: ActivePane,
    pub project_index: usize,
    pub task_index: usize,
    pub detail_scroll: u16,
    pub input_mode: InputMode,
    pub theme: &'static Theme,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = config::load();
        let theme = theme::by_name(&config.theme);
        let mut data = store::load()?;

        if !data.projects.iter().any(|p| p.is_inbox) {
            data.projects
                .insert(0, Project::new("Inbox".to_string(), true));
        }

        let mut app = Self {
            data,
            active_pane: ActivePane::Projects,
            project_index: 0,
            task_index: 0,
            input_mode: InputMode::Normal,
            theme,
            detail_scroll: 0,
            should_quit: false,
        };
        app.sort_projects();
        Ok(app)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        store::save(&self.data)
    }

    pub fn tick(&mut self) {
        let now = Utc::now();
        let before = self.data.tasks.len();
        self.data.tasks.retain(|t| match t.done_at {
            Some(done_at) => now.signed_duration_since(done_at).num_seconds() < 3600,
            None => true,
        });
        if self.data.tasks.len() != before {
            let count = self.tasks_for_selected_project().len();
            if self.task_index >= count {
                self.task_index = count.saturating_sub(1);
            }
            let _ = self.save();
        }
    }

    fn sort_projects(&mut self) {
        let selected_id = self.selected_project().map(|p| p.id);
        self.data.projects.sort_by(|a, b| match (a.is_inbox, b.is_inbox) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        if let Some(id) = selected_id {
            self.project_index = self
                .data
                .projects
                .iter()
                .position(|p| p.id == id)
                .unwrap_or(0);
        }
    }

    pub fn selected_project(&self) -> Option<&Project> {
        self.data.projects.get(self.project_index)
    }

    pub fn tasks_for_selected_project(&self) -> Vec<&Task> {
        let Some(project) = self.selected_project() else {
            return vec![];
        };
        let mut tasks: Vec<&Task> = self
            .data
            .tasks
            .iter()
            .filter(|t| t.project_id == project.id)
            .collect();
        tasks.sort_by(|a, b| match (a.done, b.done) {
            (false, true) => Ordering::Less,
            (true, false) => Ordering::Greater,
            (true, true) => b.done_at.cmp(&a.done_at),
            (false, false) => match (a.position, b.position) {
                // Both have manual position: sort by position
                (Some(ap), Some(bp)) => ap.cmp(&bp),
                // Manual position wins over auto-sorted
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                // Both auto: sort by due date
                (None, None) => match (a.due_date, b.due_date) {
                    (Some(ad), Some(bd)) => ad.cmp(&bd),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => a.created_at.cmp(&b.created_at),
                },
            },
        });
        tasks
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match &self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::ProjectEdit(_) => self.handle_project_edit_key(key),
            InputMode::TaskEdit(_) => self.handle_task_edit_key(key),
            InputMode::DatePicker(_) => self.handle_date_picker_key(key),
            InputMode::LabelPicker(_) => self.handle_label_picker_key(key),
            InputMode::ChecklistEditor(_) => self.handle_checklist_editor_key(key),
            InputMode::Search(_) => self.handle_search_key(key),
            InputMode::MoveTask(_) => self.handle_move_task_key(key),
            InputMode::ConfirmDelete(_) => self.handle_confirm_delete_key(key),
            InputMode::Help => self.handle_help_key(key),
        }
    }

    // --- Normal Mode ---

    fn handle_normal_key(&mut self, key: KeyEvent) {

        // Detail pane: only scroll and navigate
        if self.active_pane == ActivePane::Detail {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.detail_scroll += 1;
                }
                KeyCode::Tab => self.cycle_pane(false),
                KeyCode::BackTab => self.cycle_pane(true),
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('?') => self.input_mode = InputMode::Help,
                KeyCode::Char('/') => self.open_search(),
                KeyCode::Esc => {
                    self.active_pane = ActivePane::Tasks;
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.input_mode = InputMode::Help,
            KeyCode::Tab => self.cycle_pane(false),
            KeyCode::BackTab => self.cycle_pane(true),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Char('p') if self.active_pane == ActivePane::Tasks => {
                self.toggle_task_pin();
            }
            KeyCode::Char('t') if self.active_pane == ActivePane::Tasks => {
                self.toggle_task_timer();
            }
            KeyCode::Char('n') => self.start_new(),
            KeyCode::Char('e') => self.start_edit(),
            KeyCode::Char('d') => self.start_delete(),
            KeyCode::Char('m') if self.active_pane == ActivePane::Tasks => self.start_move_task(),
            KeyCode::Char('/') => self.open_search(),
            KeyCode::Enter | KeyCode::Char(' ') => self.action_enter(),
            _ => {}
        }
    }

    fn cycle_pane(&mut self, reverse: bool) {
        let has_detail = self
            .selected_project()
            .map(|p| self.data.tasks.iter().any(|t| t.project_id == p.id))
            .unwrap_or(false);

        self.active_pane = if reverse {
            match self.active_pane {
                ActivePane::Projects => {
                    if has_detail {
                        ActivePane::Detail
                    } else {
                        ActivePane::Tasks
                    }
                }
                ActivePane::Tasks => ActivePane::Projects,
                ActivePane::Detail => ActivePane::Tasks,
            }
        } else {
            match self.active_pane {
                ActivePane::Projects => ActivePane::Tasks,
                ActivePane::Tasks => {
                    if has_detail {
                        ActivePane::Detail
                    } else {
                        ActivePane::Projects
                    }
                }
                ActivePane::Detail => ActivePane::Projects,
            }
        };

        if self.active_pane == ActivePane::Detail {
            self.detail_scroll = 0;
        }
    }

    fn move_up(&mut self) {
        match self.active_pane {
            ActivePane::Projects => {
                if self.project_index > 0 {
                    self.project_index -= 1;
                    self.task_index = 0;
                    self.detail_scroll = 0;
                }
            }
            ActivePane::Tasks => {
                if self.task_index > 0 {
                    self.task_index -= 1;
                    self.detail_scroll = 0;
                }
            }
            ActivePane::Detail => {}
        }
    }

    fn move_down(&mut self) {
        match self.active_pane {
            ActivePane::Projects => {
                if self.project_index < self.data.projects.len().saturating_sub(1) {
                    self.project_index += 1;
                    self.task_index = 0;
                    self.detail_scroll = 0;
                }
            }
            ActivePane::Tasks => {
                let count = self.tasks_for_selected_project().len();
                if self.task_index < count.saturating_sub(1) {
                    self.task_index += 1;
                    self.detail_scroll = 0;
                }
            }
            ActivePane::Detail => {}
        }
    }

    fn action_enter(&mut self) {
        match self.active_pane {
            ActivePane::Projects => {
                self.active_pane = ActivePane::Tasks;
                self.task_index = 0;
            }
            ActivePane::Tasks => {
                self.toggle_task_done();
            }
            ActivePane::Detail => {}
        }
    }

    fn start_new(&mut self) {
        match self.active_pane {
            ActivePane::Projects => {
                self.input_mode = InputMode::ProjectEdit(ProjectEditState {
                    input: TextInput::empty(),
                    editing_id: None,
                });
            }
            ActivePane::Tasks => {
                if self.selected_project().is_some() {
                    self.input_mode = InputMode::TaskEdit(TaskEditState {
                        title: TextInput::empty(),
                        description: TextInput::empty(),
                        due_date: None,
                        label_ids: vec![],
                        checklist: vec![],
                        active_field: TaskField::Title,
                        editing_id: None,
                    });
                }
            }
            ActivePane::Detail => {}
        }
    }

    fn start_edit(&mut self) {
        match self.active_pane {
            ActivePane::Projects => {
                if let Some(project) = self.selected_project() {
                    if project.is_inbox {
                        return;
                    }
                    let state = ProjectEditState {
                        input: TextInput::new(project.name.clone()),
                        editing_id: Some(project.id),
                    };
                    self.input_mode = InputMode::ProjectEdit(state);
                }
            }
            ActivePane::Tasks => {
                let task_data = {
                    let tasks = self.tasks_for_selected_project();
                    tasks.get(self.task_index).map(|t| {
                        (
                            t.id,
                            t.title.clone(),
                            t.description.clone(),
                            t.due_date,
                            t.label_ids.clone(),
                            t.checklist.clone(),
                        )
                    })
                };
                if let Some((id, title, description, due_date, label_ids, checklist)) = task_data {
                    self.input_mode = InputMode::TaskEdit(TaskEditState {
                        title: TextInput::new(title),
                        description: TextInput::new(description),
                        due_date,
                        label_ids,
                        checklist,
                        active_field: TaskField::Title,
                        editing_id: Some(id),
                    });
                }
            }
            ActivePane::Detail => {}
        }
    }

    fn start_delete(&mut self) {
        match self.active_pane {
            ActivePane::Projects => {
                if let Some(project) = self.selected_project() {
                    if project.is_inbox {
                        return;
                    }
                    let task_count = self
                        .data
                        .tasks
                        .iter()
                        .filter(|t| t.project_id == project.id)
                        .count();
                    self.input_mode = InputMode::ConfirmDelete(DeleteTarget::Project {
                        id: project.id,
                        name: project.name.clone(),
                        task_count,
                    });
                }
            }
            ActivePane::Tasks => {
                let task_info = {
                    let tasks = self.tasks_for_selected_project();
                    tasks.get(self.task_index).map(|t| (t.id, t.title.clone()))
                };
                if let Some((id, title)) = task_info {
                    self.input_mode = InputMode::ConfirmDelete(DeleteTarget::Task { id, title });
                }
            }
            ActivePane::Detail => {}
        }
    }

    fn start_move_task(&mut self) {
        let Some(current_project) = self.selected_project().map(|p| p.id) else {
            return;
        };
        let task_info = {
            let tasks = self.tasks_for_selected_project();
            tasks.get(self.task_index).map(|t| t.id)
        };
        let Some(task_id) = task_info else {
            return;
        };
        let index = self
            .data
            .projects
            .iter()
            .position(|p| p.id == current_project)
            .unwrap_or(0);
        self.input_mode = InputMode::MoveTask(MoveTaskState { task_id, index });
    }

    fn toggle_task_done(&mut self) {
        let task_id = {
            let tasks = self.tasks_for_selected_project();
            tasks.get(self.task_index).map(|t| t.id)
        };
        if let Some(id) = task_id {
            let now = Utc::now();
            if let Some(task) = self.data.tasks.iter_mut().find(|t| t.id == id) {
                task.done = !task.done;
                task.done_at = if task.done { Some(now) } else { None };
                if task.done {
                    stop_running_session(task, now);
                }
            }
            let _ = self.save();
        }
    }

    fn toggle_task_timer(&mut self) {
        let task_id = {
            let tasks = self.tasks_for_selected_project();
            tasks.get(self.task_index).filter(|t| !t.done).map(|t| t.id)
        };
        let Some(id) = task_id else {
            return;
        };

        let now = Utc::now();
        let is_running = self.is_task_running(id);

        for task in &mut self.data.tasks {
            if task.id == id {
                if is_running {
                    stop_running_session(task, now);
                } else {
                    stop_running_session(task, now);
                    task.time_sessions.push(TimeSession::new_started());
                }
            } else {
                stop_running_session(task, now);
            }
        }

        let _ = self.save();
    }

    fn toggle_task_pin(&mut self) {
        let task_info = {
            let tasks = self.tasks_for_selected_project();
            tasks
                .get(self.task_index)
                .filter(|t| !t.done)
                .map(|t| (t.id, t.pinned))
        };
        let Some((id, currently_pinned)) = task_info else {
            return;
        };

        if currently_pinned {
            // Unpin: return to auto-sort
            if let Some(task) = self.data.tasks.iter_mut().find(|t| t.id == id) {
                task.pinned = false;
                task.position = None;
            }
        } else {
            // Pin to top: find lowest existing position, go one lower
            let project_id = self.selected_project().map(|p| p.id);
            let min_pos = self
                .data
                .tasks
                .iter()
                .filter(|t| !t.done && t.pinned && Some(t.project_id) == project_id)
                .filter_map(|t| t.position)
                .min()
                .unwrap_or(1001);
            let new_pos = min_pos.saturating_sub(1);
            if let Some(task) = self.data.tasks.iter_mut().find(|t| t.id == id) {
                task.pinned = true;
                task.position = Some(new_pos);
            }
        }
        // Follow the task to its new position in the sorted list
        if let Some(new_idx) = self.tasks_for_selected_project().iter().position(|t| t.id == id) {
            self.task_index = new_idx;
        }
        let _ = self.save();
    }

    // --- Project Edit Mode ---

    fn handle_project_edit_key(&mut self, key: KeyEvent) {
        if is_save(key) || key.code == KeyCode::Enter {
            self.save_project();
            return;
        }
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {
                if let InputMode::ProjectEdit(ref mut state) = self.input_mode {
                    apply_text_input(&mut state.input, key);
                }
            }
        }
    }

    fn save_project(&mut self) {
        let project_data = if let InputMode::ProjectEdit(ref state) = self.input_mode {
            let name = state.input.text.trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some((name, state.editing_id))
            }
        } else {
            None
        };

        if let Some((name, editing_id)) = project_data {
            if let Some(id) = editing_id {
                if let Some(project) = self.data.projects.iter_mut().find(|p| p.id == id) {
                    project.name = name;
                }
            } else {
                let project = Project::new(name, false);
                self.data.projects.push(project);
            }
            self.sort_projects();
            let _ = self.save();
        }
        self.input_mode = InputMode::Normal;
    }

    // --- Task Edit Mode ---

    fn handle_task_edit_key(&mut self, key: KeyEvent) {
        if is_save(key) {
            self.save_task();
            return;
        }
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                self.task_edit_enter();
            }
            KeyCode::Tab => {
                self.task_edit_cycle_field(false);
            }
            KeyCode::BackTab => {
                self.task_edit_cycle_field(true);
            }
            _ => {
                if let InputMode::TaskEdit(ref mut state) = self.input_mode {
                    match state.active_field {
                        TaskField::Title => apply_text_input(&mut state.title, key),
                        TaskField::Description => match key.code {
                            KeyCode::Up => move_cursor_up(&mut state.description),
                            KeyCode::Down => move_cursor_down(&mut state.description),
                            _ => apply_text_input(&mut state.description, key),
                        },
                        TaskField::DueDate => {
                            if matches!(key.code, KeyCode::Backspace | KeyCode::Delete) {
                                state.due_date = None;
                            }
                        }
                        TaskField::Labels | TaskField::Checklist => {}
                    }
                }
            }
        }
    }

    fn task_edit_enter(&mut self) {
        let field = match &self.input_mode {
            InputMode::TaskEdit(s) => s.active_field,
            _ => return,
        };

        // Description: Enter inserts a newline
        if field == TaskField::Description {
            if let InputMode::TaskEdit(ref mut state) = self.input_mode {
                state.description.insert_char('\n');
            }
            return;
        }

        match field {
            TaskField::DueDate => {
                if let InputMode::TaskEdit(task_state) =
                    std::mem::replace(&mut self.input_mode, InputMode::Normal)
                {
                    let date = task_state
                        .due_date
                        .unwrap_or_else(|| Local::now().date_naive());
                    self.input_mode = InputMode::DatePicker(DatePickerState {
                        selected: date,
                        year: date.year(),
                        month: date.month(),
                        task_state,
                    });
                }
            }
            TaskField::Labels => {
                if let InputMode::TaskEdit(task_state) =
                    std::mem::replace(&mut self.input_mode, InputMode::Normal)
                {
                    let assigned = task_state.label_ids.clone();
                    self.input_mode = InputMode::LabelPicker(LabelPickerState {
                        index: 0,
                        assigned,
                        task_state,
                        creating: None,
                    });
                }
            }
            TaskField::Checklist => {
                if let InputMode::TaskEdit(task_state) =
                    std::mem::replace(&mut self.input_mode, InputMode::Normal)
                {
                    let items = task_state.checklist.clone();
                    self.input_mode = InputMode::ChecklistEditor(ChecklistEditorState {
                        index: 0,
                        items,
                        task_state,
                        editing: None,
                    });
                }
            }
            _ => {
                self.save_task();
            }
        }
    }

    fn task_edit_cycle_field(&mut self, reverse: bool) {
        if let InputMode::TaskEdit(ref mut state) = self.input_mode {
            state.active_field = if reverse {
                match state.active_field {
                    TaskField::Title => TaskField::Checklist,
                    TaskField::Description => TaskField::Title,
                    TaskField::DueDate => TaskField::Description,
                    TaskField::Labels => TaskField::DueDate,
                    TaskField::Checklist => TaskField::Labels,
                }
            } else {
                match state.active_field {
                    TaskField::Title => TaskField::Description,
                    TaskField::Description => TaskField::DueDate,
                    TaskField::DueDate => TaskField::Labels,
                    TaskField::Labels => TaskField::Checklist,
                    TaskField::Checklist => TaskField::Title,
                }
            };
        }
    }

    fn save_task(&mut self) {
        let task_data = if let InputMode::TaskEdit(ref state) = self.input_mode {
            let title = state.title.text.trim().to_string();
            if title.is_empty() {
                None
            } else {
                Some((
                    title,
                    state.description.text.trim().to_string(),
                    state.due_date,
                    state.label_ids.clone(),
                    state.checklist.clone(),
                    state.editing_id,
                ))
            }
        } else {
            None
        };

        if let Some((title, description, due_date, label_ids, checklist, editing_id)) = task_data {
            if let Some(id) = editing_id {
                if let Some(task) = self.data.tasks.iter_mut().find(|t| t.id == id) {
                    task.title = title;
                    task.description = description;
                    task.due_date = due_date;
                    task.label_ids = label_ids;
                    task.checklist = checklist;
                }
            } else if let Some(project_id) = self.selected_project().map(|p| p.id) {
                let mut task = Task::new(project_id, title);
                task.description = description;
                task.due_date = due_date;
                task.label_ids = label_ids;
                task.checklist = checklist;
                self.data.tasks.push(task);
            }
            let _ = self.save();
        }
        self.input_mode = InputMode::Normal;
    }

    // --- Date Picker Mode ---

    fn handle_date_picker_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                if let InputMode::DatePicker(state) =
                    std::mem::replace(&mut self.input_mode, InputMode::Normal)
                {
                    self.input_mode = InputMode::TaskEdit(state.task_state);
                }
            }
            KeyCode::Enter => {
                if let InputMode::DatePicker(mut state) =
                    std::mem::replace(&mut self.input_mode, InputMode::Normal)
                {
                    state.task_state.due_date = Some(state.selected);
                    self.input_mode = InputMode::TaskEdit(state.task_state);
                }
            }
            KeyCode::Backspace => {
                if let InputMode::DatePicker(mut state) =
                    std::mem::replace(&mut self.input_mode, InputMode::Normal)
                {
                    state.task_state.due_date = None;
                    self.input_mode = InputMode::TaskEdit(state.task_state);
                }
            }
            _ => {
                if let InputMode::DatePicker(ref mut state) = self.input_mode {
                    match key.code {
                        KeyCode::Left => {
                            if let Some(d) = state.selected.pred_opt() {
                                state.selected = d;
                                state.year = d.year();
                                state.month = d.month();
                            }
                        }
                        KeyCode::Right => {
                            if let Some(d) = state.selected.succ_opt() {
                                state.selected = d;
                                state.year = d.year();
                                state.month = d.month();
                            }
                        }
                        KeyCode::Up => {
                            if let Some(d) =
                                state.selected.checked_sub_signed(chrono::Duration::days(7))
                            {
                                state.selected = d;
                                state.year = d.year();
                                state.month = d.month();
                            }
                        }
                        KeyCode::Down => {
                            if let Some(d) =
                                state.selected.checked_add_signed(chrono::Duration::days(7))
                            {
                                state.selected = d;
                                state.year = d.year();
                                state.month = d.month();
                            }
                        }
                        KeyCode::Char('<') | KeyCode::PageUp => {
                            if state.month == 1 {
                                state.month = 12;
                                state.year -= 1;
                            } else {
                                state.month -= 1;
                            }
                            let max_day = days_in_month(state.year, state.month);
                            let day = state.selected.day().min(max_day);
                            if let Some(d) =
                                NaiveDate::from_ymd_opt(state.year, state.month, day)
                            {
                                state.selected = d;
                            }
                        }
                        KeyCode::Char('>') | KeyCode::PageDown => {
                            if state.month == 12 {
                                state.month = 1;
                                state.year += 1;
                            } else {
                                state.month += 1;
                            }
                            let max_day = days_in_month(state.year, state.month);
                            let day = state.selected.day().min(max_day);
                            if let Some(d) =
                                NaiveDate::from_ymd_opt(state.year, state.month, day)
                            {
                                state.selected = d;
                            }
                        }
                        KeyCode::Char('t') => {
                            state.selected = Local::now().date_naive();
                            state.year = state.selected.year();
                            state.month = state.selected.month();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // --- Label Picker Mode ---

    fn handle_label_picker_key(&mut self, key: KeyEvent) {
        let is_creating = matches!(
            &self.input_mode,
            InputMode::LabelPicker(s) if s.creating.is_some()
        );
        if is_creating {
            self.handle_label_create_key(key);
            return;
        }

        if is_save(key) || key.code == KeyCode::Enter {
            if let InputMode::LabelPicker(mut state) =
                std::mem::replace(&mut self.input_mode, InputMode::Normal)
            {
                state.task_state.label_ids = state.assigned;
                self.input_mode = InputMode::TaskEdit(state.task_state);
            }
            return;
        }

        match key.code {
            KeyCode::Esc => {
                if let InputMode::LabelPicker(state) =
                    std::mem::replace(&mut self.input_mode, InputMode::Normal)
                {
                    self.input_mode = InputMode::TaskEdit(state.task_state);
                }
            }
            _ => {
                if let InputMode::LabelPicker(ref mut state) = self.input_mode {
                    let label_count = self.data.labels.len();
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.index > 0 {
                                state.index -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.index < label_count.saturating_sub(1) {
                                state.index += 1;
                            }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(label) = self.data.labels.get(state.index) {
                                let id = label.id;
                                if state.assigned.contains(&id) {
                                    state.assigned.retain(|&x| x != id);
                                } else {
                                    state.assigned.push(id);
                                }
                            }
                        }
                        KeyCode::Char('n') => {
                            state.creating = Some(LabelCreateState {
                                name: TextInput::empty(),
                                color_index: 0,
                                active_field: LabelCreateField::Name,
                            });
                        }
                        KeyCode::Char('d') => {
                            if let Some(label) = self.data.labels.get(state.index) {
                                let id = label.id;
                                self.data.labels.remove(state.index);
                                state.assigned.retain(|&x| x != id);
                                for task in &mut self.data.tasks {
                                    task.label_ids.retain(|&x| x != id);
                                }
                                if state.index >= self.data.labels.len() && state.index > 0 {
                                    state.index -= 1;
                                }
                                let _ = self.save();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_label_create_key(&mut self, key: KeyEvent) {
        if is_save(key) || key.code == KeyCode::Enter {
            let new_label = if let InputMode::LabelPicker(ref state) = self.input_mode {
                state.creating.as_ref().and_then(|c| {
                    let name = c.name.text.trim().to_string();
                    if name.is_empty() {
                        None
                    } else {
                        Some(Label::new(
                            name,
                            LABEL_COLOR_NAMES[c.color_index].to_string(),
                        ))
                    }
                })
            } else {
                None
            };
            if let Some(label) = new_label {
                self.data.labels.push(label);
                let _ = self.save();
            }
            if let InputMode::LabelPicker(ref mut state) = self.input_mode {
                state.creating = None;
                state.index = self.data.labels.len().saturating_sub(1);
            }
            return;
        }
        match key.code {
            KeyCode::Esc => {
                if let InputMode::LabelPicker(ref mut state) = self.input_mode {
                    state.creating = None;
                }
            }
            KeyCode::Tab | KeyCode::BackTab => {
                if let InputMode::LabelPicker(ref mut state) = self.input_mode {
                    if let Some(ref mut creating) = state.creating {
                        creating.active_field = match creating.active_field {
                            LabelCreateField::Name => LabelCreateField::Color,
                            LabelCreateField::Color => LabelCreateField::Name,
                        };
                    }
                }
            }
            _ => {
                if let InputMode::LabelPicker(ref mut state) = self.input_mode {
                    if let Some(ref mut creating) = state.creating {
                        match creating.active_field {
                            LabelCreateField::Name => {
                                apply_text_input(&mut creating.name, key);
                            }
                            LabelCreateField::Color => match key.code {
                                KeyCode::Left => {
                                    if creating.color_index > 0 {
                                        creating.color_index -= 1;
                                    }
                                }
                                KeyCode::Right => {
                                    if creating.color_index < LABEL_COLOR_NAMES.len() - 1 {
                                        creating.color_index += 1;
                                    }
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }
        }
    }

    // --- Checklist Editor Mode ---

    fn handle_checklist_editor_key(&mut self, key: KeyEvent) {
        let is_editing = matches!(
            &self.input_mode,
            InputMode::ChecklistEditor(s) if s.editing.is_some()
        );
        if is_editing {
            self.handle_checklist_text_key(key);
            return;
        }

        if is_save(key) {
            if let InputMode::ChecklistEditor(mut state) =
                std::mem::replace(&mut self.input_mode, InputMode::Normal)
            {
                state.task_state.checklist = state.items;
                self.input_mode = InputMode::TaskEdit(state.task_state);
            }
            return;
        }

        match key.code {
            KeyCode::Esc => {
                if let InputMode::ChecklistEditor(mut state) =
                    std::mem::replace(&mut self.input_mode, InputMode::Normal)
                {
                    state.task_state.checklist = state.items;
                    self.input_mode = InputMode::TaskEdit(state.task_state);
                }
            }
            _ => {
                if let InputMode::ChecklistEditor(ref mut state) = self.input_mode {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.index > 0 {
                                state.index -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.index < state.items.len().saturating_sub(1) {
                                state.index += 1;
                            }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(item) = state.items.get_mut(state.index) {
                                item.done = !item.done;
                            }
                        }
                        KeyCode::Char('n') => {
                            let new_item = ChecklistItem::new(String::new());
                            state.items.push(new_item);
                            state.index = state.items.len() - 1;
                            state.editing = Some(TextInput::empty());
                        }
                        KeyCode::Char('e') => {
                            if let Some(item) = state.items.get(state.index) {
                                state.editing = Some(TextInput::new(item.text.clone()));
                            }
                        }
                        KeyCode::Char('d') => {
                            if !state.items.is_empty() {
                                state.items.remove(state.index);
                                if state.index >= state.items.len() && state.index > 0 {
                                    state.index -= 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_checklist_text_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                if let InputMode::ChecklistEditor(ref mut state) = self.input_mode {
                    if let Some(input) = state.editing.take() {
                        let text = input.text.trim().to_string();
                        if text.is_empty() {
                            if state.index < state.items.len()
                                && state.items[state.index].text.is_empty()
                            {
                                state.items.remove(state.index);
                                if state.index >= state.items.len() && state.index > 0 {
                                    state.index -= 1;
                                }
                            }
                        } else if let Some(item) = state.items.get_mut(state.index) {
                            item.text = text;
                        }
                    }
                }
            }
            KeyCode::Esc => {
                if let InputMode::ChecklistEditor(ref mut state) = self.input_mode {
                    if let Some(ref input) = state.editing {
                        if input.text.is_empty()
                            && state.index < state.items.len()
                            && state.items[state.index].text.is_empty()
                        {
                            state.items.remove(state.index);
                            if state.index >= state.items.len() && state.index > 0 {
                                state.index -= 1;
                            }
                        }
                    }
                    state.editing = None;
                }
            }
            _ => {
                if let InputMode::ChecklistEditor(ref mut state) = self.input_mode {
                    if let Some(ref mut input) = state.editing {
                        apply_text_input(input, key);
                    }
                }
            }
        }
    }

    // --- Move Task Mode ---

    fn handle_move_task_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                self.confirm_move_task();
            }
            _ => {
                if let InputMode::MoveTask(ref mut state) = self.input_mode {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.index > 0 {
                                state.index -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.index < self.data.projects.len().saturating_sub(1) {
                                state.index += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn confirm_move_task(&mut self) {
        let (task_id, dest_project_id) = match &self.input_mode {
            InputMode::MoveTask(state) => self
                .data
                .projects
                .get(state.index)
                .map(|p| (state.task_id, p.id)),
            _ => None,
        }
        .unwrap_or_else(|| {
            self.input_mode = InputMode::Normal;
            (Uuid::nil(), Uuid::nil())
        });

        if task_id.is_nil() || dest_project_id.is_nil() {
            return;
        }

        if let Some(task) = self.data.tasks.iter_mut().find(|t| t.id == task_id) {
            task.project_id = dest_project_id;
            task.pinned = false;
            task.position = None;
        }

        if let Some(pi) = self.data.projects.iter().position(|p| p.id == dest_project_id) {
            self.project_index = pi;
        }
        if let Some(ti) = self.tasks_for_selected_project().iter().position(|t| t.id == task_id) {
            self.task_index = ti;
        } else {
            self.task_index = 0;
        }
        self.active_pane = ActivePane::Tasks;
        self.input_mode = InputMode::Normal;
        self.detail_scroll = 0;
        let _ = self.save();
    }

    // --- Confirm Delete Mode ---

    fn handle_confirm_delete_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                self.confirm_delete();
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
    }

    fn confirm_delete(&mut self) {
        enum Action {
            Project(Uuid),
            Task(Uuid),
        }
        let action = match &self.input_mode {
            InputMode::ConfirmDelete(DeleteTarget::Project { id, .. }) => {
                Some(Action::Project(*id))
            }
            InputMode::ConfirmDelete(DeleteTarget::Task { id, .. }) => Some(Action::Task(*id)),
            _ => None,
        };
        if let Some(action) = action {
            match action {
                Action::Project(id) => {
                    self.data.tasks.retain(|t| t.project_id != id);
                    self.data.projects.retain(|p| p.id != id);
                    if self.project_index >= self.data.projects.len() {
                        self.project_index = self.data.projects.len().saturating_sub(1);
                    }
                    self.task_index = 0;
                }
                Action::Task(id) => {
                    self.data.tasks.retain(|t| t.id != id);
                    let count = self.tasks_for_selected_project().len();
                    if self.task_index >= count {
                        self.task_index = count.saturating_sub(1);
                    }
                }
            }
            let _ = self.save();
        }
        self.input_mode = InputMode::Normal;
    }

    // --- Search Mode ---

    fn open_search(&mut self) {
        let results = self.compute_search_results("");
        self.input_mode = InputMode::Search(SearchState {
            query: TextInput::empty(),
            results,
            selected: 0,
        });
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                self.jump_to_search_result();
            }
            KeyCode::Up => {
                if let InputMode::Search(ref mut state) = self.input_mode {
                    if state.selected > 0 {
                        state.selected -= 1;
                    }
                }
            }
            KeyCode::Down => {
                if let InputMode::Search(ref mut state) = self.input_mode {
                    if state.selected < state.results.len().saturating_sub(1) {
                        state.selected += 1;
                    }
                }
            }
            _ => {
                if let InputMode::Search(ref mut state) = self.input_mode {
                    apply_text_input(&mut state.query, key);
                }
                let query = match &self.input_mode {
                    InputMode::Search(s) => s.query.text.clone(),
                    _ => return,
                };
                let results = self.compute_search_results(&query);
                if let InputMode::Search(ref mut state) = self.input_mode {
                    state.results = results;
                    state.selected = 0;
                }
            }
        }
    }

    fn compute_search_results(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for task in &self.data.tasks {
            let project_name = self
                .data
                .projects
                .iter()
                .find(|p| p.id == task.project_id)
                .map(|p| p.name.clone())
                .unwrap_or_default();

            if query.is_empty() {
                results.push(SearchResult {
                    task_id: task.id,
                    project_id: task.project_id,
                    title: task.title.clone(),
                    project_name,
                    due_date: task.due_date,
                    done: task.done,
                    label_ids: task.label_ids.clone(),
                    matched_indices: vec![],
                });
            } else {
                let title_match = fuzzy_match(query, &task.title);
                let desc_match = if query.len() >= 2 {
                    fuzzy_match(query, &task.description)
                } else {
                    None
                };

                if let Some(m) = title_match {
                    results.push(SearchResult {
                        task_id: task.id,
                        project_id: task.project_id,
                        title: task.title.clone(),
                        project_name,
                        due_date: task.due_date,
                        done: task.done,
                        label_ids: task.label_ids.clone(),
                        matched_indices: m.matched_indices,
                    });
                } else if let Some(_m) = desc_match {
                    results.push(SearchResult {
                        task_id: task.id,
                        project_id: task.project_id,
                        title: task.title.clone(),
                        project_name,
                        due_date: task.due_date,
                        done: task.done,
                        label_ids: task.label_ids.clone(),
                        matched_indices: vec![],
                    });
                }
            }
        }

        if query.is_empty() {
            // Sort undone first, then by due date
            results.sort_by(|a, b| {
                a.done.cmp(&b.done).then_with(|| a.due_date.cmp(&b.due_date))
            });
        } else {
            // Sort by match quality (more matched_indices chars = better, prefer shorter titles)
            results.sort_by(|a, b| {
                let sa = a.matched_indices.len() as i32 * 10
                    - a.title.len() as i32
                    + if a.done { -50 } else { 0 };
                let sb = b.matched_indices.len() as i32 * 10
                    - b.title.len() as i32
                    + if b.done { -50 } else { 0 };
                sb.cmp(&sa)
            });
        }

        results.truncate(15);
        results
    }

    fn jump_to_search_result(&mut self) {
        let target = match &self.input_mode {
            InputMode::Search(s) => s.results.get(s.selected).map(|r| (r.task_id, r.project_id)),
            _ => None,
        };

        let Some((task_id, project_id)) = target else {
            self.input_mode = InputMode::Normal;
            return;
        };

        if let Some(pi) = self.data.projects.iter().position(|p| p.id == project_id) {
            self.project_index = pi;
        }

        let tasks = self.tasks_for_selected_project();
        if let Some(ti) = tasks.iter().position(|t| t.id == task_id) {
            self.task_index = ti;
        }

        self.active_pane = ActivePane::Tasks;
        self.input_mode = InputMode::Normal;
    }

    pub fn is_task_running(&self, task_id: Uuid) -> bool {
        self.data
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .map(|t| t.time_sessions.iter().any(|s| s.ended_at.is_none()))
            .unwrap_or(false)
    }

    pub fn task_tracked_seconds(&self, task_id: Uuid) -> i64 {
        let now = Utc::now();
        self.data
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .map(|t| {
                t.time_sessions
                    .iter()
                    .map(|s| {
                        let end = s.ended_at.unwrap_or(now);
                        (end - s.started_at).num_seconds().max(0)
                    })
                    .sum()
            })
            .unwrap_or(0)
    }

    pub fn running_task_summary(&self) -> Option<(String, i64)> {
        let now = Utc::now();
        self.data.tasks.iter().find_map(|task| {
            task.time_sessions
                .iter()
                .any(|s| s.ended_at.is_none())
                .then(|| {
                    let seconds = task
                        .time_sessions
                        .iter()
                        .map(|s| {
                            let end = s.ended_at.unwrap_or(now);
                            (end - s.started_at).num_seconds().max(0)
                        })
                        .sum();
                    (task.title.clone(), seconds)
                })
        })
    }

    // --- Help Mode ---

    fn handle_help_key(&mut self, _key: KeyEvent) {
        self.input_mode = InputMode::Normal;
    }

    // --- Paste ---

    pub fn handle_paste(&mut self, text: &str) {
        match &mut self.input_mode {
            InputMode::ProjectEdit(state) => {
                paste_into(&mut state.input, text, false);
            }
            InputMode::TaskEdit(state) => match state.active_field {
                TaskField::Title => paste_into(&mut state.title, text, false),
                TaskField::Description => paste_into(&mut state.description, text, true),
                _ => {}
            },
            InputMode::Search(state) => {
                paste_into(&mut state.query, text, false);
            }
            InputMode::LabelPicker(state) => {
                if let Some(ref mut creating) = state.creating {
                    if creating.active_field == LabelCreateField::Name {
                        paste_into(&mut creating.name, text, false);
                    }
                }
            }
            InputMode::ChecklistEditor(state) => {
                if let Some(ref mut input) = state.editing {
                    paste_into(input, text, false);
                }
            }
            _ => {}
        }
        // Recompute search results after paste
        if let InputMode::Search(_) = &self.input_mode {
            let query = match &self.input_mode {
                InputMode::Search(s) => s.query.text.clone(),
                _ => return,
            };
            let results = self.compute_search_results(&query);
            if let InputMode::Search(ref mut s) = self.input_mode {
                s.results = results;
                s.selected = 0;
            }
        }
    }
}

// --- Utility ---

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next.unwrap().pred_opt().unwrap().day()
}

pub struct FuzzyMatch {
    pub matched_indices: Vec<usize>,
}

pub fn stop_running_session(task: &mut Task, now: chrono::DateTime<Utc>) {
    if let Some(session) = task.time_sessions.iter_mut().rev().find(|s| s.ended_at.is_none()) {
        session.ended_at = Some(now);
    }
}

pub fn format_duration_compact(total_seconds: i64) -> String {
    let total_seconds = total_seconds.max(0);
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if hours > 0 {
        format!("{}h {:02}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn fuzzy_match(query: &str, target: &str) -> Option<FuzzyMatch> {
    let query_chars: Vec<char> = query.to_lowercase().chars().collect();
    let target_chars: Vec<char> = target.to_lowercase().chars().collect();

    if query_chars.is_empty() {
        return Some(FuzzyMatch {
            matched_indices: vec![],
        });
    }

    let mut matched = Vec::new();
    let mut qi = 0;

    for (ti, &tc) in target_chars.iter().enumerate() {
        if qi < query_chars.len() && tc == query_chars[qi] {
            matched.push(ti);
            qi += 1;
        }
    }

    if qi == query_chars.len() {
        Some(FuzzyMatch {
            matched_indices: matched,
        })
    } else {
        None
    }
}
