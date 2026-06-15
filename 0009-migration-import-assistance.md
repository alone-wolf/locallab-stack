# 阶段 0009：迁移与导入辅助

## 目标

为现有 Docker Compose 应用迁移到 LocalLabStack 提供安全、可回退的辅助能力。第一版重点服务当前 Gitea + Postgres 迁移场景，先生成迁移计划和目标文件，不自动移动用户数据。

## 前置条件

- 阶段 0000 到 0008 已完成。
- `gitea-postgres` 模板稳定。
- `app create`、`gateway render`、`hosts plan` 已可形成完整配置闭环。

## 命令规格

```bash
llstk app import-compose <path> --name <name> [--dry-run]
llstk app migrate-gitea [--source <path>] [--dry-run]
```

说明：

- `app import-compose` 是通用方向，第一版可以只做分析和建议，不要求完整自动转换。
- `app migrate-gitea` 是针对当前项目的明确迁移辅助，可以优先实现。
- 本阶段的最低交付是 dry-run 计划命令和迁移文档；不得只留下口头说明或未接入 CLI 的散文文档。

## Gitea 目标结构

```text
.locallab/app.gitea/
  docker-compose.yml
  .env
  llstk.yml
  data/
    gitea/
    postgres/
  config/
```

## 迁移规则

现有根级 compose 如果包含：

- `gitea`
- `gitea-db`
- 私有网络 `gitea-net`
- 数据目录 `./gitea`
- 数据目录 `./postgres`

迁移目标应满足：

- Gitea 数据位于 `.locallab/app.gitea/data/gitea`。
- Postgres 数据位于 `.locallab/app.gitea/data/postgres`。
- 移除主机端口 `3000:3000`。
- 保留 SSH 端口 `2222:22`。
- `gitea` 同时接入全局网络和私有网络。
- `gitea-db` 只接入私有网络。
- 数据库 host 使用 `gitea-db:5432`。
- 密码等敏感值放入 `.env`。

## 安全策略

- 第一版不得自动移动或删除数据目录。
- 涉及数据移动时只输出建议命令。
- 输出建议命令前必须提醒用户先停止旧 compose 并备份数据。
- 不自动修改原始 compose 文件。
- 不自动删除旧容器、旧网络或旧 volume。

## `app migrate-gitea --dry-run`

至少输出：

- 检测到的旧 compose 路径。
- 目标应用目录。
- 将生成或建议生成的文件。
- 需要用户手动执行的数据迁移命令。
- 迁移后的验证命令。

示例输出片段：

```text
source compose: docker-compose.yml
target app: .locallab/app.gitea

manual data moves:
  mv ./gitea .locallab/app.gitea/data/gitea
  mv ./postgres .locallab/app.gitea/data/postgres

verify:
  llstk gateway render
  llstk hosts plan
  llstk app up gitea
```

## `app import-compose`

第一版通用导入可以只识别：

- services 名称。
- ports。
- volumes。
- networks。

输出分析报告，不生成应用，除非实现者选择添加显式 `--write`。如果添加 `--write`，必须先支持 `--dry-run`。

## 具体任务

- [ ] 添加迁移文档或命令说明。
- [ ] 实现 compose YAML 的只读解析或文本级摘要。
- [ ] 实现 `app migrate-gitea --dry-run`。
- [ ] 可选实现 `app migrate-gitea` 生成目标模板文件，但不移动数据。
- [ ] 实现 `app import-compose <path> --name <name> --dry-run` 分析报告。
- [ ] 添加 fixture compose 文件测试。

## 错误场景

- source compose 不存在时失败并显示路径。
- source compose YAML 无法解析时失败。
- 目标应用已存在时，不得覆盖，除非未来提供显式 `--force`。
- 检测不到 gitea/gitea-db 时，`migrate-gitea` 应提示不匹配。
- 发现 bind mount 路径不明确时，应输出人工处理提示。

## 测试要求

至少覆盖：

- fixture 中的 gitea compose 能被识别。
- dry-run 输出目标目录和数据移动建议。
- dry-run 不创建应用目录。
- source compose 缺失时报错。
- 非 gitea compose 执行 `migrate-gitea` 时失败并解释原因。
- import-compose 能列出 services、ports、volumes、networks。

## 验收标准

本阶段完成时，用户可以从当前 Gitea compose 得到一份清楚的迁移计划，并能按计划人工完成：

```bash
llstk init
llstk app create gitea --template gitea-postgres
llstk gateway render
llstk hosts plan
llstk app up gitea
```

迁移辅助不得破坏旧 compose、旧数据目录或旧容器。

## 本阶段不做

- 不自动移动生产或本地数据。
- 不删除旧 compose。
- 不删除旧容器。
- 不尝试自动理解所有 Compose 特性。
- 不支持 Kubernetes 或远程服务器迁移。

## 交付物

- Gitea 迁移辅助命令或完整迁移文档。
- Compose 分析报告能力。
- fixture 和迁移测试。
