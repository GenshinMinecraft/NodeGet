# Key-Value CRUD

## Set Key-Value

方法 `kv_set_value`，需要提供 `token`、`namespace`、`key`、`value`:

```json
{
    "token": "demo_token",
    "namespace": "kv_test",
    "key": "metadata_test",
    "value": [ // 任意类型 Json 数据
        12312313213
    ]
}
```

## Get Key-Value

方法 `kv_get_value`，需要提供 `token`、`namespace`、`key`:

```json
{
    "token": "demo_token",
    "namespace": "kv_test",
    "key": "metadata_test"
}
```

## Delete Key-Value

方法 `kv_delete_key`，需要提供 `token`、`namespace`、`key`:

```json
{
    "token": "demo_token",
    "namespace": "kv_test",
    "key": "metadata_test"
}
```

## List All Key-Value

方法 `kv_get_all_keys`，需要提供 `token`、`namespace`:

```json
{
    "token": "demo_token",
    "namespace": "kv_test"
}
```