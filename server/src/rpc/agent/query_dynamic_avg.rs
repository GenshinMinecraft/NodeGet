use crate::entity::dynamic_monitoring;
use crate::rpc::RpcHelper;
use crate::rpc::agent::AgentRpcImpl;
use crate::rpc::agent::avg_utils::{JsonAverageAccumulator, ProcessCountAverageAccumulator};
use crate::token::get::check_token_limit;
use futures::StreamExt;
use jsonrpsee::core::RpcResult;
use log::error;
use nodeget_lib::error::NodegetError;
use nodeget_lib::monitoring::query::{DynamicDataAvgQuery, DynamicDataQueryField};
use nodeget_lib::permission::data_structure::{DynamicMonitoring, Permission, Scope};
use nodeget_lib::permission::token_auth::TokenOrAuth;
use nodeget_lib::utils::error_message::anyhow_error_to_raw;
use sea_orm::{
    ColumnTrait, DatabaseBackend, DatabaseConnection, EntityTrait, FromQueryResult, Order,
    QueryFilter, QueryOrder, QuerySelect, Statement,
};
use serde_json::value::RawValue;
use serde_json::{Map, Value};

#[derive(Debug, FromQueryResult)]
struct TimeRange {
    min_timestamp: Option<i64>,
    max_timestamp: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct JsonAggRow {
    data: Value,
}

enum FieldAverageAccumulator {
    Generic(JsonAverageAccumulator),
    SystemProcessCount(ProcessCountAverageAccumulator),
}

impl FieldAverageAccumulator {
    fn for_field(field: DynamicDataQueryField) -> Self {
        match field {
            DynamicDataQueryField::System => {
                Self::SystemProcessCount(ProcessCountAverageAccumulator::default())
            }
            DynamicDataQueryField::Cpu
            | DynamicDataQueryField::Ram
            | DynamicDataQueryField::Load
            | DynamicDataQueryField::Disk
            | DynamicDataQueryField::Network
            | DynamicDataQueryField::Gpu => Self::Generic(JsonAverageAccumulator::default()),
        }
    }

    fn add(&mut self, value: &Value) {
        match self {
            Self::Generic(acc) => acc.add(value),
            Self::SystemProcessCount(acc) => acc.add(value),
        }
    }

    fn finalize(&self) -> Value {
        match self {
            Self::Generic(acc) => acc.finalize(),
            Self::SystemProcessCount(acc) => acc.finalize(),
        }
    }
}

struct BucketAccumulator {
    timestamp_sum: i128,
    row_count: u64,
    fields: Vec<FieldAverageAccumulator>,
}

impl BucketAccumulator {
    fn new(selected_fields: &[DynamicDataQueryField]) -> Self {
        Self {
            timestamp_sum: 0,
            row_count: 0,
            fields: selected_fields
                .iter()
                .map(|field| FieldAverageAccumulator::for_field(*field))
                .collect(),
        }
    }

    fn add_row(
        &mut self,
        timestamp: i64,
        row_obj: &Map<String, Value>,
        selected_fields: &[DynamicDataQueryField],
    ) {
        self.timestamp_sum += i128::from(timestamp);
        self.row_count += 1;

        for (index, field) in selected_fields.iter().enumerate() {
            if let Some(value) = row_obj.get(field.column_name()) {
                self.fields[index].add(value);
            }
        }
    }

