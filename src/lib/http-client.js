import { invoke } from "@tauri-apps/api/core";

export function sendHttpRequest(payload) {
  return invoke("send_http_request", { payload });
}

export function loadAppState() {
  return invoke("load_app_state");
}

export function saveAppState(payload) {
  return invoke("save_app_state", { payload });
}

export function getEnvVars(workspaceName, collectionName) {
  return invoke("get_env_vars", {
    workspaceName,
    collectionName: collectionName || null,
  });
}

export function saveEnvVars(workspaceName, collectionName, vars) {
  return invoke("save_env_vars", {
    workspaceName,
    collectionName: collectionName || null,
    vars,
  });
}

export function getCollectionConfig(workspaceName, collectionName) {
  return invoke("get_collection_config", { workspaceName, collectionName });
}

export function saveCollectionConfig(workspaceName, collectionName, config) {
  return invoke("save_collection_config", { workspaceName, collectionName, config });
}

export function getResolvedStoragePath() {
  return invoke("get_resolved_storage_path");
}

