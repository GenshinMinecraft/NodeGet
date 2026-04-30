use crate::rpc::RpcHelper;
use crate::rpc::{rpc_exec, token_identity};
use jsonrpsee::core::RpcResult;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use nodeget_lib::crontab_result::query::CrontabResultDataQuery;
use serde_json::value::RawValue;
use tracing::Instrument;

mod auth;
mod delete;
mod query;

#[rpc(server, namespace = "crontab-result")]
pub trait Rpc {
    #[method(name = "query")]
    async fn query(&self, token: String, query: CrontabResultDataQuery)
    -> RpcResult<Box<RawValue>>;

    #[method(name = "delete")]
    async fn delete(
        &self,
        token: String,
        query: CrontabResultDataQuery,
    ) -> RpcResult<Box<RawValue>>;
}

pub struct CrontabResultRpcImpl;

impl RpcHelper for CrontabResultRpcImpl {}

#[async_trait]
impl RpcServer for CrontabResultRpcImpl {
    async fn query(
        &self,
        token: String,
        query: CrontabResultDataQuery,
    ) -> RpcResult<Box<RawValue>> {
        let (tk, un) = token_identity(&token);
        let span = tracing::info_span!(target: "crontab_result", "crontab-result::query", token_key = tk, username = un, query = ?query);
        async { rpc_exec!(query::query(token, query).await) }
            .instrument(span)
            .await
    }

    async fn delete(
        &self,
        token: String,
        query: CrontabResultDataQuery,
    ) -> RpcResult<Box<RawValue>> {
        let (tk, un) = token_identity(&token);
        let span = tracing::info_span!(target: "crontab_result", "crontab-result::delete", token_key = tk, username = un, query = ?query);
        async { rpc_exec!(delete::delete(token, query).await) }
            .instrument(span)
            .await
    }
}
