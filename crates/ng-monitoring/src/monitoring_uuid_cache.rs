//! 监控 UUID 双向缓存。
//!
//! 维护 `monitoring_uuid` 表的内存映射，支持 UUID↔ID 双向查找。
//! 实现 `DbBackedCache` trait，通过 `make_global_cache!` 宏生成全局单例。
//! 支持软删除（`soft_delete`）：软删除的 UUID 仍保留在缓存中，但 `list_all()` 等方法会过滤；
//! `get_or_insert()` 会自动复活（resurrect）软删除条目。

use ng_core::error::NodegetError;
use ng_db::entity::monitoring_uuid;
use ng_infra::server::{DbBackedCache, load_from_db};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::collections::HashMap;
use std::future::Future;
use std::sync::RwLock;
use tracing::{debug, info, trace};
use uuid::Uuid;

/// 缓存内部数据结构，持有两个方向的映射。
struct MonitoringUuidCacheInner {
    /// UUID → (ID, `soft_delete`) 映射
    by_uuid: HashMap<Uuid, (i16, bool)>,
    /// ID → (UUID, `soft_delete`) 映射
    by_id: HashMap<i16, (Uuid, bool)>,
}

/// 监控 UUID 双向缓存，支持软删除标记。
pub struct MonitoringUuidCache {
    /// 内部双向映射数据（`UUID↔ID`），通过 `RwLock` 保证并发安全
    inner: RwLock<MonitoringUuidCacheInner>,
}

/// 从 `RwLock` 获取读锁，锁中毒时自动恢复。
fn recover_read(
    lock: &RwLock<MonitoringUuidCacheInner>,
) -> std::sync::RwLockReadGuard<'_, MonitoringUuidCacheInner> {
    lock.read().unwrap_or_else(|e| {
        tracing::warn!(target: "monitoring_uuid_cache", "lock poisoned during read, recovering");
        e.into_inner()
    })
}

/// 从 `RwLock` 获取写锁，锁中毒时自动恢复。
fn recover_write(
    lock: &RwLock<MonitoringUuidCacheInner>,
) -> std::sync::RwLockWriteGuard<'_, MonitoringUuidCacheInner> {
    lock.write().unwrap_or_else(|e| {
        tracing::warn!(target: "monitoring_uuid_cache", "lock poisoned during write, recovering");
        e.into_inner()
    })
}

/// 将 `monitoring_uuid.id`（DB `integer` = i32）安全转换为缓存键 i16。
///
/// 三张监控表的 `uuid_id` 列为 `small_integer`(i16)，缓存与公共 API 也用 i16。
/// 直接 `as i16` 在 id > 32767 时静默回绕，导致两个不同 UUID 映射到同一 id（跨 Agent
/// 数据错配、`UNIQUE(uuid_id, data_hash)` 冲突）。此函数改用 checked conversion，
/// 超界返回 `None`，由调用方决定跳过（构建缓存）或返错（写入路径）。
///
/// 根治需破坏性迁移把三表 `uuid_id` 列改为 `integer`(i32) 并改全链路类型（见 REVIEW H1）；
/// 在此之前，checked conversion 至少防止静默回绕。
fn try_id_i16(model_id: i32) -> Option<i16> {
    i16::try_from(model_id)
        .map_err(|_| {
            tracing::error!(
                target: "monitoring_uuid_cache",
                model_id,
                "monitoring_uuid.id exceeds i16 range ({}); skipping to avoid silent wraparound. \
                 Root fix: migrate uuid_id columns to integer (i32). See REVIEW H1.",
                model_id
            );
        })
        .ok()
}

// 通过 make_global_cache! 宏生成 init() / global() / reload() 全局单例方法
ng_infra::make_global_cache!(MonitoringUuidCache, MONITORING_UUID_CACHE_GLOBAL);

impl DbBackedCache for MonitoringUuidCache {
    type Model = monitoring_uuid::Model;