    fn into_json(self, uuid: &str, selected_fields: &[DynamicDataQueryField]) -> Value {
        let mut result = Map::new();
        result.insert("uuid".to_owned(), Value::String(uuid.to_owned()));
        let avg_timestamp = (self.timestamp_sum / i128::from(self.row_count)) as i64;
        result.insert("timestamp".to_owned(), Value::from(avg_timestamp));

        for (index, field) in selected_fields.iter().enumerate() {
            result.insert(field.json_key().to_owned(), self.fields[index].finalize());
        }

        Value::Object(result)
    }
}

pub async fn query_dynamic_avg(
    token: String,
    dynamic_data_avg_query: DynamicDataAvgQuery,
) -> RpcResult<Box<RawValue>> {
    let process_logic = async {
        validate_avg_query(&dynamic_data_avg_query)?;

        let token_or_auth = TokenOrAuth::from_full_token(&token)
            .map_err(|e| NodegetError::ParseError(format!("Failed to parse token: {e}")))?;

        let permissions: Vec<Permission> = dynamic_data_avg_query
            .fields
            .iter()
            .map(|field| Permission::DynamicMonitoring(DynamicMonitoring::Read(*field)))
            .collect();

        let is_allowed = check_token_limit(
            &token_or_auth,
            vec![Scope::AgentUuid(dynamic_data_avg_query.uuid)],
            permissions,
        )
        .await?;

        if !is_allowed {
            return Err(NodegetError::PermissionDenied(
                "Permission Denied: Insufficient DynamicMonitoring Read permissions".to_owned(),
            )
            .into());
        }

        let db = AgentRpcImpl::get_db()?;
        if db.get_database_backend() == DatabaseBackend::Postgres {
            return query_dynamic_avg_postgres(&db, &dynamic_data_avg_query).await;
        }

        let (min_timestamp, max_timestamp) = query_time_range(&db, &dynamic_data_avg_query).await?;
        let Some(min_timestamp) = min_timestamp else {
            return RawValue::from_string("[]".to_owned())
                .map_err(|e| NodegetError::SerializationError(e.to_string()).into());
        };
        let max_timestamp = max_timestamp.unwrap_or(min_timestamp);

        let points = dynamic_data_avg_query.points as usize;
        let mut buckets: Vec<Option<BucketAccumulator>> = (0..points).map(|_| None).collect();
        let mut query = dynamic_monitoring::Entity::find()
            .select_only()
            .column(dynamic_monitoring::Column::Timestamp)
            .filter(dynamic_monitoring::Column::Uuid.eq(dynamic_data_avg_query.uuid));

        if let Some(start) = dynamic_data_avg_query.timestamp_from {
            query = query.filter(dynamic_monitoring::Column::Timestamp.gte(start));
        }
        if let Some(end) = dynamic_data_avg_query.timestamp_to {
            query = query.filter(dynamic_monitoring::Column::Timestamp.lte(end));
        }

        for field in &dynamic_data_avg_query.fields {
            query = match field {
                DynamicDataQueryField::Cpu => query.column(dynamic_monitoring::Column::CpuData),
                DynamicDataQueryField::Ram => query.column(dynamic_monitoring::Column::RamData),
                DynamicDataQueryField::Load => query.column(dynamic_monitoring::Column::LoadData),
                DynamicDataQueryField::System => query.column(dynamic_monitoring::Column::SystemData),
                DynamicDataQueryField::Disk => query.column(dynamic_monitoring::Column::DiskData),
                DynamicDataQueryField::Network => {
                    query.column(dynamic_monitoring::Column::NetworkData)
                }
                DynamicDataQueryField::Gpu => query.column(dynamic_monitoring::Column::GpuData),
            };
        }

        let mut stream = query
            .order_by(dynamic_monitoring::Column::Timestamp, Order::Asc)
            .into_json()
            .stream(db)
            .await
            .map_err(|e| {
                error!("Database query error: {e}");
                NodegetError::DatabaseError(format!("Database query error: {e}"))
            })?;

        while let Some(item_res) = stream.next().await {
            let value = item_res.map_err(|e| {
                error!("Stream read error: {e}");
                NodegetError::DatabaseError(format!("Stream read error: {e}"))
            })?;

            let Some(obj) = value.as_object() else {
                continue;
            };
            let Some(timestamp) = obj.get("timestamp").and_then(Value::as_i64) else {
                continue;
            };

            let bucket_index = calc_bucket_index(
                timestamp,
                min_timestamp,
                max_timestamp,
                dynamic_data_avg_query.points,
            );

            if buckets[bucket_index].is_none() {
                buckets[bucket_index] = Some(BucketAccumulator::new(&dynamic_data_avg_query.fields));
            }

            if let Some(bucket) = buckets[bucket_index].as_mut() {
                bucket.add_row(timestamp, obj, &dynamic_data_avg_query.fields);
            }
        }

        let uuid = dynamic_data_avg_query.uuid.to_string();
        let rows: Vec<Value> = buckets
            .into_iter()
            .flatten()
            .map(|bucket| bucket.into_json(&uuid, &dynamic_data_avg_query.fields))
            .collect();

        let json = serde_json::to_string(&rows)
            .map_err(|e| NodegetError::SerializationError(format!("Serialization failed: {e}")))?;
        RawValue::from_string(json)
            .map_err(|e| NodegetError::SerializationError(format!("RawValue creation error: {e}")).into())
    };

    match process_logic.await {
        Ok(result) => Ok(result),
        Err(e) => {
            let raw = anyhow_error_to_raw(&e).unwrap_or_else(|_| {
                RawValue::from_string(r#"{"error_id":999,"error_message":"Internal error"}"#.to_owned())
                    .unwrap_or_else(|_| RawValue::from_string("null".to_owned()).unwrap())
            });
            let nodeget_err = nodeget_lib::error::anyhow_to_nodeget_error(&e);
            let json_str = raw.get();
            Err(jsonrpsee::types::ErrorObject::owned(
                nodeget_err.error_code() as i32,
                format!("{nodeget_err}"),
                Some(json_str),
            ))
        }
    }
}

fn validate_avg_query(query: &DynamicDataAvgQuery) -> anyhow::Result<()> {
    if query.fields.is_empty() {
        return Err(NodegetError::InvalidInput(
            "fields cannot be empty for dynamic_data_avg_query".to_owned(),
        )
        .into());
    }
    if query.points == 0 {
        return Err(NodegetError::InvalidInput("points must be >= 1".to_owned()).into());
    }
    if let (Some(start), Some(end)) = (query.timestamp_from, query.timestamp_to)
        && start > end
    {
        return Err(NodegetError::InvalidInput(
            "timestamp_from cannot be greater than timestamp_to".to_owned(),
        )
        .into());
    }
    Ok(())
}

async fn query_dynamic_avg_postgres(
    db: &DatabaseConnection,
    query: &DynamicDataAvgQuery,
) -> anyhow::Result<Box<RawValue>> {
    let sql = build_postgres_dynamic_avg_sql(&query.fields);
    let statement = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        [
            query.uuid.to_string().into(),
            query.timestamp_from.into(),
            query.timestamp_to.into(),
            i64::try_from(query.points)
                .map_err(|_| NodegetError::InvalidInput("points is too large".to_owned()))?
                .into(),
        ],
    );

    let row = JsonAggRow::find_by_statement(statement)
        .one(db)
        .await
        .map_err(|e| {
            error!("Failed to query dynamic avg in postgres: {e}");
            NodegetError::DatabaseError(format!("Failed to query dynamic avg in postgres: {e}"))
        })?;

    let json = row.map(|r| r.data).unwrap_or(Value::Array(Vec::new()));
    let json = serde_json::to_string(&json)
        .map_err(|e| NodegetError::SerializationError(format!("Serialization failed: {e}")))?;
    RawValue::from_string(json)
        .map_err(|e| NodegetError::SerializationError(format!("RawValue creation error: {e}")).into())
}

async fn query_time_range(
    db: &DatabaseConnection,
    query: &DynamicDataAvgQuery,
) -> anyhow::Result<(Option<i64>, Option<i64>)> {
    let mut range_query = dynamic_monitoring::Entity::find()
        .select_only()
        .column_as(
            dynamic_monitoring::Column::Timestamp.min(),
            "min_timestamp",
        )
        .column_as(
            dynamic_monitoring::Column::Timestamp.max(),
            "max_timestamp",
        )
        .filter(dynamic_monitoring::Column::Uuid.eq(query.uuid));

    if let Some(start) = query.timestamp_from {
        range_query = range_query.filter(dynamic_monitoring::Column::Timestamp.gte(start));
    }
    if let Some(end) = query.timestamp_to {
        range_query = range_query.filter(dynamic_monitoring::Column::Timestamp.lte(end));
    }

    let range = range_query.into_model::<TimeRange>().one(db).await.map_err(|e| {
        error!("Failed to query dynamic avg time range: {e}");
        NodegetError::DatabaseError(format!("Failed to query dynamic avg time range: {e}"))
    })?;

    Ok(
        range
            .map(|r| (r.min_timestamp, r.max_timestamp))
            .unwrap_or((None, None)),
    )
}

fn calc_bucket_index(timestamp: i64, min_timestamp: i64, max_timestamp: i64, points: u64) -> usize {
    if points <= 1 || min_timestamp >= max_timestamp {
        return 0;
    }

    let span = (i128::from(max_timestamp) - i128::from(min_timestamp)) + 1;
    let offset = (i128::from(timestamp) - i128::from(min_timestamp)).clamp(0, span - 1);
    let idx = (offset * i128::from(points)) / span;
    idx.min(i128::from(points - 1)) as usize
}

fn build_postgres_dynamic_avg_sql(fields: &[DynamicDataQueryField]) -> String {
    let select_columns = fields
        .iter()
        .map(|field| format!(", {}", field.column_name()))
        .collect::<String>();

    let aggregate_columns = fields
        .iter()
        .map(build_postgres_dynamic_field_aggregate_sql)
        .collect::<Vec<_>>()
        .join(",\n            ");

    let final_json_fields = fields
        .iter()
        .map(|field| format!(", '{}', agg.{}", field.json_key(), field.json_key()))
        .collect::<String>();

    let aggregate_columns = if aggregate_columns.is_empty() {
        String::new()
    } else {
        format!(",\n            {aggregate_columns}")
    };

    format!(
        r#"
WITH filtered AS MATERIALIZED (
    SELECT timestamp{select_columns}
    FROM dynamic_monitoring
    WHERE uuid = CAST($1 AS uuid)
      AND ($2::bigint IS NULL OR timestamp >= $2)
      AND ($3::bigint IS NULL OR timestamp <= $3)
),
bounds AS MATERIALIZED (
    SELECT MIN(timestamp) AS min_ts, MAX(timestamp) AS max_ts
    FROM filtered
),
bucketed AS MATERIALIZED (
    SELECT
        CASE
            WHEN bounds.min_ts IS NULL THEN NULL
            WHEN bounds.min_ts = bounds.max_ts OR $4::bigint <= 1 THEN 0
            ELSE LEAST(
                $4::bigint - 1,
                ((filtered.timestamp - bounds.min_ts) * $4::bigint) / ((bounds.max_ts - bounds.min_ts) + 1)
            )
        END AS bucket,
        filtered.timestamp{select_columns}
    FROM filtered
    CROSS JOIN bounds
),
agg AS (
    SELECT
        bucketed.bucket AS bucket,
        AVG(bucketed.timestamp)::bigint AS timestamp{aggregate_columns}
    FROM bucketed
    WHERE bucketed.bucket IS NOT NULL
    GROUP BY bucketed.bucket
    ORDER BY bucketed.bucket
)
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object(
            'uuid', $1::text,
            'timestamp', agg.timestamp{final_json_fields}
        )
        ORDER BY agg.bucket
    ),
    '[]'::jsonb
) AS data
FROM agg
"#
    )
}

