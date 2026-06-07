### zed 需要在setting.json 配置
```json
{
  "lsp": {
    "rust-analyzer": {
      "initialization_options": {
        "check": {
          "allTargets": false,
          "extraArgs": [
            "--bins"
          ]
        }
      }
    }
  }
}
```
