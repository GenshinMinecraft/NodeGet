use crate::token;
use crate::token::super_token::check_super_token;
use jsonrpsee::core::RpcResult;
use nodeget_lib::error::NodegetError;
use nodeget_lib::permission::token_auth::TokenOrAuth;
use serde_json::value::RawValue;

pub async fn delete(token: String, target_token: Option<String>) -> RpcResult<Box<RawValue>> {
    let process_logic = async {
        let token_or_auth = TokenOrAuth::from_full_token(&token)
            .map_err(|e| NodegetError::ParseError(format!("Failed to parse token: {e}")))?;

        let is_super_token = check_super_token(&token_or_auth)
            .await
            .map_err(|e| NodegetError::PermissionDenied(format!("{e}")))?;

        if !is_super_token {
            return Err(NodegetError::PermissionDenied(
                "Only SuperToken can delete tokens".to_owned(),
            )
            .into());
        }

        let Some(target_token_to_delete) = target_token else {
            return Err(NodegetError::PermissionDenied(
                "Target token (key/username) is required for SuperToken deletion".to_string(),
            )
            .into());
        };

        let delete_result_by_key = token::delete_token_by_key(target_token_to_delete.clone())
            .await
            .map_err(|e| NodegetError::DatabaseError(e.to_string()))?;

        let json_str = if delete_result_by_key.rows_affected > 0 {
            format!(
                "{{\"success\":true,\"message\":\"Token {} deleted successfully by SuperToken\",\"rows_affected\":{},\"matched_by\":\"token_key\"}}",
                target_token_to_delete, delete_result_by_key.rows_affected
            )
        } else {
            let delete_result_by_username = token::delete_token_by_username(target_token_to_delete.clone())
                .await
                .map_err(|e| NodegetError::DatabaseError(e.to_string()))?;

            if delete_result_by_username.rows_affected > 0 {
                format!(
                    "{{\"success\":true,\"message\":\"Token {} deleted successfully by SuperToken\",\"rows_affected\":{},\"matched_by\":\"username\"}}",
                    target_token_to_delete, delete_result_by_username.rows_affected
                )
            } else {
                format!(
                    "{{\"success\":false,\"message\":\"Token {target_token_to_delete} not found\"}}"
                )
            }
        };

        RawValue::from_string(json_str)
            .map_err(|e| NodegetError::SerializationError(e.to_string()).into())
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
