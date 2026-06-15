# 阶段 0005：Hosts 规划

## 目标

实现只读的 hosts 规划与状态检查，让用户在写入 `/etc/hosts` 前明确知道 LocalLabStack 需要哪些本地域名解析记录。

## 前置条件

- 阶段 0000 到 0004 已完成。
- 可以扫描和读取应用清单。
- 根清单中存在 `hosts.enabled` 和 `hosts.ip`。

## 命令规格

```bash
llstk hosts plan [--format text|block]
llstk hosts status [--hosts-file <path>]
```

参数：

- `--format text`：默认格式，每行一条记录。
- `--format block`：输出完整受管理区块。

本阶段严禁写入 `/etc/hosts`。

## 规划规则

- 只包含有公开 upstream 的应用域名。
- 如果根清单 `hosts.enabled` 为 `false`，`hosts plan` 应明确输出 hosts 管理已禁用，并返回 0。
- IP 使用根清单 `hosts.ip`，默认 `127.0.0.1`。
- 域名按字典序排序。
- 重复域名去重。

## 输出示例

默认 text：

```text
127.0.0.1 gitea.locallab
127.0.0.1 minio.locallab
```

block：

```text
# BEGIN LocalLabStack
127.0.0.1 gitea.locallab
127.0.0.1 minio.locallab
# END LocalLabStack
```

无公开应用：

```text
no public app domains found
```

## `hosts status`

`hosts status` 应只读取 `/etc/hosts`，不写入。

参数：

- `--hosts-file <path>`：用于测试或高级场景，默认 `/etc/hosts`。阶段 0008 的 `hosts sync` 必须复用同一个参数语义。

至少输出：

- hosts 管理是否启用。
- 计划记录数量。
- `/etc/hosts` 中是否存在 LocalLabStack 受管理区块。
- 当前区块是否与计划一致。

示例：

```text
enabled: true
planned records: 1
managed block: present
status: up to date
```

如果无权限读取 `/etc/hosts`，应失败并提示路径。

## 受管理区块格式

必须固定为：

```text
# BEGIN LocalLabStack
<ip> <domain>
# END LocalLabStack
```

后续阶段 `hosts sync` 必须复用同一个生成函数。

## 具体任务

- [ ] 实现 hosts 计划记录模型。
- [ ] 实现从应用清单提取公开域名。
- [ ] 实现 hosts block 渲染函数。
- [ ] 实现 `/etc/hosts` 受管理区块解析函数。
- [ ] 实现 `hosts plan`。
- [ ] 实现 `hosts status`。
- [ ] 添加 hosts 计划和 block 解析测试。

## 错误场景

- 未初始化工作区时，提示先运行 `llstk init`。
- 根清单非法时失败。
- 应用清单非法时失败并显示路径。
- `hosts.ip` 为空时失败。
- `/etc/hosts` 读取失败时，`hosts status` 应失败并显示路径。
- `--format` 非法时由 clap 处理。

## 测试要求

至少覆盖：

- 创建 gitea 后 `hosts plan` 输出 `127.0.0.1 gitea.locallab`。
- `hosts plan --format block` 输出 begin/end marker。
- 无公开应用时输出 `no public app domains found`。
- 多应用时输出按域名排序。
- 重复域名去重。
- hosts block 解析能识别 present、missing、out of date。
- `hosts.enabled: false` 时不输出记录。

## 验收标准

本阶段完成时，以下命令必须形成 MVP 闭环：

```bash
cargo run --bin llstk -- init
cargo run --bin llstk -- app create gitea --template gitea-postgres
cargo run --bin llstk -- gateway render
cargo run --bin llstk -- hosts plan
cargo run --bin llstk -- hosts plan --format block
```

并且 `hosts plan` 不会修改任何系统文件。

## 本阶段不做

- 不写入 `/etc/hosts`。
- 不请求 sudo。
- 不修改 DNS resolver 配置。
- 不处理通配域名解析。
- 不校验域名是否已经可访问。

## 交付物

- hosts 计划生成逻辑。
- 受管理区块渲染和解析逻辑。
- `hosts plan` 和 `hosts status`。
- 覆盖 hosts plan/status 的测试。
