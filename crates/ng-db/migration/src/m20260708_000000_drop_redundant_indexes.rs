use sea_orm_migration::prelude::*;

/// 删除 5 个冗余二级索引（详见 REVIEW L20 / L31 / L32）：
///
/// - `idx-crontab-name` — 重复 crontab.name 的 UNIQUE 约束（列定义已带 `unique_key`）
/// - `idx-js_worker-id` — 重复 js_worker.id 的 PRIMARY KEY
/// - `idx-crontab_result-id` — 重复 crontab_result.id 的 PRIMARY KEY
/// - `idx-crontab_result-relative_id` — 该列仅写不读，无任何查询过滤使用它
/// - `idx-kv-namespace` — 被复合唯一索引 `idx-kv-namespace-key-unique` (namespace, key)
///   的最左前缀完全覆盖
///
/// 删除后：等值/范围查询仍由对应 UNIQUE 约束 / PK / 复合索引服务，无功能影响；
/// 消除每 INSERT/UPDATE/DELETE 的冗余 B-tree 维护开销与存储。
///
/// `m20260608_000000_add_indexes.rs` 的注释已自承前 3 个「应在后续迁移中移除」，即本迁移。
#[derive(DeriveMigrationName)]
pub struct Migration;

/// 待删除的冗余索引名清单。
const REDUNDANT_INDEXES: &[&str] = &[
    "idx-crontab-name",
    "idx-js_worker-id",
    "idx-crontab_result-id",
    "idx-crontab_result-relative_id",
    "idx-kv-namespace",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // DROP INDEX IF EXISTS 在 SQLite 与 PostgreSQL 均受支持，保证幂等
        // （索引缺失或已迁移的库重复执行不报错）。
        for idx in REDUNDANT_INDEXES {
            db.execute_unprepared(&format!("DROP INDEX IF EXISTS \"{idx}\""))
                .await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // 这些索引在各自的 create-table 迁移中创建，回滚不在此重建（重建会重新引入冗余）。
        // 如需恢复，请执行对应 create-table 迁移或手动 CREATE INDEX。
        Ok(())
    }
}
