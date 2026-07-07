//! portunex-upstream —— 上游厂商调用。
//! 已实现:forward_json(通用 JSON 转发,供 /messages 等)。
//! TODO(需真实凭据):各家 OAuth 换/刷 token、WebSocket 池、SSE 流式、node_proxy TLS 伪装。
use serde_json::Value;

/// 把请求体转发到上游 provider,返回 (状态码, 响应 JSON)。
/// auth_type: "api_key" -> x-api-key(Anthropic 风格);其它 -> Authorization: Bearer。
pub async fn forward_json(base_url: &str, path: &str, auth_type: &str, secret: &str, body: &Value) -> anyhow::Result<(u16, Value)> {
    let client = reqwest::Client::builder().build()?;
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    let mut rb = client.post(&url).json(body);
    rb = match auth_type {
        "api_key" => rb.header("x-api-key", secret).header("anthropic-version", "2023-06-01"),
        _ => rb.header("authorization", format!("Bearer {}", secret)),
    };
    let resp = rb.send().await?;
    let status = resp.status().as_u16();
    let json: Value = resp.json().await.unwrap_or(Value::Null);
    Ok((status, json))
}