    /// 缓存名称，用于日志标识。
    fn cache_name() -> &'static str {
        "monitoring_uuid"
    }

    /// 从数据库模型列表构建缓存实例。
    fn build_cache(models: Vec<Self::Model>) -> Self {
        let mut by_uuid = HashMap::with_capacity(models.len());
        let mut by_id = HashMap::with_capacity(models.len());
        for model in models {
            // 超界 id 跳过（不污染缓存），而非静默回绕致 id 冲突。见 try_id_i16 / H1。
            let Some(id) = try_id_i16(model.id) else {
                continue;
            };
            by_uuid.insert(model.uuid, (id, model.soft_delete));
            by_id.insert(id, (model.uuid, model.soft_delete));
        }
        Self {
            inner: RwLock::new(MonitoringUuidCacheInner { by_uuid, by_id }),
        }
    }

    /// 从新的模型列表原地替换缓存内容（用于热重载）。
    ///
    /// 1. 构建新的 `by_uuid` / `by_id` 映射
    /// 2. 获取写锁，原子替换内部 `HashMap`
    /// 3. drop 旧映射释放内存
    #[allow(clippy::unused_async)]
    async fn reload_from_models(&self, models: Vec<Self::Model>) {
        let mut by_uuid = HashMap::with_capacity(models.len());
        let mut by_id = HashMap::with_capacity(models.len());
        for model in models {
            // 超界 id 跳过（不污染缓存），而非静默回绕致 id 冲突。见 try_id_i16 / H1。
            let Some(id) = try_id_i16(model.id) else {
                continue;
            };
            by_uuid.insert(model.uuid, (id, model.soft_delete));
            by_id.insert(id, (model.uuid, model.soft_delete));
        }
        let old_maps = {
            let mut guard = recover_write(&self.inner);
            let old_by_uuid = std::mem::replace(&mut guard.by_uuid, by_uuid);
            let old_by_id = std::mem::replace(&mut guard.by_id, by_id);
            drop(guard);
            (old_by_uuid, old_by_id)
        };
        drop(old_maps);
    }

    /// 从数据库全量加载 `monitoring_uuid` 表。
    fn load_all() -> impl Future<Output = anyhow::Result<Vec<Self::Model>>> + Send {
        load_from_db::<monitoring_uuid::Entity>()
    }
}

impl MonitoringUuidCache {
    /// 根据 UUID 查找对应的数字 ID。
    pub fn get_id(&self, uuid: &Uuid) -> Option<i16> {
        recover_read(&self.inner)
            .by_uuid
            .get(uuid)
            .map(|(id, _)| *id)
    }

    /// 根据 ID 查找对应的 UUID。
    pub fn get_uuid(&self, id: i16) -> Option<Uuid> {
        recover_read(&self.inner)
            .by_id
            .get(&id)
            .map(|(uuid, _)| *uuid)
    }

    /// 判断 UUID 是否处于活跃状态（存在且未被软删除）。
    pub fn is_active(&self, uuid: &Uuid) -> bool {
        recover_read(&self.inner)
            .by_uuid
            .get(uuid)
            .is_some_and(|(_, soft_delete)| !soft_delete)
    }

    /// 判断 UUID 是否存在于缓存中（含软删除条目）。
    pub fn exists(&self, uuid: &Uuid) -> bool {
        recover_read(&self.inner).by_uuid.contains_key(uuid)
    }

    /// 列出所有非软删除的 UUID，按字典序排序。
    pub fn list_all(&self) -> Vec<Uuid> {
        let guard = recover_read(&self.inner);
        let mut uuids: Vec<Uuid> = guard
            .by_uuid
            .iter()
            .filter(|(_, (_, soft_delete))| !soft_delete)
            .map(|(uuid, _)| *uuid)
            .collect();
        drop(guard);
        uuids.sort();
        uuids
    }

    /// 列出所有 UUID 及其软删除状态，按 UUID 排序。
    pub fn list_all_with_agent_mode(&self) -> Vec<(Uuid, bool)> {
        let guard = recover_read(&self.inner);
        let mut result: Vec<(Uuid, bool)> = guard
            .by_uuid
            .iter()
            .map(|(uuid, (_, soft_delete))| (*uuid, *soft_delete))
            .collect();
        drop(guard);
        result.sort_by_key(|a| a.0);
        result
    }

