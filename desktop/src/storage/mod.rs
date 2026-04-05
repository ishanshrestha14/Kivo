use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const WORKSPACE_FILE_NAME: &str = "workspace.json";
const COLLECTION_CONFIG_FILE_NAME: &str = "collection.json";


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVarsResult {
    pub workspace: Vec<EnvVar>,
    pub collection: Vec<EnvVar>,
    pub merged: HashMap<String, String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CollectionScripts {
    #[serde(default)]
    pub pre_request: String,
    #[serde(default)]
    pub post_response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionConfig {
    #[serde(default)]
    pub default_headers: Vec<KeyValueRow>,
    #[serde(default = "default_auth_record")]
    pub default_auth: AuthRecord,
    #[serde(default)]
    pub scripts: CollectionScripts,
}

fn default_auth_record() -> AuthRecord {
    AuthRecord { auth_type: "none".to_string(), token: String::new() }
}

impl Default for CollectionConfig {
    fn default() -> Self {
        CollectionConfig {
            default_headers: vec![],
            default_auth: default_auth_record(),
            scripts: CollectionScripts::default(),
        }
    }
}

fn parse_env_file_ordered(path: &Path) -> Vec<EnvVar> {
    let Ok(content) = fs::read_to_string(path) else { return vec![] };
    let mut seen = std::collections::HashSet::new();
    let mut vars = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            if key.is_empty() || seen.contains(&key) {
                continue;
            }
            let raw_val = line[eq_pos + 1..].trim();
            let value = if (raw_val.starts_with('"') && raw_val.ends_with('"'))
                || (raw_val.starts_with('\'') && raw_val.ends_with('\''))
            {
                raw_val[1..raw_val.len() - 1].to_string()
            } else {
                raw_val.to_string()
            };
            seen.insert(key.clone());
            vars.push(EnvVar { key, value });
        }
    }
    vars
}

fn parse_env_file(path: &Path) -> HashMap<String, String> {
    parse_env_file_ordered(path)
        .into_iter()
        .map(|v| (v.key, v.value))
        .collect()
}

fn write_env_file(path: &Path, vars: &[EnvVar]) -> Result<(), String> {
    let lines: Vec<String> = vars
        .iter()
        .filter(|v| !v.key.trim().is_empty())
        .map(|v| format!("{}={}", v.key.trim(), v.value))
        .collect();
    let content = if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    };
    fs::write(path, content).map_err(|e| format!("Failed to write .env: {e}"))
}

fn ensure_env_and_gitignore(dir: &Path) {
    let env_path = dir.join(".env");
    if !env_path.exists() {
        let _ = fs::write(&env_path, "");
    }

    let gitignore_path = dir.join(".gitignore");
    if !gitignore_path.exists() {
        let _ = fs::write(&gitignore_path, ".env\n");
    } else if let Ok(content) = fs::read_to_string(&gitignore_path) {
        if !content.lines().any(|l| l.trim() == ".env") {
            let appended = format!("{}\n.env\n", content.trim_end());
            let _ = fs::write(&gitignore_path, appended);
        }
    }
}

fn get_collection_dir(root: &Path, workspace_name: &str, collection_name: &str) -> PathBuf {
    root.join(workspace_name).join("collections").join(collection_name)
}

pub fn load_env_vars(workspace_path: &Path, collection_path: Option<&Path>) -> HashMap<String, String> {
    let mut vars = parse_env_file(&workspace_path.join(".env"));
    if let Some(col_path) = collection_path {
        for (k, v) in parse_env_file(&col_path.join(".env")) {
            vars.insert(k, v);
        }
    }
    vars
}

pub fn load_collection_config_from_path(collection_path: &Path) -> CollectionConfig {
    let path = collection_path.join(COLLECTION_CONFIG_FILE_NAME);
    let Ok(json) = fs::read_to_string(&path) else { return CollectionConfig::default() };
    serde_json::from_str(&json).unwrap_or_default()
}



