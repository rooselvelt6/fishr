use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishType {
    pub id: String,
    pub name: String,
    pub species: String,
    pub category: FishCategory,
    pub description: String,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum FishCategory {
    #[default]
    White,
    Blue,
    Shellfish,
    Crustacean,
    Other,
}

impl FishCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FishCategory::White => "White",
            FishCategory::Blue => "Blue",
            FishCategory::Shellfish => "Shellfish",
            FishCategory::Crustacean => "Crustacean",
            FishCategory::Other => "Other",
        }
    }
}

impl FishType {
    pub fn new(name: String, species: String, category: FishCategory, description: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            name,
            species,
            category,
            description,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
