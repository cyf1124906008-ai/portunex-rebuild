//! OpenAI chat/completions 请求 -> Anthropic messages 请求(简化版,纯函数)。
use serde_json::{json, Value};

pub fn openai_to_claude(req: &Value) -> Value {
    let model = req.get("model").cloned().unwrap_or(Value::Null);
    let max_tokens = req.get("max_tokens").and_then(|v| v.as_i64()).unwrap_or(1024);
    let mut system: Option<Value> = None;
    let mut messages: Vec<Value> = Vec::new();
    if let Some(arr) = req.get("messages").and_then(|m| m.as_array()) {
        for m in arr {
            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = m.get("content").cloned().unwrap_or(Value::String(String::new()));
            if role == "system" { system = Some(content); }
            else { messages.push(json!({"role": role, "content": content})); }
        }
    }
    let mut out = json!({"model": model, "max_tokens": max_tokens, "messages": messages});
    if let Some(s) = system { out["system"] = s; }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn system_is_extracted_and_messages_filtered() {
        let input = json!({"model":"claude-x","max_tokens":100,
            "messages":[{"role":"system","content":"be nice"},{"role":"user","content":"hi"}]});
        let out = openai_to_claude(&input);
        assert_eq!(out["system"], json!("be nice"));
        assert_eq!(out["max_tokens"], json!(100));
        let msgs = out["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0]["role"], "user");
    }
}
