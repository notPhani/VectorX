// We will develop CRUD operations on projects in this file such as : Add, Delete, Edit, List, Search, Open, Star, Import, Export, Rebuild Index, Validate Project, Update Last Opened
// So, each project is a folder with the following structure :  Project_ID (UUID) -> project.json, Scratch, Papers, Writings, Config, Vectors 
// project.json will contain the metadata of the project such as : name, description, created_at, updated_at, last_opened_at, starred, tags, version number etc.
// Scratch will contain the nodes info and layout, Papers will contain the database of papers related to the Project, Writings will contain the database of writings related to the Project, Config will contain the configuration of the project such as : embedding model, vector database, etc. Vectors will contain the vector files of the project.
// And we will have one global file called projects.json which will contain the list of all projects with their metadata for easy access and management that will be loaded into the AppState on startup for listing and searching projects.
// Each CRUD operation will be implemented as a function in this file that will manipulate the project folder and files accordingly and update the projects.json file as needed.
// We will also implement functions for importing and exporting projects as zip files, rebuilding the index of the project, validating the project structure and files, and updating the last opened timestamp of the project.
// for now Open function will be a trigger that will load the project into the AppState and set it as the current project else it will be defaulted to latest opened project on startup, and we will implement the actual loading of the project data into the AppState in a separate function that will be called by the Open function.
/*
User/AppData/VectorX/
├── project_registry.json (The Global Registry)
└── Projects/
    └── [UUID]/
        ├── project_metadata.json (Metadata: name, tags, etc.)
        ├── Scratch/ (Nodes & Layout)
        ├── Papers/ (PDFs & Metadata)
        ├── Writings/ (Drafts & Markdown)
        ├── Config/ (LLM & Embedder settings)
        ├── Vectors/ (RAG Index)
        └── .lock (Presence indicator) 
*/
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use crate::AppState;

// --- Structs & Enums ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BannerMetadata {
    pub seed: u64,
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectMetadata {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub banner: BannerMetadata, 
    pub created_at: DateTime<Utc>, 
    pub updated_at: DateTime<Utc>,
    pub last_opened_at: DateTime<Utc>,
    pub starred: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectRegistry {
    pub projects: Vec<ProjectMetadata>,
}

#[derive(Debug, Clone, Copy)]
pub enum ProjectFolder {
    Scratch,
    Papers,
    Writings,
    Config,
    Vectors,
}

impl ProjectFolder {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectFolder::Scratch => "Scratch",
            ProjectFolder::Papers => "Papers",
            ProjectFolder::Writings => "Writings",
            ProjectFolder::Config => "Config",
            ProjectFolder::Vectors => "Vectors",
        }
    }

    pub fn all() -> &'static [ProjectFolder] {
        &[
            ProjectFolder::Scratch,
            ProjectFolder::Papers,
            ProjectFolder::Writings,
            ProjectFolder::Config,
            ProjectFolder::Vectors,
        ]
    }
}

// --- The CRUD Command ---
#[tauri::command]
pub fn create_project(
    name: String, 
    description: Option<String>, 
    banner: BannerMetadata, 
    starred: bool,
    state: State<'_, AppState> // Injected by Tauri
) -> Result<ProjectMetadata, String> {
    
    let safe_name = name.trim();
    if safe_name.is_empty(){
        return Err("Project name cannot be empty".into());
    }

    let project_id = Uuid::new_v4();
    let now = Utc::now();
    
    let metadata = ProjectMetadata {
        id: project_id,
        name: safe_name.to_string(),
        description,
        banner,
        created_at: now,
        updated_at: now,
        last_opened_at: now,
        starred,
    };

    // 1. Create the projects folder if it doesn't exist
    let projects_root = state.root.join("Projects");

    fs::create_dir_all(&projects_root).map_err(|e| e.to_string())?;

    let project_path = projects_root.join(project_id.to_string());

    fs::create_dir_all(&project_path).map_err(|e| e.to_string())?;

    // 2. Scaffold subfolders safely
    for folder in ProjectFolder::all() {
        fs::create_dir_all(project_path.join(folder.as_str())).map_err(|e| e.to_string())?;
    }

    // 3. Create and write the project_metadata.json file (Atomic Write)
    let metadata_path = project_path.join("project_metadata.json");
    let metadata_tmp_path = project_path.join("project_metadata.tmp");
    
    let metadata_bytes = serde_json::to_vec_pretty(&metadata).map_err(|e| e.to_string())?;
    
    fs::write(&metadata_tmp_path, &metadata_bytes).map_err(|e| e.to_string())?;
    fs::rename(&metadata_tmp_path, &metadata_path).map_err(|e| e.to_string())?;

    // 4. Update Global Registry (Atomic Write)
    let mut registry = state.registry.lock().map_err(|_| "Failed to lock registry")?;
    registry.projects.insert(0, metadata.clone()); // Insert at top of list

    let registry_path = state.root.join("project_registry.json");
    if !registry_path.exists() {
        fs::write(&registry_path, b"{\"projects\": []}")
            .map_err(|e| e.to_string())?;
    }
    let registry_tmp_path = state.root.join("project_registry.tmp");

    let registry_bytes = serde_json::to_vec_pretty(&*registry).map_err(|e| e.to_string())?;
    
    fs::write(&registry_tmp_path, &registry_bytes).map_err(|e| e.to_string())?;
    fs::rename(&registry_tmp_path, &registry_path).map_err(|e| e.to_string())?;

    Ok(metadata)
}