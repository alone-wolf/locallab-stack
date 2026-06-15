# 阶段 0007：证书集成

## 目标

集成本地 HTTPS 证书流程，使 LocalLabStack 的 Caddy 网关可以使用本地受信任证书。第一版以 `mkcert` 为主，保持实现实用且可理解。

## 前置条件

- 阶段 0000 到 0006 已完成。
- `.locallab/certs/issued/` 目录由 `llstk init` 创建。
- 根清单中存在 cert provider 和 domains 配置。

## 命令规格

```bash
llstk cert init
llstk cert issue [--force]
llstk cert status
```

命令含义：

- `cert init`：检查并初始化 `mkcert` 本地 CA 信任。
- `cert issue`：为根清单中的 domains 签发证书。
- `cert status`：只读检查证书文件、provider、domains 和 mkcert 可用性。

## 证书 provider

第一版只支持：

```yaml
cert:
  provider: mkcert
```

如果 provider 不是 `mkcert`，命令应失败并提示 unsupported cert provider。

## 生成文件

```text
.locallab/certs/
  issued/
    locallab.pem
    locallab-key.pem
```

注意：

- `.locallab/certs/issued/locallab-key.pem` 是私钥，必须提醒用户不要提交。
- 如果项目 `.gitignore` 尚未覆盖 `.locallab/certs/issued/*.pem` 或 `.locallab/`，本阶段应补充合适规则。

## 外部命令策略

必须通过 `cert` 模块集中调用 `mkcert`，不允许在命令层散落 `Command::new("mkcert")`。

建议命令：

```bash
mkcert -install
mkcert -cert-file .locallab/certs/issued/locallab.pem \
       -key-file .locallab/certs/issued/locallab-key.pem \
       locallab gitea.locallab
```

实际实现必须处理路径中包含空格的情况，不能拼接 shell 字符串执行。

## 写入策略

- `cert issue` 发现证书和私钥都已存在时，默认跳过并提示。
- `cert issue --force` 可以重新生成证书和私钥。
- 只存在证书或只存在私钥时，应失败并提示不完整状态；用户传入 `--force` 才覆盖。
- 不删除 CA 文件。

## `cert status`

至少输出：

- provider。
- domains。
- `mkcert` 是否可用。
- issued cert 是否存在。
- issued key 是否存在。
- 如果可行，输出证书过期时间；不能解析时给 warning，不使整个命令失败。

示例：

```text
provider: mkcert
domains: locallab, gitea.locallab
mkcert: available
certificate: present .locallab/certs/issued/locallab.pem
private key: present .locallab/certs/issued/locallab-key.pem
```

## 具体任务

- [ ] 实现 mkcert 命令封装。
- [ ] 实现 `cert init`。
- [ ] 实现 `cert issue`。
- [ ] 实现 `cert status`。
- [ ] 实现证书文件状态检测。
- [ ] 必要时更新 `.gitignore`。
- [ ] 添加不依赖真实 mkcert 的 command builder 测试。
- [ ] 添加证书状态逻辑测试。

## 错误场景

- 未初始化工作区时，提示先运行 `llstk init`。
- 根清单 cert provider 非 `mkcert` 时失败。
- `mkcert` 不存在时，命令失败并提示安装方式应查看 mkcert 文档。
- 证书目录不存在时，应尝试创建；创建失败则显示路径。
- 证书状态不完整且没有 `--force` 时失败。
- mkcert 返回非 0 时，保留 stderr。

## 测试要求

至少覆盖：

- cert command builder 生成正确参数，不使用 shell 拼接。
- provider 非 mkcert 时失败。
- 证书和私钥都不存在时状态为 missing。
- 只存在证书时状态为 incomplete。
- 已存在完整证书时 `cert issue` 默认跳过。
- `cert issue --force` 会构造重新签发命令。
- `.gitignore` 包含私钥保护规则。

真实 mkcert 执行可以不纳入默认测试，但必须文档化手动验证步骤。

## 验收标准

本阶段完成时，在安装 mkcert 的机器上可以运行：

```bash
cargo run --bin llstk -- init
cargo run --bin llstk -- cert status
cargo run --bin llstk -- cert init
cargo run --bin llstk -- cert issue
cargo run --bin llstk -- cert status
```

并生成：

```text
.locallab/certs/issued/locallab.pem
.locallab/certs/issued/locallab-key.pem
```

## 本阶段不做

- 不实现 OpenSSL 手动 CA 模式。
- 不自动安装 mkcert。
- 不提交或展示私钥内容。
- 不修改系统 hosts。
- 不自动 reload Caddy。

## 交付物

- mkcert 封装。
- `cert init`、`cert issue`、`cert status`。
- 私钥保护规则。
- 覆盖证书状态和命令构造的测试。