fn default_sidebar_width() -> u16 {
    304
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedAppState {
    pub version: u8,
    pub storage_path: Option<PathBuf>,
    pub active_workspace_name: String,
    pub active_collection_name: String,
    pub active_request_name: String,
    pub sidebar_tab: String,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u16,
    pub workspaces: Vec<WorkspaceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRecord {
    pub name: String,
    pub description: Option<String>,
    pub collections: Vec<CollectionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionMeta {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFile {
    pub info: WorkspaceInfo,
    pub collections: Vec<CollectionMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRecord {
    pub name: String,
    pub requests: Vec<RequestRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestRecord {
    pub name: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub query_params: Vec<KeyValueRow>,
    #[serde(default)]
    pub headers: Vec<KeyValueRow>,
    pub auth: AuthRecord,
    pub body_type: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub docs: String,
    pub active_editor_tab: String,
    pub active_response_tab: String,
    pub response_body_view: String,
    pub last_response: Option<SavedResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValueRow {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRecord {
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedResponse {
    pub status: u16,
    pub badge: String,
    pub status_text: String,
    pub duration: String,
    pub size: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub cookies: Vec<String>,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub raw_body: String,
    #[serde(default)]
    pub is_json: bool,
    pub meta: ResponseMeta,
    #[serde(default)]
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMeta {
    pub url: String,
    pub method: String,
}



fn default_state() -> PersistedAppState {
    PersistedAppState {
        version: 1,
        storage_path: None,
        active_workspace_name: String::new(),
        active_collection_name: String::new(),
        active_request_name: String::new(),
        sidebar_tab: "requests".to_string(),
        sidebar_width: default_sidebar_width(),
        workspaces: vec![],
    }
}

fn get_state_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("Failed to resolve app data directory: {err}"))?;

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)
            .map_err(|err| format!("Failed to create app data directory: {err}"))?;
    }

    Ok(app_dir.join("state.json"))
}

#[tauri::command]
pub fn get_app_config(app: AppHandle) -> Result<PersistedAppState, String> {
    let path = get_state_path(&app)?;

    if !path.exists() {
        return Ok(default_state());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read state: {err}"))?;

    serde_json::from_str(&contents).map_err(|err| format!("Failed to parse state: {err}"))
}

#[tauri::command]
pub fn set_storage_path(app: AppHandle, path: String) -> Result<(), String> {
    let state_path = get_state_path(&app)?;
    let storage_path = PathBuf::from(path);

    let mut state = if state_path.exists() {
        let contents = fs::read_to_string(&state_path)
            .map_err(|err| format!("Failed to read state: {err}"))?;
        serde_json::from_str::<PersistedAppState>(&contents)
            .unwrap_or_else(|_| default_state())
    } else {
        default_state()
    };

    state.storage_path = Some(storage_path);

    let serialized = serde_json::to_string_pretty(&state)
        .map_err(|err| format!("Failed to serialize state: {err}"))?;

    fs::write(&state_path, serialized).map_err(|err| format!("Failed to write state: {err}"))
}

#[tauri::command]
pub fn get_default_storage_path(app: AppHandle) -> Result<String, String> {
    let document_dir = app
        .path()
        .document_dir()
        .map_err(|err| format!("Failed to resolve document directory: {err}"))?;

    Ok(document_dir.join("Kivo").to_string_lossy().to_string())
}

pub fn get_storage_root(app: &AppHandle) -> Result<PathBuf, String> {
    let state = get_app_config(app.clone())?;

    if let Some(path) = state.storage_path {
        Ok(path)
    } else {
        match app.path().document_dir() {
            Ok(doc_dir) => Ok(doc_dir.join("Kivo")),
            Err(_) => {
                app.path()
                    .app_data_dir()
                    .map_err(|err| format!("Failed to resolve fallback storage directory: {err}"))
            }
        }
    }
}

#[tauri::command]
pub fn load_app_state(app: AppHandle) -> Result<PersistedAppState, String> {
    let root = get_storage_root(&app)?;

    if !root.exists() {
        return Ok(default_state());
    }

    let mut workspaces = Vec::new();

    let entries = fs::read_dir(&root).map_err(|err| format!("Failed to read storage root: {err}"))?;

    for entry in entries {
        let entry = entry.map_err(|err| format!("Failed to read directory entry: {err}"))?;
        let path = entry.path();

        if path.is_dir() {
            let workspace_file_path = path.join(WORKSPACE_FILE_NAME);
            if workspace_file_path.exists() {
                let workspace_json = fs::read_to_string(&workspace_file_path)
                    .map_err(|err| format!("Failed to read workspace.json: {err}"))?;
                let workspace_file: WorkspaceFile = serde_json::from_str(&workspace_json)
                    .map_err(|err| format!("Failed to parse workspace.json: {err}"))?;

                let mut collections = Vec::new();

        for col_meta in workspace_file.collections {
                    let col_path = if col_meta.path.starts_with("/") || col_meta.path.contains(":\\") {
                        PathBuf::from(&col_meta.path)
                    } else {
                        path.join(&col_meta.path)
                    };

                    if col_path.exists() && col_path.is_dir() {
                        let mut requests = Vec::new();
                        let req_entries = fs::read_dir(&col_path)
                            .map_err(|err| format!("Failed to read collection directory: {err}"))?;

                        for req_entry in req_entries {
                            let req_entry = req_entry.map_err(|err| {
                                format!("Failed to read request directory entry: {err}")
                            })?;
                            let req_path = req_entry.path();

                            if req_path.is_file()
                                && req_path.extension().map_or(false, |ext| ext == "json")
                            {
                                let req_json = fs::read_to_string(&req_path)
                                    .map_err(|err| format!("Failed to read request file: {err}"))?;
                                let request: RequestRecord = serde_json::from_str(&req_json)
                                    .map_err(|err| format!("Failed to parse request file: {err}"))?;
                                requests.push(request);
                            }
                        }

                        collections.push(CollectionRecord {
                            name: col_meta.name,
                            requests,
                        });
                    }
                }

                workspaces.push(WorkspaceRecord {
                    name: workspace_file.info.name,
                    description: workspace_file.info.description,
                    collections,
                });
            }
        }
    }

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("Failed to resolve app data directory: {err}"))?;
    let state_file_path = app_data_dir.join("state.json");

    if state_file_path.exists() {
        let state_json = fs::read_to_string(&state_file_path)
            .map_err(|err| format!("Failed to read state.json: {err}"))?;
        let mut state: PersistedAppState = serde_json::from_str(&state_json)
            .map_err(|err| format!("Failed to parse state.json: {err}"))?;
        state.workspaces = workspaces;
        Ok(state)
    } else {
        let mut state = default_state();
        state.workspaces = workspaces;
        Ok(state)
    }
}

#[tauri::command]
pub fn save_app_state(app: AppHandle, payload: PersistedAppState) -> Result<(), String> {
    let root = get_storage_root(&app)?;

    if !root.exists() {
        fs::create_dir_all(&root).map_err(|err| format!("Failed to create storage root: {err}"))?;
    }

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("Failed to resolve app data directory: {err}"))?;
    let state_file_path = app_data_dir.join("state.json");

    let mut state_to_save = payload.clone();
    state_to_save.workspaces = vec![]; 
    
    if state_to_save.storage_path.is_none() {
        if let Ok(config) = get_app_config(app.clone()) {
            state_to_save.storage_path = config.storage_path;
        }
    }

    let state_json = serde_json::to_string_pretty(&state_to_save)
        .map_err(|err| format!("Failed to serialize state.json: {err}"))?;
    fs::write(&state_file_path, state_json)
        .map_err(|err| format!("Failed to write state.json: {err}"))?;

    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    if !payload.workspaces.iter().any(|w| w.name == dir_name) {
                        if path.join(WORKSPACE_FILE_NAME).exists() {
                            let _ = fs::remove_dir_all(&path);
                        }
                    }
                }
            }
        }
    }

    for workspace in payload.workspaces {
        let workspace_path = root.join(&workspace.name);
        if !workspace_path.exists() {
            fs::create_dir_all(&workspace_path)
                .map_err(|err| format!("Failed to create workspace directory: {err}"))?;
        }
        ensure_env_and_gitignore(&workspace_path);

        let mut collections_meta = Vec::new();
        let collections_root = workspace_path.join("collections");
        if !collections_root.exists() {
            let _ = fs::create_dir_all(&collections_root);
        }

        if let Ok(entries) = fs::read_dir(&collections_root) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = entry.file_name().to_string_lossy().to_string();
                        if !workspace.collections.iter().any(|c| c.name == dir_name) {
                            let _ = fs::remove_dir_all(&path);
                        }
                    }
                }
            }
        }

        for collection in workspace.collections {
            let collection_dir_name = format!("collections/{}", collection.name);
            let collection_path = workspace_path.join(&collection_dir_name);

            if !collection_path.exists() {
                fs::create_dir_all(&collection_path)
                    .map_err(|err| format!("Failed to create collection directory: {err}"))?;
            }
            ensure_env_and_gitignore(&collection_path);

            let existing_entries = fs::read_dir(&collection_path)
                .map_err(|err| format!("Failed to read collection directory: {err}"))?;
            for entry in existing_entries {
                let entry = entry.map_err(|err| format!("Failed to read entry: {err}"))?;
                let ep = entry.path();
                if ep.is_file() {
                    let is_json = ep.extension().map_or(false, |e| e == "json");
                    let is_collection_config = ep.file_name()
                        .map_or(false, |n| n == COLLECTION_CONFIG_FILE_NAME);
                    if is_json && !is_collection_config {
                        fs::remove_file(&ep)
                            .map_err(|err| format!("Failed to remove old request file: {err}"))?;
                    }
                }
            }

            for request in collection.requests {
                let request_file_name = format!("{}.json", request.name);
                let request_path = collection_path.join(request_file_name);
                let request_json = serde_json::to_string_pretty(&request)
                    .map_err(|err| format!("Failed to serialize request: {err}"))?;
                fs::write(request_path, request_json)
                    .map_err(|err| format!("Failed to write request file: {err}"))?;
            }

            collections_meta.push(CollectionMeta {
                name: collection.name.clone(),
                path: collection_dir_name,
            });
        }

        let workspace_file = WorkspaceFile {
            info: WorkspaceInfo {
                name: workspace.name.clone(),
                resource_type: "workspace".to_string(),
                description: workspace.description.clone(),
            },
            collections: collections_meta,
        };

        let workspace_json = serde_json::to_string_pretty(&workspace_file)
            .map_err(|err| format!("Failed to serialize workspace.json: {err}"))?;
        fs::write(workspace_path.join(WORKSPACE_FILE_NAME), workspace_json)
            .map_err(|err| format!("Failed to write workspace.json: {err}"))?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_config_directory(app: AppHandle) -> Result<(), String> {
    let config = get_app_config(app.clone())?;

    let path = if let Some(storage_path) = config.storage_path {
        storage_path
    } else {
        app.path()
            .app_data_dir()
            .map_err(|err| format!("Failed to resolve app data directory: {err}"))?
    };

    if !path.exists() {
        fs::create_dir_all(&path).map_err(|err| format!("Failed to create storage directory: {err}"))?;
    }

    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_path(path.to_string_lossy().to_string(), None::<String>)
        .map_err(|err| format!("Failed to open storage directory: {err}"))?;

    Ok(())
}

