use crate::entity::dynamic_monitoring_summary;
use crate::rpc::RpcHelper;
use crate::rpc::agent::AgentRpcImpl;
use crate::token::get::check_token_limit;
use jsonrpsee::core::RpcResult;
use nodeget_lib::error::NodegetError;
use nodeget_lib::monitoring::data_structure::DynamicMonitoringSummaryData;
use nodeget_lib::permission::data_structure::{
    DynamicMonitoringSummary, Permission, Scope,
};
use nodeget_lib::permission::token_auth::TokenOrAuth;
use sea_orm::{ActiveValue, EntityTrait, Set};
use serde_json::value::RawValue;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::debug;

static ERROR_COUNTER: AtomicU64 = AtomicU64::new(0);
fn generate_error_id() -> u64 {
    ERROR_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub async fn report_dynamic_summary(
    token: String,
    data: DynamicMonitoringSummaryData,
) -> RpcResult<Box<RawValue>> {
    let process_logic = async {
        let agent_uuid = uuid::Uuid::from_str(&data.uuid)
            .map_err(|e| NodegetError::ParseError(format!("Invalid UUID format: {e}")))?;

        let token_or_auth = TokenOrAuth::from_full_token(&token)
            .map_err(|e| NodegetError::ParseError(format!("Failed to parse token: {e}")))?;

        let is_allowed = check_token_limit(
            &token_or_auth,
            vec![Scope::AgentUuid(agent_uuid)],
            vec![Permission::DynamicMonitoringSummary(
                DynamicMonitoringSummary::Write,
            )],
        )
        .await?;

        if !is_allowed {
            return Err(NodegetError::PermissionDenied(
                "Permission Denied: Missing DynamicMonitoringSummary Write permission for this Agent"
                    .to_owned(),
            )
            .into());
        }

        let db = AgentRpcImpl::get_db()?;

        let in_data = dynamic_monitoring_summary::ActiveModel {
            id: ActiveValue::default(),
            uuid: Set(agent_uuid.to_string()),
            timestamp: Set(data.time.cast_signed()),
            cpu_usage: Set(data.cpu_usage),
            gpu_usage: Set(data.gpu_usage),
            used_swap: Set(data.used_swap),
            total_swap: Set(data.total_swap),
            used_memory: Set(data.used_memory),
            total_memory: Set(data.total_memory),
            available_memory: Set(data.available_memory),
            load_one: Set(data.load_one),
            load_five: Set(data.load_five),
            load_fifteen: Set(data.load_fifteen),
            uptime: Set(data.uptime),
            boot_time: Set(data.boot_time),
            process_count: Set(data.process_count),
            total_space: Set(data.total_space),
            available_space: Set(data.available_space),
            read_speed: Set(data.read_speed),
            write_speed: Set(data.write_speed),
            tcp_connections: Set(data.tcp_connections),
            udp_connections: Set(data.udp_connections),
            total_received: Set(data.total_received),
            total_transmitted: Set(data.total_transmitted),
            transmit_speed: Set(data.transmit_speed),
            receive_speed: Set(data.receive_speed),
        };

        debug!(target: "monitoring", agent_uuid = %data.uuid, "Received dynamic summary data");

        let result = dynamic_monitoring_summary::Entity::insert(in_data)
            .exec(db)
            .await
            .map_err(|e| {
                let error_id = generate_error_id();
                tracing::error!(target: "monitoring", error_id = error_id, error = %e, "Database insert error");
                NodegetError::DatabaseError(format!(
                    "Database error occurred. Reference: {error_id}"
                ))
            })?;

        debug!(target: "monitoring", id = result.last_insert_id, "Inserted dynamic summary data");

        let json_str = format!("{{\"id\":{}}}", result.last_insert_id);
        RawValue::from_string(json_str)
            .map_err(|e| NodegetError::SerializationError(e.to_string()).into())
    };

    match process_logic.await {
        Ok(result) => Ok(result),
        Err(e) => {
            let nodeget_err = nodeget_lib::error::anyhow_to_nodeget_error(&e);
            Err(jsonrpsee::types::ErrorObject::owned(
                nodeget_err.error_code() as i32,
                format!("{nodeget_err}"),
                None::<()>,
            ))
        }
    }
}
