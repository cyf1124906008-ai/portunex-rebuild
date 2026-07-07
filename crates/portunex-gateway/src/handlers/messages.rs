//! /messages(Anthropic 兼容):API Key 鉴权 -> 解析模型 -> 选 provider -> 转发上游 -> 回包。
//! 用本地 mock provider 可端到端验证;接真实 Claude 需 provider 里配真实 OAuth/key。
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::middleware::auth::ApiKeyUser;
use crate::state::AppState;
use portunex_db::repositories::model_alias::ModelAliasRepo;
use portunex_db::repositories::provider::ProviderRepo;
use portunex_db::repositories::provider_credential::ProviderCredentialRepo;

type ApiErr = (StatusCode, Json<Value>);
fn err(c: StatusCode, m: &str) -> ApiErr { (c, Json(json!({"error": {"type": "gateway_error", "message": m}}))) }
fn db_err(e: sqlx::Error) -> ApiErr { (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":{"type":"db","message":e.to_string()}}))) }

pub async fn messages(_key: ApiKeyUser, State(st): State<AppState>, Json(body): Json<Value>) -> Result<(StatusCode, Json<Value>), ApiErr> {
    let pool = st.pool();
    let model = body.get("model").and_then(|m| m.as_str()).unwrap_or("").to_string();

    // 解析别名 -> kind + 上游真实模型名
    let alias = ModelAliasRepo::find_by_alias(pool, &model).await.map_err(db_err)?;
    let (kind, upstream_model) = match &alias {
        Some(a) => (a.kind.clone(), a.upstream_model.clone()),
        None => ("claude_console".to_string(), None),
    };

    // 选 provider + 凭据
    let provider = ProviderRepo::pick_healthy_by_kind(pool, &kind).await.map_err(db_err)?
        .ok_or(err(StatusCode::SERVICE_UNAVAILABLE, "no_provider_available"))?;
    let base = provider.base_url.clone().ok_or(err(StatusCode::INTERNAL_SERVER_ERROR, "provider_no_base_url"))?;
    let (atype, secret) = ProviderCredentialRepo::find_for_provider(pool, provider.id).await.map_err(db_err)?
        .ok_or(err(StatusCode::INTERNAL_SERVER_ERROR, "provider_no_credential"))?;

    // 替换上游真实模型名
    let mut fwd = body.clone();
    if let Some(um) = upstream_model { fwd["model"] = Value::String(um); }

    // 转发
    let (status, resp) = portunex_upstream::forward_json(&base, "/v1/messages", &atype, &secret, &fwd).await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, "upstream_unreachable").tap_detail(e))?;
    // TODO: 计费 + usage_records + sticky 绑定
    let code = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
    Ok((code, Json(resp)))
}

// 小工具:给错误附带 detail(避免上游错误吞掉)
trait TapDetail { fn tap_detail(self, e: anyhow::Error) -> ApiErr; }
impl TapDetail for ApiErr {
    fn tap_detail(self, e: anyhow::Error) -> ApiErr {
        (self.0, Json(json!({"error": {"type":"upstream", "message": e.to_string()}})))
    }
}
