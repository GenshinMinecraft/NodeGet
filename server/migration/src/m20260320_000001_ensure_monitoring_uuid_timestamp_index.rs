use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            CREATE INDEX IF NOT EXISTS "idx-dynamic-uuid-timestamp"
            ON "dynamic_monitoring" ("uuid", "timestamp");

            CREATE INDEX IF NOT EXISTS "idx-static-uuid-timestamp"
            ON "static_monitoring" ("uuid", "timestamp");
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            DROP INDEX IF EXISTS "idx-dynamic-uuid-timestamp";
            DROP INDEX IF EXISTS "idx-static-uuid-timestamp";
            "#,
        )
        .await?;

        Ok(())
    }
}
