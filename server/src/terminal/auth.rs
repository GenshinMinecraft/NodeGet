use crate::token::get::check_token_limit;
use nodeget_lib::error::NodegetError;
use nodeget_lib::permission::data_structure::{Permission, Scope, Terminal};
use nodeget_lib::permission::token_auth::TokenOrAuth;
use uuid::Uuid;

/// 检查是否有 Terminal 连接权限
///
/// # 参数
/// * `token` - 令牌字符串
/// * `agent_uuid` - Agent 的 UUID
///
/// # 返回值
/// 如果有权限返回 Ok(()，否则返回错误
pub async fn check_terminal_connect_permission(
    token: &str,
    agent_uuid: &str,
) -> anyhow::Result<()> {
    // 解析 Agent UUID
    let agent_uuid = Uuid::parse_str(agent_uuid)
        .map_err(|_| NodegetError::ParseError("Invalid Agent UUID format".to_owned()))?;

    let token_or_auth = TokenOrAuth::from_full_token(token)
        .map_err(|e| NodegetError::ParseError(format!("Failed to parse token: {e}")))?;

    // 构建 scope - 使用 AgentUuid
    let scope = Scope::AgentUuid(agent_uuid);

    // 检查 Terminal Connect 权限
    let has_permission = check_token_limit(
        &token_or_auth,
        vec![scope.clone()],
        vec![Permission::Terminal(Terminal::Connect)],
    )
    .await?;

    if has_permission {
        return Ok(());
    }

    // 也检查 Global Scope 的权限
    let global_scope = Scope::Global;
    let has_global_permission = check_token_limit(
        &token_or_auth,
        vec![global_scope],
        vec![Permission::Terminal(Terminal::Connect)],
    )
    .await?;

    if has_global_permission {
        return Ok(());
    }

    Err(NodegetError::PermissionDenied(format!(
        "No terminal connect permission for agent '{agent_uuid}'"
    ))
    .into())
}
