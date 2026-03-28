# 获取 Runtime 池信息

调用者可以通过 `js-worker_get_rt_pool` 查看当前 JS Runtime 池状态。

需要传入 `token`：

```json
{
  "token": "demo_token"
}
```

返回结构：

```json
{
  "total_workers": 2,
  "workers": [
    {
      "script_name": "demo_worker",
      "active_requests": 0,
      "last_used_ms": 1774652000123,
      "idle_ms": 4200,
      "runtime_clean_time_ms": 60000
    }
  ]
}
```

## 完整示例

```json
{
  "jsonrpc": "2.0",
  "method": "js-worker_get_rt_pool",
  "params": {
    "token": "demo_token"
  },
  "id": 1
}
```

## 权限要求

- 需要 `Permission::NodeGet(NodeGet::GetRtPool)`。
- 建议在 `Scope::Global` 下授予。
