//! OpenAI 兼容 /v1/models —— 源自 model_aliases。
use axum::{extract::State, Json};
use serde_json::{json, Value};
use crate::state::AppState;
use portunex_db::repositories::model_alias::ModelAliasRepo;

pub async fn list(State(st): State<AppState>) -> Json<Value> {
    let aliases = ModelAliasRepo::list(st.pool()).await.unwrap_or_default();
    let data: Vec<Value> = aliases.iter()
        .map(|a| json!({ "id": a.alias, "object": "model", "owned_by": "portunex" }))
        .collect();
    Json(json!({ "object": "list", "data": data }))
}
