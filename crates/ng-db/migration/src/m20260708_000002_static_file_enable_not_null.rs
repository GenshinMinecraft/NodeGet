use crate::sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

/// `static_file.enable` 列：清除历史 NULL 并收紧为 NOT NULL DEFAULT true（见 REVIEW L33）。
///
/// 此前列为 `boolean DEFAULT true` 但可空（无 NOT NULL），且 `update_static` 曾用 `Set(None)`
/// 写入 NULL（已另修为 `Unchanged`）。NULL 与 Some(true) 被 router 同视为 enabled，三值逻辑
/// 是 latent footgun。本迁移：先 UPDATE 把存量 NULL 填为 true，再收紧为 NOT NULL。
///
/// - PostgreSQL：`ALTER COLUMN ... SET NOT NULL` 直接支持。
/// - SQLite：不支持 `ALTER COLUMN SET NOT NULL`，故仅 UPDATE 清除 NULL（数据层无 NULL），
///   约束层不强制——由 app 层（entity 已改 bool、update_static 用 Unchanged）保证不再产生 NULL。
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // 两后端：先把存量 NULL 填为 true（清除历史 NULL 数据）
        db.execute_unprepared("UPDATE \"static_file\" SET \"enable\" = true WHERE \"enable\" IS NULL")
            .await?;
        // PostgreSQL：收紧为 NOT NULL（SQLite 因 ALTER 限制跳过，靠 app 层保证）
        if matches!(manager.get_database_backend(), DbBackend::Postgres) {
            db.execute_unprepared(
                "ALTER TABLE \"static_file\" ALTER COLUMN \"enable\" SET NOT NULL",
            )
            .await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚不在此重建可空语义（NOT NULL 是更严格的正确契约）。
        Ok(())
    }
}
