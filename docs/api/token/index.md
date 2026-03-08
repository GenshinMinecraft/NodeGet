# Token 鎬昏

Token 鏄湰椤圭洰鐨勯壌鏉冩牳蹇冿紝浠讳綍鏈夋潈闄愮殑鎿嶄綔閮藉簲鎸佹湁 鏈夊搴旀潈闄愮殑 Token

## Token 鍒嗙被

鍦ㄦ湰椤圭洰锛孴oken 鍙互鍒嗕负涓ょ被

- SuperToken: 鍦?Server 鍒濆鍖栨椂鍒涘缓鐨勫敮涓€鍊硷紝鏁版嵁搴?ID 涓?1 鐨?Token锛屽湪鎵€鏈夋搷浣滀腑璇?Token 鐩存帴鏀捐
- Token: 鐢?SuperToken 鍒涘缓鐨勫瓙 Token

Token 鍙互鏄笅鍒楀€?

- `TOKEN_KEY:TOKEN_SECRET`: Token Key 鏄庢枃鍌ㄥ瓨锛孴oken Secret 涓轰富瑕侀壌鏉冮儴鍒?
- `Username|Password`: Username 鏄庢枃鍌ㄥ瓨锛孭assword 涓轰富瑕侀壌鏉冮儴鍒?

鍖哄埆浣嶄簬鍒嗛殧绗︿笉鍚岋紝鍦?Username+Password 鏂规涓紝鍙彇绗竴涓垎闅旂 `|`锛屽悗闈綔涓?Password

鐗圭偣:

- Token 涓?Username+Password 绛変环锛屼絾 Server 鍐呴儴閴存潈鍙湁 Token銆傚湪浠讳綍 API 涓袱绉嶅舰寮忓潎鍙?
- Token 涓?Username 涓€涓€瀵瑰簲锛孲uperToken 瀵瑰簲鐨?Username 涓?root
- Token 涓嶅彲鍙樹笖涓嶅彲鎸囧畾锛屼絾 Username+Password 鍙互鑷鏇存敼

## 鍩烘湰缁撴瀯

涓€涓?Token 瀵瑰簲濡備笅缁撴瀯浣?

```rust
pub struct Token {
    pub version: u8, // 鏆傛椂涓?1
    pub token_key: String, // 鏍囪瘑 Token 鏈€涓昏鐨勯敭
    pub timestamp_from: Option<i64>, // Token 鏈夋晥鏈燂紝姣鏃堕棿鎴?
    pub timestamp_to: Option<i64>,
    pub token_limit: Vec<Limit>, // 鏉冮檺鑼冨洿
    pub username: Option<String>, // 鐢ㄦ埛鍚?
}
```

Token Secret 涓?Password 瀛樹簬鏁版嵁搴撲腑锛屾棤鍙嶅悜瑙ｆ瀽

涓€涓?Token 鍙互瀵瑰簲澶氫釜 Limit锛屽湪涓嶅悓鐨勪綔鐢ㄥ煙 (Scope) 涓嬫湁涓嶅悓鐨勬潈闄?(Permission)

### Limit

涓€涓?Limit 瀵瑰簲澶氫釜 Scope 涓?Permission

```rust
pub struct Limit {
    pub scopes: Vec<Scope>,
    pub permissions: Vec<Permission>,
}
```

### Scope

Scope 涓轰綔鐢ㄥ煙锛屽嵆琛ㄧず鍦ㄦ煇涓€涓璞?(鐩墠涓?Agent Uuid) 鏈夋潈闄?

```rust
pub enum Scope {
    // 鍏ㄥ眬浣滅敤鍩燂紝閫傜敤浜庢墍鏈夊湴鐐?
    Global,
    // 鐗瑰畾 Agent 浣滅敤鍩燂紝閫氳繃 UUID 鎸囧畾
    AgentUuid(uuid::Uuid),
    // KvNamespace 浣滅敤鍩燂紝閫氳繃鍚嶇О鎸囧畾
    KvNamespace(String),
}
```

### Permission

