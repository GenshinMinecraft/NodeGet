use serde_json::Value;
use uuid::Uuid;

/// 检查字符串是否为有效的 UUID 格式
pub fn is_valid_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}

/// 从 JSON 值中获取指定 key 的毫秒数值
///
/// 注意：KV 值存储在 `kv` 字段下，格式为：
/// `{"kv": {"database_limit_task": 1000, ...}, "namespace": "..."}`
pub fn get_limit_millis(json_value: &Value, key: &str) -> Option<i64> {
    json_value
        .get("kv")
        .and_then(|kv| kv.get(key))
        .and_then(sea_orm::JsonValue::as_i64)
}
