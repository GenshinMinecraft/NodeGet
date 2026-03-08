# 列出所有 Token

列出数据库中的所有 Token 信息。

## 方法

调用方法名为 `token_list_all_tokens`，需要提供以下参数：

```json
{
  "token": "demo_super_token"
}
```

## 权限要求

只有 **SuperToken** 可以调用该方法。

普通 Token 会返回权限错误。

## 返回结果

返回结构如下：

```json
{
  "tokens": [
    {
      "version": 1,
      "token_key": "n0kB8lSAykFd9Egu",
      "timestamp_from": null,
      "timestamp_to": null,
      "token_limit": [],
      "username": "root"
    },
    {
      "version": 1,
      "token_key": "demo_child_key",
      "timestamp_from": 1735689600000,
      "timestamp_to": 1767225600000,
      "token_limit": [
        {
          "scopes": [
            "global"
          ],
          "permissions": [
            {
              "task": "listen"
            }
          ]
        }
      ],
      "username": "gm"
    }
  ]
}
```

`token_secret` 和 `password` 不会在该接口中返回。
