//! OpenAI /v1/chat/completions:用 openai_to_claude 转换请求 -> 转发上游(claude_console)
//! -> 把 Anthropic 响应转回 OpenAI chat 形。用本地 mock 可端到端验证。
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::middleware::auth::ApiKeyUser;
use crate::state::AppState;
use portunex_db::repositories::model_alias::ModelAliasRepo;
use portunex_db::repositories::provider::ProviderRepo;
use portunex_db::repositories::provider_credential::ProviderCredentialRepo;
use portunex_core::translators::openai_to_claude::openai_to_claude;

type ApiErr = (StatusCode, Json<Value>);
fn err(c: StatusCode, m: &str) -> ApiErr { (c, Json(json!({"error": {"message": m, "type": "gateway_error"}}))) }
fn db_err(e: sqlx::Error) -> ApiErr { (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":{"message":e.to_string(),"type":"db"}}))) }

pub async fn chat_completions(_key: ApiKeyUser, State(st): State<AppState>, Json(body): Json<Value>) -> Result<(StatusCode, Json<Value>), ApiErr> {
    let pool = st.pool();
    let model = body.get("model").and_then(|m| m.as_str()).unwrap_or("").to_string();

    // 1) OpenAI 请求 -> Claude 请求
    let mut claude_req = openai_to_claude(&body);

    // 2) 解析别名 + 选 provider
    let alias = ModelAliasRepo::find_by_alias(pool, &model).await.map_err(db_err)?;
    let (kind, upstream_model) = match &alias {
        Some(a) => (a.kind.clone(), a.upstream_model.clone()),
        None => ("claude_console".to_string(), None),
    };
    if let Some(um) = upstream_model { claude_req["model"] = Value::String(um); }
    let provider = ProviderRepo::pick_healthy_by_kind(pool, &kind).await.map_err(db_err)?
        .ok_or(err(StatusCode::SERVICE_UNAVAILABLE, "no_provider_available"))?;
    let base = provider.base_url.clone().ok_or(err(StatusCode::INTERNAL_SERVER_ERROR, "provider_no_base_url"))?;
    let (atype, secret) = ProviderCredentialRepo::find_for_provider(pool, provider.id).await.map_err(db_err)?
        .ok_or(err(StatusCode::INTERNAL_SERVER_ERROR, "provider_no_credential"))?;

    // 3) 转发
    let (status, claude_resp) = portunex_upstream::forward_json(&base, "/v1/messages", &atype, &secret, &claude_req).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(json!({"error":{"message":e.to_string(),"type":"upstream"}}))))?;

    // 4) Claude 响应 -> OpenAI chat 形(简化:提取 text 拼成 choices[0].message.content)
    let text = claude_resp.get("content").and_then(|c| c.as_array())
        .map(|arr| arr.iter().filter_map(|b| b.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join(""))
        .unwrap_or_default();
    let openai_resp = json!({
        "id": claude_resp.get("id").cloned().unwrap_or(json!("chatcmpl-gw")),
        "object": "chat.completion",
        "model": model,
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": text},
            "finish_reason": claude_resp.get("stop_reason").cloned().unwrap_or(json!("stop"))
        }]
    });
    let code = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
    Ok((code, Json(openai_resp)))
}