```rust
// 鏉冮檺鏋氫妇锛屽畾涔変笉鍚岀被鍨嬬殑鎿嶄綔鏉冮檺
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    // 闈欐€佺洃鎺ф潈闄?
    StaticMonitoring(StaticMonitoring),
    // 鍔ㄦ€佺洃鎺ф潈闄?
    DynamicMonitoring(DynamicMonitoring),
    // 浠诲姟鏉冮檺
    Task(Task),
    // Crontab 鏉冮檺
    Crontab(Crontab),

    // Kv 鏉冮檺
    Kv(Kv),

    // Terminal 鏉冮檺
    Terminal(Terminal),

    // CrontabResult 鏉冮檺
    CrontabResult(CrontabResult),
    
    // NodeGet 鏉冮檺
    NodeGet(NodeGet),
}

// 闈欐€佺洃鎺ф潈闄愭灇涓?
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaticMonitoring {
    // 璇诲彇鏉冮檺锛屾寚瀹氬彲璇诲彇鐨勫瓧娈电被鍨?
    Read(StaticDataQueryField),
    // 鍐欏叆鏉冮檺
    Write,
}

// 鍔ㄦ€佺洃鎺ф潈闄愭灇涓?
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DynamicMonitoring {
    // 璇诲彇鏉冮檺锛屾寚瀹氬彲璇诲彇鐨勫瓧娈电被鍨?
    Read(DynamicDataQueryField),
    // 鍐欏叆鏉冮檺
    Write,
}

// 浠诲姟鏉冮檺鏋氫妇
// Type 瀛楁鍚?
// 鎺ュ彈 ping / tcp_ping / http_ping / web_shell / execute / ip
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Task {
    // 鍒涘缓鏉冮檺锛屾寚瀹氫换鍔＄被鍨?
    Create(String),
    // 璇诲彇鏉冮檺锛屾寚瀹氫换鍔＄被鍨?
    Read(String),
    // 鍐欏叆鏉冮檺锛屾寚瀹氫换鍔＄被鍨?
    Write(String),
    // 鐩戝惉鏉冮檺
    Listen,
}

// Crontab 鏉冮檺鏋氫妇
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Crontab {
    // 鍙互璇诲彇鍦ㄨ嚜宸?Scope 涓嬬殑鎵€鏈?Crontab
    Read,
    // 鍙互鍒涘缓 Crontab
    // 鑻?Crontab 绫诲瀷涓轰笅鍙戠粰 Agent 浠诲姟锛屽垯璇?Token 杩樺繀椤绘嫢鏈夊搴?Agent 鐨?Task Create 鏉冮檺
    // 鑻?Crontab 绫诲瀷涓?Server 浠诲姟锛屽垯 Scope 蹇呴』涓?Global锛屽惁鍒欐棤鏁?
    Write,
    // 鍒犻櫎 Crontab
    Delete,
}

// CrontabResult 鏉冮檺鏋氫妇
// 娉ㄦ剰锛氳鏉冮檺浠呭湪 Global Scope 涓嬫湁鏁?
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrontabResult {
    // 璇诲彇鏉冮檺锛屾寚瀹氬彲璇诲彇鐨?cron_name
    Read(String),
    // 鍒犻櫎鏉冮檺锛屾寚瀹氬彲鍒犻櫎鐨?cron_name
    Delete(String),
}

// Kv 鏉冮檺鏋氫妇
// 娉ㄦ剰锛氳鏉冮檺浠呭湪 Global 鎴?KvNamespace Scope 涓嬫湁鏁?
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kv {
    // 鍒楀嚭璇?Namespace 涓嬫墍鏈夐敭
    ListAllKeys,
    // 涓嬮潰涓?KV 鏁版嵁搴撶殑 CRUD 鎿嶄綔
    // Write 鍦ㄩ亣鍒板悓鍚?Key 浼氳鐩栨搷浣?
    // 鍙互鎷ユ湁閫氶厤绗︼紝姣斿 `metadata_*`锛岃〃杈惧彲浠ユ搷浣?杩欎竴 KvNamespace Scope 涓嬬殑鎵€鏈変互 `metadata_` 寮€澶寸殑閿?
    Read(String),
    Write(String),
    Delete(String),
}

// Terminal 鏉冮檺鏋氫妇
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terminal {
    // 鍦?Agent Uuid 涓嬫嫢鏈夎鏉冮檺锛岃〃鏄庡彲浠ラ€氳繃璇?Token 杩炴帴鍒拌 Agent 鐨?Terminal
    // Global Scope 涓嬪彲浠ヨ繛鎺ュ埌鎵€鏈夌殑 Agent
    // 娉ㄦ剰锛氭澶勫彧鏄繛鎺ワ紝鑰屼笉鏄垱寤烘垨涓诲姩璁?Agent 杩炴帴
    Connect,
}

// NodeGet 鏉冮檺鏋氫妇
// 娉ㄦ剰锛氳鏉冮檺浠呭湪 Global Scope 涓嬫湁鏁?
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeGet {
    // 鍒楀嚭鎵€鏈?Agent Uuid
    ListAllAgentUuid,
}

```

