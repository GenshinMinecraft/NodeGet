use crate::kv::create_kv;
use crate::rpc::kv::auth::check_kv_create_permission;
use jsonrpsee::core::RpcResult;
use log::debug;
use nodeget_lib::error::NodegetError;
use serde_json::value::RawValue;

pub async fn create(token: String, name: String) -> RpcResult<Box<RawValue>> {
    let process_logic = async {
        debug!("KV RPC: Processing create namespace request for '{name}'");

        // 检查创建命名空间的权限
        check_kv_create_permission(&token).await?;

        let kv_store = create_kv(name).await?;

        let json_str = serde_json::to_string(&kv_store).map_err(|e| {
            NodegetError::SerializationError(format!("Failed to serialize KV store: {e}"))
        })?;

        RawValue::from_string(json_str)
            .map_err(|e| NodegetError::SerializationError(format!("{e}")).into())
    };

    match process_logic.await {
        Ok(result) => Ok(result),
        Err(e) => {
            let nodeget_err = nodeget_lib::error::anyhow_to_nodeget_error(&e);
            Err(jsonrpsee::types::ErrorObject::owned(
                nodeget_err.error_code() as i32,
                format!("{nodeget_err}"),
                None::<()>,
            ))
        }
    }
}
