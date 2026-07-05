//! entities/vendor_models.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VendorModels {
    pub id: i64,
    pub kind: Option<String>,
    pub model_id: Option<String>,
    pub active: Option<bool>,
    pub meta_json: Option<serde_json::Value>,
}
