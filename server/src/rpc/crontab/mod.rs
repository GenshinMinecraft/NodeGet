mod create;

use crate::entity::metadata as metadata_entity;
use crate::rpc::RpcHelper;
use crate::token::get::check_token_limit;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use nodeget_lib::crontab::{Cron, CronType};
use nodeget_lib::metadata;
use nodeget_lib::permission::data_structure::{Metadata as MetadataPermission, Permission, Scope};
use nodeget_lib::permission::token_auth::TokenOrAuth;
use nodeget_lib::utils::error_message::generate_error_message;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
};
use serde_json::{Value, json};
use uuid::Uuid;

#[rpc(server, namespace = "crontab")]
pub trait Rpc {
    #[method(name = "create")]
    async fn create(
        &self,
        token: String,
        name: String,
        cron_expression: String,
        cron_type: CronType,
    ) -> Value;
}

pub struct CrontabRpcImpl;

impl RpcHelper for CrontabRpcImpl {}

#[async_trait]
impl RpcServer for CrontabRpcImpl {
    async fn create(
        &self,
        token: String,
        name: String,
        cron_expression: String,
        cron_type: CronType,
    ) -> Value {
        create::create(token, name, cron_expression, cron_type).await
    }
}
