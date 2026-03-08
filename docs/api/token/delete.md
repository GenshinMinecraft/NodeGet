# 删除 Token

删除指定的 Token。

## 方法

调用方法名为 `token_delete`，需要提供以下参数：

```json
{
  "token": "demo_super_token",
  "target_token": "target_token_key_or_username"
}
```

## 权限要求

只有 **SuperToken** 可以删除 Token。

`target_token` 支持两种匹配方式：

- `token_key`
- `username`

服务端会先按 `token_key` 匹配；若未命中，再按 `username` 匹配。
