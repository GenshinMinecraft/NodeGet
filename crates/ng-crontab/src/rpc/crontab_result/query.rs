//! `crontab-result.query` RPC 实现：按条件查询定时任务执行结果。

use crate::query::{CrontabResultDataQuery, CrontabResultQueryCondition};
use jsonrpsee::core::RpcResult;
use ng_core::error::NodegetError;
use ng_core::utils::MAX_QUERY_LIMIT;
use ng_db::entity::crontab_result;
use ng_db::get_db;
use ng_infra::server::to_rpc_error;
use sea_orm::{ColumnTrait, EntityTrait, ExprTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::value::RawValue;
use std::collections::HashSet;
use tracing::debug;

/// 按条件查询定时任务执行结果。
///
/// 1. 构建 SeaORM 查询，应用各过滤条件
/// 2. 从条件中收集 CronName，逐个检查读权限
/// 3. 默认按 run_time 降序排序
/// 4. 未显式指定 Limit/Last 时施加 1000 条默认上限，防止无界查询
/// 5. 执行查询并序列化结果
///
/// - `token` - 认证 Token 字符串
/// - `query` - 查询条件
/// - 返回 `Vec<crontab_result::Model>` 的 JSON 序列化
pub async fn query(token: String, query: CrontabResultDataQuery) -> RpcResult<Box<RawValue>> {
    let process_logic = async {
        debug!(target: "crontab_result", condition_count = query.condition.len(), "processing crontab_result query request");
        let db =
            get_db().ok_or_else(|| NodegetError::DatabaseError("DB not initialized".to_owned()))?;

        // 构建 SeaORM 查询
        let mut select = crontab_result::Entity::find();

        // 收集所有 CronName 条件，用于后续权限检查
        let mut cron_names: HashSet<&str> = HashSet::new();
        let mut has_cron_name_filter = false;
        // Limit/Last 在循环内只记录、循环后互斥单次 apply（对齐 js_result 范式，见 REVIEW L25）。
        // 此前循环内直接多次 .limit()，依赖 SeaORM last-write-wins，Last+Limit(N) 组合时
        // 有效 limit 取决于 condition 顺序，行为不确定。
        let mut is_last = false;
        let mut limit_count: Option<u64> = None;

        // 处理查询条件，应用到 SeaORM 查询
        for condition in &query.condition {
            match condition {
                CrontabResultQueryCondition::Id(id) => {
                    select = select.filter(crontab_result::Column::Id.eq(*id));
                }
                CrontabResultQueryCondition::CronId(cron_id) => {
                    select = select.filter(crontab_result::Column::CronId.eq(*cron_id));
                }
                CrontabResultQueryCondition::CronName(cron_name) => {
                    cron_names.insert(cron_name);
                    has_cron_name_filter = true;
                    select = select.filter(crontab_result::Column::CronName.eq(cron_name.clone()));
                }
                CrontabResultQueryCondition::RunTimeFromTo(start, end) => {
                    select = select.filter(
                        crontab_result::Column::RunTime
                            .gte(*start)
                            .and(crontab_result::Column::RunTime.lte(*end)),
                    );
                }
                CrontabResultQueryCondition::RunTimeFrom(start) => {
                    select = select.filter(crontab_result::Column::RunTime.gte(*start));
                }
                CrontabResultQueryCondition::RunTimeTo(end) => {
                    select = select.filter(crontab_result::Column::RunTime.lte(*end));
                }
                CrontabResultQueryCondition::IsSuccess => {
                    select = select.filter(crontab_result::Column::Success.eq(true));
                }
                CrontabResultQueryCondition::IsFailure => {
                    select = select.filter(crontab_result::Column::Success.eq(false));
                }
                CrontabResultQueryCondition::Limit(limit) => {
                    // 循环内只记录，循环后统一 apply（避免多次 .limit() 的 last-write-wins 歧义）
                    limit_count = Some(std::cmp::min(*limit, MAX_QUERY_LIMIT));
                }
                CrontabResultQueryCondition::Last => {
                    is_last = true;
                }
            }
        }

        // 权限检查：每个 distinct CronName 都需要读权限
        if has_cron_name_filter {
            for cron_name in &cron_names {
                super::auth::check_crontab_result_read_permission(&token, cron_name).await?;
            }
        } else {
            // 没有 CronName 过滤时需要全局读权限
            super::auth::check_crontab_result_read_permission(&token, "*").await?;
        }

        debug!(target: "crontab_result", condition_count = query.condition.len(), has_cron_name_filter, "crontab_result query permission check passed");

        // 循环后互斥单次 apply limit + 排序（对齐 js_result）。
        // - Last：按 run_time 降序取 1 条
        // - Limit(N)：按 run_time 降序取 N 条（已钳到 MAX_QUERY_LIMIT）
        // - 都无：按 run_time 降序取默认 1000 条（防无界查询）
        if is_last {
            select = select
                .order_by_desc(crontab_result::Column::RunTime)
                .limit(1);
        } else if let Some(limit) = limit_count {
            select = select
                .order_by_desc(crontab_result::Column::RunTime)
                .limit(limit);
        } else {
            select = select
                .order_by_desc(crontab_result::Column::RunTime)
                .limit(1000);
        }

        // 执行查询
        let results = select.all(db).await.map_err(|e| {
            NodegetError::DatabaseError(format!("Failed to query crontab_result: {e}"))
        })?;

        debug!(target: "crontab_result", result_count = results.len(), "crontab_result query completed");

        // 序列化结果
        serde_json::value::to_raw_value(&results)
            .map_err(|e| NodegetError::SerializationError(format!("{e}")).into())
    };

    match process_logic.await {
        Ok(result) => Ok(result),
        Err(e) => Err(to_rpc_error(&e)),
    }
}
