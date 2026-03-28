use crate::DB;
use crate::entity::js_result;
use crate::token::get::check_token_limit;
use nodeget_lib::error::NodegetError;
use nodeget_lib::permission::data_structure::{JsResult as JsResultPermission, Permission, Scope};
use nodeget_lib::permission::token_auth::TokenOrAuth;
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

#[derive(Debug, Clone, Copy)]
pub enum JsResultAction {
    Read,
    Delete,
}

fn build_required_permission(action: JsResultAction, worker_name: &str) -> Permission {
    match action {
        JsResultAction::Read => {
            Permission::JsResult(JsResultPermission::Read(worker_name.to_owned()))
        }
        JsResultAction::Delete => {
            Permission::JsResult(JsResultPermission::Delete(worker_name.to_owned()))
        }
    }
}

pub async fn ensure_js_result_permission(
    token: &str,
    worker_name: &str,
    action: JsResultAction,
) -> anyhow::Result<()> {
    let token_or_auth = TokenOrAuth::from_full_token(token)
        .map_err(|e| NodegetError::ParseError(format!("Failed to parse token: {e}")))?;

    let is_allowed = check_token_limit(
        &token_or_auth,
        vec![Scope::JsWorker(worker_name.to_owned())],
        vec![build_required_permission(action, worker_name)],
    )
    .await?;

    if is_allowed {
        return Ok(());
    }

    Err(NodegetError::PermissionDenied(format!(
        "Permission denied for js_result on worker '{worker_name}', action: {action:?}"
    ))
    .into())
}

pub async fn resolve_accessible_js_result_workers(
    token: &str,
    action: JsResultAction,
) -> anyhow::Result<Vec<String>> {
    let token_or_auth = TokenOrAuth::from_full_token(token)
        .map_err(|e| NodegetError::ParseError(format!("Failed to parse token: {e}")))?;

    let db = DB
        .get()
        .ok_or_else(|| NodegetError::DatabaseError("DB not initialized".to_owned()))?;

    let mut worker_names: Vec<String> = js_result::Entity::find()
        .select_only()
        .column(js_result::Column::JsWorkerName)
        .order_by_asc(js_result::Column::JsWorkerName)
        .into_tuple()
        .all(db)
        .await
        .map_err(|e| NodegetError::DatabaseError(e.to_string()))?;

    worker_names.dedup();

    let mut allowed = Vec::new();
    for worker_name in worker_names {
        let is_allowed = check_token_limit(
            &token_or_auth,
            vec![Scope::JsWorker(worker_name.clone())],
            vec![build_required_permission(action, worker_name.as_str())],
        )
        .await?;

        if is_allowed {
            allowed.push(worker_name);
        }
    }

    Ok(allowed)
}
