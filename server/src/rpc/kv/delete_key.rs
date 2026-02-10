use crate::kv::delete_key_from_kv;
use crate::rpc::kv::auth::check_kv_delete_permission;
use jsonrpsee::core::RpcResult;
use log::debug;
use nodeget_lib::error::NodegetError;
use serde_json::value::RawValue;

pub async fn delete_key(token: String, namespace: String, key: String) -> RpcResult<Box<RawValue>> {
    let process_logic = async {
        debug!("KV RPC: Processing delete_key request for namespace '{namespace}', key '{key}'");

        // 检查删除权限
        check_kv_delete_permission(&token, &namespace, &key).await?;

        delete_key_from_kv(namespace, key).await?;

        let json_str = "{\"success\":true}".to_string();

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
