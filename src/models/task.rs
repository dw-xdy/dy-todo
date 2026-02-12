use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    Work,
    Personal,
    Urgent,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct TodoTask {
    pub title: String,
    pub description: String,
    pub is_completed: bool,
    pub tags: HashSet<Tag>,
}

impl TodoTask {
    pub fn new(title: String, description: String) -> Self {
        Self {
            title,
            description,
            is_completed: false,
            tags: HashSet::new(),
        }
    }
}