#[tauri::command]
pub fn reveal_item(
    app: AppHandle,
    workspace_name: String,
    collection_name: Option<String>,
    request_name: Option<String>,
) -> Result<(), String> {
    let root = get_storage_root(&app)?;
    let mut path = root.join(&workspace_name);

    if let Some(col_name) = collection_name {
        let workspace_file_path = path.join(WORKSPACE_FILE_NAME);
        if workspace_file_path.exists() {
            let workspace_json = fs::read_to_string(&workspace_file_path)
                .map_err(|err| format!("Failed to read workspace.json: {err}"))?;
            let workspace_file: WorkspaceFile = serde_json::from_str(&workspace_json)
                .map_err(|err| format!("Failed to parse workspace.json: {err}"))?;

            if let Some(col_meta) = workspace_file.collections.iter().find(|c| c.name == col_name) {
                if col_meta.path.starts_with("/") || col_meta.path.contains(":\\") {
                    path = PathBuf::from(&col_meta.path);
                } else {
                    path = path.join(&col_meta.path);
                }

                if let Some(req_name) = request_name {
                    let req_file_path = path.join(format!("{}.json", req_name));
                    if req_file_path.exists() {
                        path = req_file_path;
                    }
                }
            }
        }
    }

    if !path.exists() {
        if let Some(parent) = path.parent() {
            if parent.exists() {
                path = parent.to_path_buf();
            }
        }
    }

    tauri_plugin_opener::OpenerExt::opener(&app)
        .reveal_item_in_dir(path.to_string_lossy().to_string())
        .map_err(|err| format!("Failed to reveal item: {err}"))?;

    Ok(())
}

