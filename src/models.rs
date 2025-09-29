use crate::utils::get_projects_db;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;

#[derive(Serialize, Deserialize)]
pub struct Projects {
    pub current: Option<String>,
    #[serde(flatten)]
    pub projects: HashMap<String, String>,
}

pub fn ensure_projects_db() -> Result<()> {
    let db_path = get_projects_db();
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if !db_path.exists() {
        let projects = Projects {
            current: None,
            projects: HashMap::new(),
        };
        let json = serde_json::to_string_pretty(&projects)?;
        fs::write(&db_path, json)?;
    }
    Ok(())
}

pub fn load_projects() -> Result<Projects> {
    let db_path = get_projects_db();
    let mut file = std::fs::File::open(&db_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let projects: Projects = serde_json::from_str(&contents)?;
    Ok(projects)
}

pub fn save_projects(projects: &Projects) -> Result<()> {
    let db_path = get_projects_db();
    let json = serde_json::to_string_pretty(projects)?;
    fs::write(&db_path, json)?;
    Ok(())
}