    // get_or_insert 涵盖缓存命中、DB 查询、软删复活、insert 与 UNIQUE 冲突回退（含复活）
    // 等多条路径，逻辑内聚但行数超 pedantic 阈值；不强行拆分以免割裂「复活一致性」语义。
    #[allow(clippy::too_many_lines)]
    /// 查找或插入 UUID，返回对应的数字 ID。
    ///
    /// 1. 先查内存缓存，若 UUID 活跃则直接返回 ID
    /// 2. 缓存未命中则查询数据库
    /// 3. 若数据库中存在且被软删除，则复活（设置 `soft_delete=false`）
    /// 4. 若数据库中不存在，则插入新记录
    /// 5. 更新内存缓存后返回 ID
    ///
    /// # Errors
    ///
    /// - 数据库连接未初始化时返回 `NodegetError::DatabaseError`
    /// - 查询或插入数据库失败时返回 `NodegetError::DatabaseError`
    /// - 复活软删除条目时更新失败返回 `NodegetError::DatabaseError`
    pub async fn get_or_insert(&self, uuid: Uuid) -> Result<i16, NodegetError> {
        {
            let guard = recover_read(&self.inner);
            if let Some((id, soft_delete)) = guard.by_uuid.get(&uuid)
                && !soft_delete
            {
                trace!(target: "monitoring", %uuid, id, "get_or_insert: 缓存命中，UUID 活跃");
                return Ok(*id);
            }
        }

        trace!(target: "monitoring", %uuid, "get_or_insert: 缓存未命中，查询数据库");

        let db = ng_db::get_db().ok_or_else(|| {
            NodegetError::DatabaseError("Database connection not initialized".to_owned())
        })?;

        let existing = monitoring_uuid::Entity::find()
            .filter(monitoring_uuid::Column::Uuid.eq(uuid))
            .one(db)
            .await
            .map_err(|e| {
                NodegetError::DatabaseError(format!("Failed to query monitoring_uuid: {e}"))
            })?;

        if let Some(model) = existing {
            // 超界 id 返错（无法用 i16 表示，继续会静默回绕致 id 冲突）。见 try_id_i16 / H1。
            let id = try_id_i16(model.id).ok_or_else(|| {
                NodegetError::DatabaseError(format!(
                    "monitoring_uuid id {} for {} exceeds i16 range; \
                     migrate uuid_id columns to i32 (see REVIEW H1)",
                    model.id, uuid
                ))
            })?;
            if model.soft_delete {
                debug!(target: "monitoring", %uuid, id, "get_or_insert: 数据库中找到软删除条目，执行复活");
                let mut active: monitoring_uuid::ActiveModel = model.into();
                active.soft_delete = Set(false);
                active.update(db).await.map_err(|e| {
                    NodegetError::DatabaseError(format!("Failed to resurrect monitoring_uuid: {e}"))
                })?;
                info!(target: "monitoring_uuid_cache", %uuid, "Resurrected soft-deleted uuid");
            } else {
                debug!(target: "monitoring", %uuid, id, "get_or_insert: 数据库中找到活跃条目，更新缓存");
            }
            let mut guard = recover_write(&self.inner);
            guard.by_uuid.insert(uuid, (id, false));
            guard.by_id.insert(id, (uuid, false));
            drop(guard);
            return Ok(id);
        }

        debug!(target: "monitoring", %uuid, "get_or_insert: 数据库中不存在，插入新记录");

        let new_model = monitoring_uuid::ActiveModel {
            id: ActiveValue::default(),
            uuid: Set(uuid),
            soft_delete: Set(false),
        };

        // 并发首次注册同一 UUID 时，两个请求都可能通过上方的 DB 不存在检查后
        // 进入 insert；其一成功，另一触发 UNIQUE(uuid) 冲突。此处识别该冲突并
        // 回退查询已插入行（INSERT OR IGNORE 语义），避免给调用方返回错误的
        // DatabaseError(103)。与 super_token::generate_super_token 的处理同构。
        let (id, soft_delete) = match monitoring_uuid::Entity::insert(new_model).exec(db).await {
            // 新插入成功：soft_delete 必为 false（上方 ActiveModel 设 Set(false)）
            Ok(result) => try_id_i16(result.last_insert_id).map(|id| (id, false)),
            Err(e) => {
                if matches!(
                    e.sql_err(),
                    Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
                ) {
                    debug!(target: "monitoring", %uuid, "get_or_insert: insert 冲突，回退查询已存在行");
                    // 从 re-queried model 读取 soft_delete，而非硬编码 false。
                    // 修正 TOCTOU：并发 soft_delete RPC 可能在 insert 失败与 re-query 之间
                    // 将该行标记为 soft_delete=true，硬编码 false 会使缓存与 DB 不一致，
                    // 且不一致会持续到下次 reload（soft_delete/get_or_insert 均短路于缓存）。
                    let m = monitoring_uuid::Entity::find()
                        .filter(monitoring_uuid::Column::Uuid.eq(uuid))
                        .one(db)
                        .await
                        .map_err(|qe| {
                            NodegetError::DatabaseError(format!(
                                "Failed to query monitoring_uuid after unique conflict: {qe}"
                            ))
                        })?
                        .ok_or_else(|| {
                            NodegetError::DatabaseError(format!(
                                "monitoring_uuid insert conflicted but row not found: {e}"
                            ))
                        })?;
                    // 先取 id（into() 会消费 m）；超界则 arm 产 None，由外层 ok_or_else 兜底。
                    match try_id_i16(m.id) {
                        Some(id) => {
                            // 与 existing 分支保持一致的复活语义：若 re-query 到 soft_delete=true，
                            // 同样执行复活（update soft_delete=false），而非原样缓存软删状态
                            // （否则 get_or_insert「自动复活软删条目」契约在 conflict 路径下失效）。
                            let soft_delete = if m.soft_delete {
                                debug!(target: "monitoring", %uuid, id, "get_or_insert: 冲突行已软删，执行复活");
                                let mut active: monitoring_uuid::ActiveModel = m.into();
                                active.soft_delete = Set(false);
                                active.update(db).await.map_err(|ue| {
                                    NodegetError::DatabaseError(format!(
                                        "Failed to resurrect monitoring_uuid after conflict: {ue}"
                                    ))
                                })?;
                                info!(target: "monitoring_uuid_cache", %uuid, "Resurrected soft-deleted uuid (conflict path)");
                                false
                            } else {
                                m.soft_delete
                            };
                            Some((id, soft_delete))
                        }
                        None => None,
                    }
                } else {
                    return Err(NodegetError::DatabaseError(format!(
                        "Failed to insert monitoring_uuid: {e}"
                    )));
                }
            }
        }
        .ok_or_else(|| {
            NodegetError::DatabaseError(format!(
                "monitoring_uuid id for {uuid} exceeds i16 range after insert/conflict; \
                 migrate uuid_id columns to i32 (see REVIEW H1)"
            ))
        })?;

        debug!(target: "monitoring", %uuid, id, soft_delete, "get_or_insert: 新记录插入成功");
        let mut guard = recover_write(&self.inner);
        guard.by_uuid.insert(uuid, (id, soft_delete));
        guard.by_id.insert(id, (uuid, soft_delete));
        drop(guard);
        Ok(id)
    }