#[tauri::command]
pub fn get_resolved_storage_path(app: AppHandle) -> Result<String, String> {
    let root = get_storage_root(&app)?;
    Ok(root.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_env_vars(
    app: AppHandle,
    workspace_name: String,
    collection_name: Option<String>,
) -> Result<EnvVarsResult, String> {
    let root = get_storage_root(&app)?;
    let workspace_path = root.join(&workspace_name);

    let workspace_vars = parse_env_file_ordered(&workspace_path.join(".env"));

    let collection_vars = match &collection_name {
        Some(col) => {
            let col_path = get_collection_dir(&root, &workspace_name, col);
            parse_env_file_ordered(&col_path.join(".env"))
        }
        None => vec![],
    };

    let mut merged = HashMap::new();
    for v in &workspace_vars {
        merged.insert(v.key.clone(), v.value.clone());
    }
    for v in &collection_vars {
        merged.insert(v.key.clone(), v.value.clone());
    }

    Ok(EnvVarsResult { workspace: workspace_vars, collection: collection_vars, merged })
}

#[tauri::command]
pub fn save_env_vars(
    app: AppHandle,
    workspace_name: String,
    collection_name: Option<String>,
    vars: Vec<EnvVar>,
) -> Result<(), String> {
    let root = get_storage_root(&app)?;

    let env_path = match &collection_name {
        Some(col) => {
            let col_path = get_collection_dir(&root, &workspace_name, col);
            if !col_path.exists() {
                fs::create_dir_all(&col_path)
                    .map_err(|e| format!("Failed to create collection dir: {e}"))?;
            }
            col_path.join(".env")
        }
        None => {
            let ws_path = root.join(&workspace_name);
            if !ws_path.exists() {
                return Err(format!("Workspace '{}' does not exist", workspace_name));
            }
            ws_path.join(".env")
        }
    };

    write_env_file(&env_path, &vars)
}

#[tauri::command]
pub fn get_collection_config(
    app: AppHandle,
    workspace_name: String,
    collection_name: String,
) -> Result<CollectionConfig, String> {
    let root = get_storage_root(&app)?;
    let col_path = get_collection_dir(&root, &workspace_name, &collection_name);
    Ok(load_collection_config_from_path(&col_path))
}

#[tauri::command]
pub fn save_collection_config(
    app: AppHandle,
    workspace_name: String,
    collection_name: String,
    config: CollectionConfig,
) -> Result<(), String> {
    let root = get_storage_root(&app)?;
    let col_path = get_collection_dir(&root, &workspace_name, &collection_name);

    if !col_path.exists() {
        fs::create_dir_all(&col_path)
            .map_err(|e| format!("Failed to create collection dir: {e}"))?;
    }

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize collection config: {e}"))?;
    fs::write(col_path.join(COLLECTION_CONFIG_FILE_NAME), json)
        .map_err(|e| format!("Failed to write collection.json: {e}"))
}