鑻ュ瓨鍦ㄤ簬 Limit 鐨?permissions 涓紝鍗充负鎷ユ湁璇ユ潈闄?

## Demo

### Agent 鍩虹

鐜版湁杩欎箞涓€涓粨鏋勪綋

```json
{
  "scopes": [
    {
      "agent_uuid": "adf78235-a23c-46fc-bc85-694f64c39aaf"
    },
    {
      "agent_uuid": "33c1b63a-35f1-4b9f-9659-66e7a3e5a75c"
    }
  ],
  "permissions": [
    {
      "dynamic_monitoring": "write"
    },
    {
      "static_monitoring": "write"
    },
    {
      "task": "listen"
    },
    {
      "task": {
        "write": "ping"
      }
    },
    {
      "task": {
        "write": "tcp_ping"
      }
    },
    {
      "task": {
        "write": "http_ping"
      }
    },
    {
      "task": {
        "write": "web_shell"
      }
    },
    {
      "task": {
        "write": "execute"
      }
    },
    {
      "task": {
        "write": "ip"
      }
    }
  ]
}
```

杩欐槸涓€涓?Agent 鑳芥甯歌皟鐢ㄦ墍鏈夊姛鑳界殑 Limit锛屽畠琛ㄧず:

Agent Uuid 涓?`ad..af` 涓?`33..5c` 鐨?Agent锛屽叿鏈変笂浼?StaticMonitoring / DynamicMonitoring 鏁版嵁銆佺洃鍚?Server 涓嬪彂
Task銆佷笂鎶ョ洰鍓嶆墍鏈?Task 浠诲姟绫诲瀷 鐨勬潈闄?

### 鏌ヨ 鍩虹

鐜版湁杩欎箞涓€涓粨鏋勪綋

```json
{
  "scopes": [
    {
      "agent_uuid": "53f125b6-e7aa-447f-a27c-085a53a36462"
    },
    {
      "agent_uuid": "3e6f227f-56e3-4ca0-a12f-04014ebeebe7"
    }
  ],
  "permissions": [
    {
      "dynamic_monitoring": {
        "read": "cpu"
      }
    },
    {
      "dynamic_monitoring": {
        "read": "system"
      }
    },
    {
      "static_monitoring": {
        "read": "cpu"
      }
    },
    {
      "static_monitoring": {
        "read": "system"
      }
    }
  ]
}
```

瀹冭〃绀?

鐢ㄦ埛鍙互鏌ヨ Agent Uuid 涓?`ad..af` 涓?`33..5c` 鐨?Agent 鐨?StaticMonitoring / DynamicMonitoring Data 涓?cpu / system 瀛楁

### Crontab 鏉冮檺绀轰緥

鐜版湁杩欎箞涓€涓粨鏋勪綋

```json
{
  "scopes": [
    {
      "global": null
    }
  ],
  "permissions": [
    {
      "crontab": "read"
    },
    {
      "crontab": "write"
    },
    {
      "crontab": "delete"
    }
  ]
}
```

杩欐槸涓€涓叿鏈夊叏灞€ Crontab 鏉冮檺鐨?Limit锛屽畠琛ㄧず:

鍏锋湁瀵规墍鏈?Crontab 鐨勮鍙栥€佸啓鍏ュ拰鍒犻櫎鏉冮檺銆?

鎴栭拡瀵圭壒瀹?Agent 鐨勬潈闄?

```json
{
  "scopes": [
    {
      "agent_uuid": "00000000-0000-0000-0000-000000000001"
    },
    {
      "agent_uuid": "00000000-0000-0000-0000-000000000002"
    }
  ],
  "permissions": [
    {
      "crontab": "read"
    },
    {
      "crontab": "write"
    }
  ]
}
```

杩欒〃绀?

瀵?UUID 涓?`00000000-0000-0000-0000-000000000001` 鍜?`00000000-0000-0000-0000-000000000002` 鐨?Agent 鐩稿叧鐨?Crontab
鍏锋湁璇诲彇鍜屽啓鍏ユ潈闄愩€?

## Token API 鏂规硶

- [鍒涘缓 Token](./create.md)
- [鑾峰彇 Token 璇︽儏](./get.md)
- [鍒犻櫎 Token](./delete.md)
- [鍒楀嚭鎵€鏈?Token](./list_all_tokens.md)
- [编辑 Token 权限](./edit.md)


