# 阶段 0006：Docker Compose 集成

## 目标

让 LocalLabStack 能够以可预测、可审计的方式调用 Docker Compose 管理 gateway 和应用生命周期。完成后，用户可以用 `llstk` 启停应用，而不需要手动进入各应用目录执行 compose 命令。

## 前置条件

- 阶段 0000 到 0005 已完成。
- `llstk init` 能生成根级 gateway compose。
- `llstk app create` 能生成应用 compose。
- 应用清单和路径模型稳定。

## 命令规格

```bash
llstk status
llstk doctor
llstk gateway reload
llstk app up <name> [--detach]
llstk app down <name>
llstk app restart <name>
llstk app logs <name> [--follow] [--tail <n>]
llstk app remove <name> [--dry-run]
```

默认行为：

- `app up` 默认使用 detached 模式；`--detach` 可以保留为显式选项。
- `app logs` 默认不 follow。
- `app remove` 第一版只停止并删除 compose 管理的容器，不删除 `data/`，且必须支持 `--dry-run`。

## 外部命令策略

第一版直接调用：

```bash
docker compose
```

必须通过一个集中封装的 `compose` 模块执行，不允许在各命令中散落 `Command::new("docker")`。

封装层至少支持：

- 设置工作目录。
- 传入 compose 子命令参数。
- 捕获 exit status。
- 将 stdout/stderr 透传或收集。
- 在错误信息中包含执行目录和命令参数。

## Compose 工作目录

命令对应目录：

```text
gateway compose    .locallab/
app compose        .locallab/app.<name>/
```

`llstk app up gitea` 应等价于：

```bash
cd .locallab/app.gitea
docker compose up -d
```

`llstk app down gitea` 应等价于：

```bash
cd .locallab/app.gitea
docker compose down
```

## `doctor` 检查项

`llstk doctor` 不应修改系统状态。

至少检查：

- `docker` 可执行文件是否存在。
- `docker compose version` 是否成功。
- `.locallab/llstk.yml` 是否存在且合法。
- `.locallab/docker-compose.yml` 是否存在。
- `.locallab/gateway/Caddyfile` 是否存在。
- 全局网络名称是否能从根清单读取。
- 已创建应用的 `docker-compose.yml` 是否存在。
- 已创建应用的 `llstk.yml` 是否合法。

输出示例：

```text
ok docker
ok docker compose
ok root manifest .locallab/llstk.yml
ok gateway compose .locallab/docker-compose.yml
warn cert file missing .locallab/certs/issued/locallab.pem
```

如果存在 error 项，命令返回非 0。

## `status` 输出

第一版 `status` 可以只输出 LocalLabStack 视角的静态状态和可选 Docker 状态：

- root 路径。
- root manifest 是否存在。
- app 数量。
- public route 数量。
- gateway Caddyfile 是否存在。

如果 Docker 可用，可以补充 `docker compose ps`；如果 Docker 不可用，不能导致整个 `status` 失败，应显示 warning。

## `gateway reload`

第一版实现可以调用：

```bash
docker compose exec gateway caddy reload --config /etc/caddy/Caddyfile
```

要求：

- reload 前检查根 compose 和 Caddyfile 存在。
- reload 失败时透出 Caddy/Docker 错误。
- 不自动执行 `gateway render`，除非未来添加显式 `--render`。

## `app remove`

第一版必须保守：

- 停止并移除 compose 容器。
- 不删除应用目录。
- 不删除 `data/`。
- 不删除 `config/`。
- 输出下一步手动删除路径的提示。

`--dry-run` 应展示会执行的 compose 命令，但不调用 Docker。

## 具体任务

- [ ] 实现 `compose` 外部命令封装。
- [ ] 实现 `doctor`。
- [ ] 实现 `status`。
- [ ] 实现 `app up/down/restart/logs/remove`。
- [ ] 实现 `gateway reload`。
- [ ] 为 compose 封装添加可测试的 command builder。
- [ ] 为不依赖真实 Docker 的逻辑添加测试。
- [ ] 为真实 Docker 场景保留可选 ignored integration test 或文档化手动验证步骤。

## 错误场景

- Docker 不存在时，`doctor` 应报告 error，app lifecycle 命令应失败。
- Docker Compose 不可用时，应明确提示 `docker compose` 检查失败。
- 应用不存在时，应提示可用应用或建议运行 `llstk app list`。
- 应用 compose 文件不存在时，应失败并显示路径。
- 外部命令返回非 0 时，应保留 exit status 和 stderr。
- `app logs --tail` 不是正整数时由 clap 拒绝。

## 测试要求

至少覆盖：

- compose command builder 为 app up 生成正确目录和参数。
- compose command builder 为 app logs follow/tail 生成正确参数。
- 应用不存在时 `app up` 失败。
- `doctor` 在未初始化目录下报告 root manifest 缺失。
- `status` 在 Docker 不可用时不 panic。
- `app remove --dry-run` 不调用 Docker，并输出预期命令。

真实 Docker 行为可以先不纳入默认 `cargo test`，但必须在文档中列出手动验证命令。

## 验收标准

本阶段完成时：

```bash
cargo test
cargo run --bin llstk -- doctor
cargo run --bin llstk -- status
cargo run --bin llstk -- app up gitea
cargo run --bin llstk -- app logs gitea --tail 100
cargo run --bin llstk -- app down gitea
```

在 Docker 可用且已创建 `gitea` 应用时，生命周期命令能执行对应 compose 操作；在 Docker 不可用时，错误信息清楚且不破坏文件。

## 本阶段不做

- 不支持 Podman。
- 不自动拉取或管理 Docker 安装。
- 不删除应用数据目录。
- 不做健康检查轮询。
- 不实现 Web 控制台。

## 交付物

- Docker Compose 命令封装。
- `doctor` 和 `status`。
- 应用生命周期命令。
- `gateway reload`。
- 覆盖命令构造和错误场景的测试。
