# 编辑 Token 权限

修改指定 Token 的 `token_limit`。

## 方法

调用方法名为 `token_edit`，需要提供以下参数：

```json
{
  "token": "demo_super_token",
  "target_token": "target_token_key_or_username",
  "limit": [
    {
      "scopes": [
        "global"
      ],
      "permissions": [
        {
          "task": {
            "read": "ping"
          }
        }
      ]
    }
  ]
}
```

## 权限要求

只有 **SuperToken** 可以调用该方法。

`target_token` 支持两种匹配方式：

- `token_key`
- `username`

服务端会先按 `token_key` 匹配；若未命中，再按 `username` 匹配。

## 返回结果

```json
{
  "success": true,
  "id": 2,
  "token_key": "BgFqEhzoCISpAAON"
}
```

该方法会覆盖目标 Token 的 `token_limit` 字段。