    /// 软删除指定 UUID。
    ///
    /// - 返回 `true` — 成功标记为软删除
    /// - 返回 `false` — UUID 不存在
    /// - 已软删除的 UUID 再次调用仍返回 `true`
    ///
    /// # Errors
    ///
    /// - 数据库连接未初始化时返回 `NodegetError::DatabaseError`
    /// - 查询或更新数据库失败时返回 `NodegetError::DatabaseError`
    pub async fn soft_delete(&self, uuid: Uuid) -> Result<bool, NodegetError> {
        let db = ng_db::get_db().ok_or_else(|| {
            NodegetError::DatabaseError("Database connection not initialized".to_owned())
        })?;

        let existing = monitoring_uuid::Entity::find()
            .filter(monitoring_uuid::Column::Uuid.eq(uuid))
            .one(db)
            .await
            .map_err(|e| {
                NodegetError::DatabaseError(format!(
                    "Failed to query monitoring_uuid for soft_delete: {e}"
                ))
            })?;

        let Some(model) = existing else {
            return Ok(false);
        };

        if model.soft_delete {
            return Ok(true);
        }

        // 超界 id 返错（无法用 i16 表示，继续会静默回绕致 id 冲突）。见 try_id_i16 / H1。
        let id = try_id_i16(model.id).ok_or_else(|| {
            NodegetError::DatabaseError(format!(
                "monitoring_uuid id {} for {} exceeds i16 range; \
                 migrate uuid_id columns to i32 (see REVIEW H1)",
                model.id, uuid
            ))
        })?;
        let mut active: monitoring_uuid::ActiveModel = model.into();
        active.soft_delete = Set(true);
        active.update(db).await.map_err(|e| {
            NodegetError::DatabaseError(format!("Failed to soft_delete monitoring_uuid: {e}"))
        })?;

        let mut guard = recover_write(&self.inner);
        guard.by_uuid.insert(uuid, (id, true));
        guard.by_id.insert(id, (uuid, true));
        drop(guard);

        info!(target: "monitoring_uuid_cache", %uuid, "Soft-deleted uuid");
        Ok(true)
    }
}
