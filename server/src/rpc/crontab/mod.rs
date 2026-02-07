mod create;
mod get;

use crate::rpc::RpcHelper;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use nodeget_lib::crontab::CronType;
use serde_json::Value;

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

    #[method(name = "get")]
    async fn get(&self, token: String) -> Value;
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

    async fn get(&self, token: String) -> Value {
        get::get(token).await
    }
}
