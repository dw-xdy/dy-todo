use chrono::{DateTime, Utc};
use std::collections::HashSet;

// 任务状态（系统内置，不可自定义）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Completed, // ✅ 已完成
    Todo,      // 🔲未完成/待办
    Overdue,   // 🔴 已逾期/过期
    DueToday,  // 🟡 今日到期
}

impl TaskStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Completed => "✅",
            Self::Todo => "🔲",
            Self::Overdue => "🔴",
            Self::DueToday => "🟡",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    name: String,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct TodoTask {
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub tags: HashSet<Tag>,
    pub created_at: DateTime<Utc>,          // 创建时间
    pub due_date: Option<DateTime<Utc>>,    // 截止日期
    pub finish_date: Option<DateTime<Utc>>, // 完成日期
}

impl TodoTask {
    pub fn new(title: String, description: String) -> Self {
        Self {
            title,
            description,
            status: TaskStatus::Todo,
            tags: HashSet::new(),
            created_at: Utc::now(),
            due_date: None,
            finish_date: None,
        }
    }

    pub fn add_tag(&mut self, tag_name: String) {
        self.tags.insert(Tag::new(tag_name));
    }

    pub fn remove_tag(&mut self, tag_name: &str) {
        self.tags.retain(|tag| tag.name() != tag_name);
    }

    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
    }

    pub fn set_due_date(&mut self, due_date: DateTime<Utc>) {
        self.due_date = Some(due_date);
        self.update_status();
    }

    pub fn update_status(&mut self) {
        if self.status == TaskStatus::Completed {
            return;
        }

        let now = Utc::now();

        if let Some(due) = self.due_date {
            if due < now {
                self.status = TaskStatus::Overdue;
            } else if due.date_naive() == now.date_naive() {
                self.status = TaskStatus::DueToday;
            } else {
                self.status = TaskStatus::Todo;
            }
        } else {
            self.status = TaskStatus::Todo;
        }
    }
}
