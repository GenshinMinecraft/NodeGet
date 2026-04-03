# 查询执行结果

调用者可以通过 `js-result_query` 查询 `JsResult`。

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
        "limit": 20
      }
    ]
  }
}
```

返回结构：

```json
[
  {
    "id": 1,
    "js_worker_id": 10,
    "js_worker_name": "demo_worker",
    "run_type": "call",
    "start_time": 1775000000000,
    "finish_time": 1775000000123,
    "param": {
      "hello": "world"
    },
    "result": {
      "ok": true
    },
    "error_message": null
  }
]
```

## 完整示例

### 查询某个脚本最近 10 条结果

```json
{
  "jsonrpc": "2.0",
  "method": "js-result_query",
  "params": {
    "token": "demo_token",
    "query": {
      "condition": [
        {
          "js_worker_name": "demo_worker"
        },
        {
          "limit": 10
        }
      ]
    }
  },
  "id": 1
}
```

### 查询运行中的记录

```json
{
  "jsonrpc": "2.0",
  "method": "js-result_query",
  "params": {
    "token": "demo_token",
    "query": {
      "condition": [
        {
          "js_worker_name": "demo_worker"
        },
        {
          "is_running": null
        }
      ]
    }
  },
  "id": 2
}
```

### 查询最后一条记录

```json
{
  "jsonrpc": "2.0",
  "method": "js-result_query",
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
  "id": 3
}
```

### 按 run_type 查询

```json
{
  "jsonrpc": "2.0",
  "method": "js-result_query",
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
  "id": 4
}
```

## 权限要求

- 需要 `Permission::JsResult(JsResult::Read("worker_name_or_pattern"))`。
- 作用域要求：`Scope::JsWorker(worker_name)`，支持后缀 `*` 通配。
