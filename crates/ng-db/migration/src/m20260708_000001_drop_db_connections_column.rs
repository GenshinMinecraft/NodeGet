use crate::sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

/// 删除 `db_registry.db_connections` 列（见 REVIEW L34）。
///
/// 该列名义上是「连接引用计数」，实际只增不减（create_conn 递增，remove_conn 删整行不递减），
/// 是 stale counter 而非真 refcount。真正的活信号是 in-memory pool 的 `is_active`。
/// 删除该列消除误导性的「引用计数」契约与维护开销。
///
/// 真实活跃连接数请以 `DbInfo.is_active` / in-memory pool 为准。
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // PostgreSQL 支持 DROP COLUMN IF EXISTS；SQLite 3.35+ 支持 DROP COLUMN 但无 IF EXISTS 子句。
        // 迁移由 SeaORM 记录版本、仅执行一次，故 SQLite 直接 DROP 即可；
        // PostgreSQL 加 IF EXISTS 以容忍已手动删列的库。
        match manager.get_database_backend() {
            DbBackend::Postgres => {
                db.execute_unprepared(
                    "ALTER TABLE \"db_registry\" DROP COLUMN IF EXISTS \"db_connections\"",
                )
                .await?;
            }
            DbBackend::Sqlite => {
                db.execute_unprepared("ALTER TABLE \"db_registry\" DROP COLUMN \"db_connections\"")
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚不在此重建该列（重建会重新引入 stale counter）。
        // 如需恢复，请手动 ALTER TABLE ... ADD COLUMN db_connections integer。
        Ok(())
    }
}
