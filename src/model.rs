use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub is_inbox: bool,
}

impl Project {
    pub fn new(name: String, is_inbox: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            created_at: Utc::now(),
            is_inbox,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

impl Label {
    pub fn new(name: String, color: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub text: String,
    pub done: bool,
}

impl ChecklistItem {
    pub fn new(text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            text,
            done: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: String,
    pub due_date: Option<NaiveDate>,
    pub done: bool,
    pub done_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub label_ids: Vec<Uuid>,
    #[serde(default)]
    pub checklist: Vec<ChecklistItem>,
    /// Manual sort position. `None` = auto-sort by due date.
    #[serde(default)]
    pub position: Option<u32>,
    /// True only if the user explicitly moved this task. Controls ↕ indicator.
    #[serde(default)]
    pub pinned: bool,
}

impl Task {
    pub fn new(project_id: Uuid, title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            title,
            description: String::new(),
            due_date: None,
            done: false,
            done_at: None,
            created_at: Utc::now(),
            label_ids: vec![],
            checklist: vec![],
            position: None,
            pinned: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppData {
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub labels: Vec<Label>,
}

pub const LABEL_COLOR_NAMES: &[&str] = &[
    "red", "orange", "yellow", "green", "blue", "purple", "pink", "teal",
];

pub fn label_color_rgb(name: &str) -> (u8, u8, u8) {
    match name {
        "red" => (0xe5, 0x53, 0x53),
        "orange" => (0xf0, 0x96, 0x45),
        "yellow" => (0xe5, 0xc0, 0x4b),
        "green" => (0x4e, 0xb5, 0x5b),
        "blue" => (0x4b, 0x91, 0xe5),
        "purple" => (0x9b, 0x6b, 0xdf),
        "pink" => (0xe5, 0x4b, 0x8a),
        "teal" => (0x4b, 0xb5, 0xaf),
        _ => (0x88, 0x88, 0x88),
    }
}
