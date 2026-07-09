//! Server 端基础设施模块。
//!
//! 仅在 `server` feature 下可用，包含依赖 jsonrpsee、sea-orm、serde_json 的 trait 和宏。
//!
//! ## 内容
//!
//! | 条目 | 来源 | 说明 |
//! |------|------|------|
//! | `load_from_db` | 迁移自 `server/src/cache/mod.rs` | 使用 `ng_db::get_db()` |
//! | `DbBackedCache` trait | 迁移自 `server/src/cache/mod.rs` | 路径已调整 |
//! | `make_global_cache!` 宏 | 迁移自 `server/src/cache/mod.rs` | `$crate::server::DbBackedCache` |
//! | `token_identity` | **re-export 自 `ng_db::rpc`** | 单一事实源在 ng-db（见 REVIEW M9） |
//! | `TruncatedRaw` | **re-export 自 `ng_db::rpc`** | 同上 |
//! | `rpc_exec!` 宏 | 本地定义，委托上方 re-export 的 TruncatedRaw | macro_rules 1.0 无法跨 crate re-export，故保留宏壳 |
//! | `RpcHelper` trait | **re-export 自 `ng_db::rpc`** | 同上 |
//! | `to_rpc_error` | **re-export 自 `ng_db::rpc`** | 同上，供 M10 内联错误转换统一复用 |

use ng_core::error::NodegetError;
use sea_orm::{EntityTrait, ModelTrait};
use std::future::Future;

// ── 辅助函数：一行从 Entity 全量加载 Models ────────────────────────────

/// 从主数据库全量加载指定 Entity 的所有 Model 记录。
///
/// 供 `DbBackedCache::load_all()` 一行调用，内部步骤：
/// 1. 获取全局 DB 连接
/// 2. 执行 `E::find().all()` 查询
///
/// # Errors
///
/// 当数据库连接未初始化或查询失败时返回错误
pub async fn load_from_db<E>() -> anyhow::Result<Vec<E::Model>>
where
    E: EntityTrait + Send + Sync,
    E::Model: ModelTrait + Clone + Send + Sync + 'static,
{
    // 1. 获取全局 DB 连接
    let db = ng_db::get_db().ok_or_else(|| {
        NodegetError::ConfigNotFound("Database connection not initialized".to_owned())
    })?;
    // 2. 执行全量查询
    E::find()
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to load from DB: {e}"))
}

// ── DbBackedCache trait ───────────────────────────────────────────────

/// DB 全量加载缓存 trait。
///
/// 实现 trait 后配合 `make_global_cache!()` 宏可消除重复的
/// `OnceLock + init/reload/global` 模板代码。
///
/// `reload_from_models` 使用 `&self`（内部可变性），
/// 因为 `OnceLock` 只提供共享引用。
#[allow(async_fn_in_trait)]
pub trait DbBackedCache: Sized + Send + Sync {
    /// 数据库 Model 类型
    type Model: ModelTrait + Clone + Send + Sync + 'static;

    /// 缓存名称（用于日志标识）
    fn cache_name() -> &'static str;

    /// 从 DB Model 列表构建全新的缓存实例。
    ///
    /// 用于首次 `init()` 和重新加载时构建。
    fn build_cache(models: Vec<Self::Model>) -> Self;

    /// 用新的 Model 列表替换缓存的内部状态（使用内部可变性）。
    ///
    /// 每个缓存必须自行实现，通过内部锁（如 RwLock）安全地替换数据。
    async fn reload_from_models(&self, models: Vec<Self::Model>);

    /// 从主 DB 加载全部记录，通常一行即可：`load_from_db::<MyEntity>()`
    fn load_all() -> impl Future<Output = anyhow::Result<Vec<Self::Model>>> + Send;
}

// ── 宏：生成 OnceLock 单例 + init/global/reload ────────────────────

