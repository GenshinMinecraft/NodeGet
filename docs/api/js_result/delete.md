# 删除执行结果

调用者可以通过 `js-result_delete` 删除 `JsResult`。

需要传入 `token` / `query`：

```json
{
  "token": "demo_token",
  "query": {
    "condition": [
      {
        "js_worker_name": "demo_worker"
      },
      {
        "is_failure": null
      },
      {
        "limit": 100
      }
    ]
  }
}
```

返回结构：

```json
{
  "success": true,
  "deleted": 8,
  "condition_count": 3
}
```

说明：

- 删除语义与查询一致：`condition` 能查到什么，就删除什么。
- 当包含 `last` 或 `limit` 时，会先按 `start_time DESC, id DESC` 选中目标行再删除。

## 完整示例

### 删除某脚本最后一条结果

```json
{
  "jsonrpc": "2.0",
  "method": "js-result_delete",
  "params": {
    "token": "demo_token",
    "query": {
      "condition": [
        {
          "js_worker_name": "demo_worker"
        },
        {
          "last": null
        }
      ]
    }
  },
  "id": 1
}
```

### 删除某脚本最近 50 条失败结果

```json
{
  "jsonrpc": "2.0",
  "method": "js-result_delete",
  "params": {
    "token": "demo_token",
    "query": {
      "condition": [
        {
          "js_worker_name": "demo_worker"
        },
        {
          "is_failure": null
        },
        {
          "limit": 50
        }
      ]
    }
  },
  "id": 2
}
```

### 按 run_type 删除

```json
{
  "jsonrpc": "2.0",
  "method": "js-result_delete",
  "params": {
    "token": "demo_token",
    "query": {
      "condition": [
        {
          "js_worker_name": "demo_worker"
        },
        {
          "run_type": "inline_call"
        },
        {
          "limit": 20
        }
      ]
    }
  },
  "id": 3
}
```

## 权限要求

- 需要 `Permission::JsResult(JsResult::Delete("worker_name_or_pattern"))`。
- 作用域要求：`Scope::JsWorker(worker_name)`，支持后缀 `*` 通配。
