# 删除脚本

调用者可以通过 `js-worker_delete` 删除脚本。

需要传入 `token` / `name`：

```json
{
  "token": "demo_token",
  "name": "demo_worker"
}
```

返回结构：

```json
{
  "success": true,
  "rows_affected": 1
}
```

说明：

- 删除成功后，脚本对应的 Runtime 实例会被立即驱逐。

## 完整示例

```json
{
  "jsonrpc": "2.0",
  "method": "js-worker_delete",
  "params": {
    "token": "demo_token",
    "name": "demo_worker"
  },
  "id": 1
}
```

## 权限要求

- 需要 `Permission::JsWorker(JsWorker::Delete)`。
- 作用域要求：`Scope::JsWorker(name)`。