fn build_postgres_dynamic_field_aggregate_sql(field: &DynamicDataQueryField) -> String {
    match field {
        DynamicDataQueryField::Cpu => r#"
jsonb_build_object(
    'per_core',
    (
        SELECT COALESCE(jsonb_agg(per_core.obj ORDER BY per_core.idx), '[]'::jsonb)
        FROM (
            SELECT
                arr.ord AS idx,
                jsonb_build_object(
                    'id', MIN(NULLIF(arr.elem->>'id', '')::numeric),
                    'cpu_usage', AVG(NULLIF(arr.elem->>'cpu_usage', '')::numeric),
                    'frequency_mhz', AVG(NULLIF(arr.elem->>'frequency_mhz', '')::numeric)
                ) AS obj
            FROM bucketed AS b2
            CROSS JOIN LATERAL jsonb_array_elements(COALESCE(b2.cpu_data->'per_core', '[]'::jsonb)) WITH ORDINALITY AS arr(elem, ord)
            WHERE b2.bucket = bucketed.bucket
            GROUP BY arr.ord
        ) AS per_core
    ),
    'total_cpu_usage', AVG(NULLIF(bucketed.cpu_data->>'total_cpu_usage', '')::numeric)
) AS cpu"#
            .to_owned(),
        DynamicDataQueryField::Ram => r#"
jsonb_build_object(
    'total_memory', AVG(NULLIF(bucketed.ram_data->>'total_memory', '')::numeric),
    'available_memory', AVG(NULLIF(bucketed.ram_data->>'available_memory', '')::numeric),
    'used_memory', AVG(NULLIF(bucketed.ram_data->>'used_memory', '')::numeric),
    'total_swap', AVG(NULLIF(bucketed.ram_data->>'total_swap', '')::numeric),
    'used_swap', AVG(NULLIF(bucketed.ram_data->>'used_swap', '')::numeric)
) AS ram"#
            .to_owned(),
        DynamicDataQueryField::Load => r#"
jsonb_build_object(
    'one', AVG(NULLIF(bucketed.load_data->>'one', '')::numeric),
    'five', AVG(NULLIF(bucketed.load_data->>'five', '')::numeric),
    'fifteen', AVG(NULLIF(bucketed.load_data->>'fifteen', '')::numeric)
) AS load"#
            .to_owned(),
        DynamicDataQueryField::System => r#"
jsonb_build_object(
    'process_count', AVG(NULLIF(bucketed.system_data->>'process_count', '')::numeric)
) AS system"#
            .to_owned(),
        DynamicDataQueryField::Disk => r#"
jsonb_build_object(
    'items',
    (
        SELECT COALESCE(jsonb_agg(disks.obj ORDER BY disks.idx), '[]'::jsonb)
        FROM (
            SELECT
                arr.ord AS idx,
                jsonb_build_object(
                    'kind', NULL,
                    'name', NULL,
                    'file_system', NULL,
                    'mount_point', NULL,
                    'total_space', AVG(NULLIF(arr.elem->>'total_space', '')::numeric),
                    'available_space', AVG(NULLIF(arr.elem->>'available_space', '')::numeric),
                    'is_removable', NULL,
                    'is_read_only', NULL,
                    'read_speed', AVG(NULLIF(arr.elem->>'read_speed', '')::numeric),
                    'write_speed', AVG(NULLIF(arr.elem->>'write_speed', '')::numeric)
                ) AS obj
            FROM bucketed AS b2
            CROSS JOIN LATERAL jsonb_array_elements(COALESCE(b2.disk_data, '[]'::jsonb)) WITH ORDINALITY AS arr(elem, ord)
            WHERE b2.bucket = bucketed.bucket
            GROUP BY arr.ord
        ) AS disks
    )
)->'items' AS disk"#
            .to_owned(),
        DynamicDataQueryField::Network => r#"
jsonb_build_object(
    'interfaces',
    (
        SELECT COALESCE(jsonb_agg(interfaces.obj ORDER BY interfaces.idx), '[]'::jsonb)
        FROM (
            SELECT
                arr.ord AS idx,
                jsonb_build_object(
                    'interface_name', NULL,
                    'total_received', AVG(NULLIF(arr.elem->>'total_received', '')::numeric),
                    'total_transmitted', AVG(NULLIF(arr.elem->>'total_transmitted', '')::numeric),
                    'receive_speed', AVG(NULLIF(arr.elem->>'receive_speed', '')::numeric),
                    'transmit_speed', AVG(NULLIF(arr.elem->>'transmit_speed', '')::numeric)
                ) AS obj
            FROM bucketed AS b2
            CROSS JOIN LATERAL jsonb_array_elements(COALESCE(b2.network_data->'interfaces', '[]'::jsonb)) WITH ORDINALITY AS arr(elem, ord)
            WHERE b2.bucket = bucketed.bucket
            GROUP BY arr.ord
        ) AS interfaces
    ),
    'udp_connections', AVG(NULLIF(bucketed.network_data->>'udp_connections', '')::numeric),
    'tcp_connections', AVG(NULLIF(bucketed.network_data->>'tcp_connections', '')::numeric)
) AS network"#
            .to_owned(),
        DynamicDataQueryField::Gpu => r#"
jsonb_build_object(
    'items',
    (
        SELECT COALESCE(jsonb_agg(gpus.obj ORDER BY gpus.idx), '[]'::jsonb)
        FROM (
            SELECT
                arr.ord AS idx,
                jsonb_build_object(
                    'id', MIN(NULLIF(arr.elem->>'id', '')::numeric),
                    'used_memory', AVG(NULLIF(arr.elem->>'used_memory', '')::numeric),
                    'total_memory', AVG(NULLIF(arr.elem->>'total_memory', '')::numeric),
                    'graphics_clock_mhz', AVG(NULLIF(arr.elem->>'graphics_clock_mhz', '')::numeric),
                    'sm_clock_mhz', AVG(NULLIF(arr.elem->>'sm_clock_mhz', '')::numeric),
                    'memory_clock_mhz', AVG(NULLIF(arr.elem->>'memory_clock_mhz', '')::numeric),
                    'video_clock_mhz', AVG(NULLIF(arr.elem->>'video_clock_mhz', '')::numeric),
                    'utilization_gpu', AVG(NULLIF(arr.elem->>'utilization_gpu', '')::numeric),
                    'utilization_memory', AVG(NULLIF(arr.elem->>'utilization_memory', '')::numeric),
                    'temperature', AVG(NULLIF(arr.elem->>'temperature', '')::numeric)
                ) AS obj
            FROM bucketed AS b2
            CROSS JOIN LATERAL jsonb_array_elements(COALESCE(b2.gpu_data, '[]'::jsonb)) WITH ORDINALITY AS arr(elem, ord)
            WHERE b2.bucket = bucketed.bucket
            GROUP BY arr.ord
        ) AS gpus
    )
)->'items' AS gpu"#
            .to_owned(),
    }
}
