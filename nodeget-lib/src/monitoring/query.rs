use serde::{Deserialize, Serialize};
use serde_json::Value;

// 静态监控数据查询字段枚举，定义可查询的静态数据类型
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum StaticDataQueryField {
    // CPU 相关信息
    Cpu,
    // 系统相关信息
    System,
    // GPU 相关信息
    Gpu,
}

impl StaticDataQueryField {
    /// 获取字段对应的数据库列名
    #[must_use]
    pub const fn column_name(&self) -> &'static str {
        match self {
            Self::Cpu => "cpu_data",
            Self::System => "system_data",
            Self::Gpu => "gpu_data",
        }
    }

    /// 获取字段的 JSON 键名
    #[must_use]
    pub const fn json_key(&self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::System => "system",
            Self::Gpu => "gpu",
        }
    }
}

// 动态监控数据查询字段枚举，定义可查询的动态数据类型
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DynamicDataQueryField {
    // CPU 相关信息
    Cpu,
    // 内存相关信息
    Ram,
    // 系统负载相关信息
    Load,
    // 系统相关信息
    System,
    // 磁盘相关信息
    Disk,
    // 网络相关信息
    Network,
    // GPU 相关信息
    Gpu,
}

impl DynamicDataQueryField {
    /// 获取字段对应的数据库列名
    #[must_use]
    pub const fn column_name(&self) -> &'static str {
        match self {
            Self::Cpu => "cpu_data",
            Self::Ram => "ram_data",
            Self::Load => "load_data",
            Self::System => "system_data",
            Self::Disk => "disk_data",
            Self::Network => "network_data",
            Self::Gpu => "gpu_data",
        }
    }

    /// 获取字段的 JSON 键名
    #[must_use]
    pub const fn json_key(&self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Ram => "ram",
            Self::Load => "load",
            Self::System => "system",
            Self::Disk => "disk",
            Self::Network => "network",
            Self::Gpu => "gpu",
        }
    }
}

// 查询条件枚举，定义各种查询过滤条件
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryCondition {
    // 按 UUID 过滤
    Uuid(uuid::Uuid),
    // 按时间戳范围过滤（开始时间，结束时间）
    TimestampFromTo(i64, i64), // start, end
    // 按时间戳起始点过滤
    TimestampFrom(i64), // start,
    // 按时间戳结束点过滤
    TimestampTo(i64), // end

    // 限制返回结果数量
    Limit(u64), // limit

    // 获取最后一条记录
    Last,
}

// 静态监控数据查询结构体
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticDataQuery {
    // 要查询的字段列表
    pub fields: Vec<StaticDataQueryField>,
    // 查询条件列表
    pub condition: Vec<QueryCondition>,
}

// 动态监控数据查询结构体
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicDataQuery {
    // 要查询的字段列表
    pub fields: Vec<DynamicDataQueryField>,
    // 查询条件列表
    pub condition: Vec<QueryCondition>,
}

// 静态监控数据响应项结构体
#[derive(Serialize)]
pub struct StaticResponseItem {
    // 设备 UUID
    pub uuid: String,
    // 时间戳
    pub timestamp: i64,
    // CPU 数据，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<Value>,
    // 系统数据，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Value>,
    // GPU 数据，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu: Option<Value>,
}

// 动态监控数据响应项结构体
#[derive(Serialize)]
pub struct DynamicResponseItem {
    // 设备 UUID
    pub uuid: String,
    // 时间戳
    pub timestamp: i64,
    // CPU 数据，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<Value>,
    // 内存数据，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram: Option<Value>,
    // 负载数据，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load: Option<Value>,
    // 系统数据，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Value>,
    // 磁盘数据，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk: Option<Value>,
    // 网络数据，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Value>,
    // GPU 数据，可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu: Option<Value>,
}