/// 为 `DbBackedCache` 实现类型生成全局单例和 `init() / global() / reload()`。
///
/// ```ignore
/// make_global_cache!(TokenCache, TOKEN_CACHE_GLOBAL);
/// ```
///
/// 生成内容：
/// - `static TOKEN_CACHE_GLOBAL: OnceLock<TokenCache>`
/// - `impl TokenCache { init, global, reload }`
#[macro_export]
macro_rules! make_global_cache {
    ($ty:ty, $global:ident) => {
        static $global: std::sync::OnceLock<$ty> = std::sync::OnceLock::new();

        impl $ty {
            /// 从 DB 全量加载并注册全局缓存。
            ///
            /// 若已初始化则改为 reload（防止并发 init 冲突），内部步骤：
            /// 1. 调用 `load_all()` 加载全部 Model
            /// 2. 调用 `build_cache()` 构建缓存实例
            /// 3. 尝试 `set()` 写入 OnceLock；若已被占用则改为 reload
            pub async fn init() -> anyhow::Result<()> {
                let __models =
                    <$ty as $crate::server::DbBackedCache>::load_all().await?;
                let __count = __models.len();
                let __instance =
                    <$ty as $crate::server::DbBackedCache>::build_cache(__models);
                // 并发 init 时 OnceLock 已被占用，回退到 reload
                if $global.set(__instance).is_err() {
                    tracing::warn!(
                        target: "cache",
                        name = <$ty as $crate::server::DbBackedCache>::cache_name(),
                        "already initialized, reloading"
                    );
                    return Self::reload().await;
                }
                tracing::info!(
                    target: "cache",
                    name = <$ty as $crate::server::DbBackedCache>::cache_name(),
                    count = __count,
                    "cache initialized"
                );
                Ok(())
            }

            /// 获取全局实例。
            ///
            /// 若未调用 `init()` 则返回 `None`。
            pub fn global() -> Option<&'static Self> {
                $global.get()
            }

            /// 从 DB 重新加载缓存数据。
            ///
            /// 若全局实例尚未初始化则无操作，内部步骤：
            /// 1. 获取全局实例引用
            /// 2. 调用 `load_all()` 重新加载全部 Model
            /// 3. 调用 `reload_from_models()` 替换内部状态
            pub async fn reload() -> anyhow::Result<()> {
                // 未初始化时跳过，避免空 reload
                let Some(__inst) = $global.get() else {
                    return Ok(());
                };
                let __models =
                    <$ty as $crate::server::DbBackedCache>::load_all().await?;
                let __count = __models.len();
                __inst.reload_from_models(__models).await;
                tracing::debug!(
                    target: "cache",
                    name = <$ty as $crate::server::DbBackedCache>::cache_name(),
                    count = __count,
                    "cache reloaded"
                );
                Ok(())
            }
        }
    };
}

// ── RPC 基础设施（re-export 自 ng-db，单一事实源）─────────────────────
//
// 此前 token_identity / TruncatedRaw / rpc_exec! / RpcHelper 在 ng-infra 与 ng-db
// 两处各有一份逐字复制的实现（见 REVIEW M9）。因依赖方向为 ng-infra → ng-db
//（RpcHelper::get_db / load_from_db 调 ng_db::get_db），无法让 ng-db 反向依赖 ng-infra，
// 故权威定义留在 ng-db（ng-task / server 二进制等已直接使用 ng_db::rpc::{...}），
// ng-infra 改为 re-export，消除双份实现与未来漂移风险。
//
// 调用方无需改动：用 ng_infra::server::{token_identity, TruncatedRaw, RpcHelper,
// to_rpc_error} 与 ng_infra::rpc_exec! 的代码，经 re-export 透明指向 ng-db 权威定义。

pub use ng_db::rpc::{RpcHelper, TruncatedRaw, to_rpc_error, token_identity};

/// RPC 方法返回 `RpcResult<Box<RawValue>>` 的统一日志宏。
///
/// 用法：`rpc_exec!(some_inner_call(args).await)`
///
/// 输出行为：
/// - 成功：`debug response=<truncated> "request completed"`
/// - 失败：`error error=<e> "request failed"`
///
/// 注意：计时中间件已按配置级别记录请求耗时，本宏仅记录结果。
///
/// 使用 `target: "rpc"` 作为跨领域 RPC 基础设施日志，
/// 区别于领域特定 target（kv、token、js_worker 等）。
///
/// 宏体引用 `$crate::server::TruncatedRaw`（上方 re-export 自 ng-db），保证
/// `ng_infra::rpc_exec!` 与 `ng_db::rpc_exec!` 两入口指向同一 TruncatedRaw 实现。
/// 宏本身在 ng-infra 与 ng-db 各保留一份是宏 re-export 的固有约束（macro_rules 1.0
/// 无法跨 crate `pub use` re-export），但两者逻辑逐字一致且 $crate 路径各自正确。
#[macro_export]
macro_rules! rpc_exec {
    ($expr:expr) => {{
        match $expr {
            Ok(raw) => {
                tracing::debug!(
                    target: "rpc",
                    response = %$crate::server::TruncatedRaw(&raw),
                    "request completed"
                );
                Ok(raw)
            }
            Err(e) => {
                tracing::error!(target: "rpc", error = %e, "request failed");
                Err(e)
            }
        }
    }};
}
