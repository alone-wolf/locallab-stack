# 阶段 0010：稳定化与发布

## 目标

把 MVP 打磨成可维护、可测试、可发布的本地基础设施工具。完成后，用户应能根据 README 从空目录走完整个 MVP 流程，并且开发者可以依靠测试和发布门禁继续迭代。

## 前置条件

- 阶段 0000 到 0009 已完成，或至少 MVP 阶段 0000 到 0005 已完成并准备发布预览版。
- 所有已实现命令都有基本测试。
- 项目根目录存在阶段文档和 plan。

## 发布范围

第一版稳定化必须覆盖 MVP 命令：

```bash
llstk init
llstk app create <name> --template <template>
llstk app list
llstk app show <name>
llstk template list
llstk template show <name>
llstk gateway render
llstk gateway status
llstk hosts plan
llstk hosts status
llstk status
llstk doctor
```

如果阶段 0006 到 0009 的外部集成尚未全部达到可发布质量，应在 README 中标记为 experimental 或 upcoming，不得把未稳定能力宣传成已完成。

## 测试门禁

必须通过：

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

如果因为环境限制无法运行 clippy，应记录原因，并至少运行 `cargo check` 和 `cargo test`。

## 测试要求

单元测试覆盖：

- manifest 读写与校验。
- workspace 路径模型。
- template 变量渲染。
- gateway route 提取和 Caddyfile 渲染。
- hosts 计划、block 渲染和 block 替换。
- compose 和 cert command builder。

集成测试覆盖：

- 临时目录中运行 `init`。
- 临时目录中运行 `app create`。
- `gateway render` 后文件内容符合预期。
- `hosts plan` 输出预期记录。
- 错误场景：未初始化、模板不存在、应用已存在、非法应用名。

外部依赖测试：

- Docker 和 mkcert 相关真实执行测试不得作为默认必跑测试。
- 可以用 `#[ignore]` 或手动验证文档记录。

## 文档要求

README 必须包含：

- LocalLabStack 是什么。
- 非目标是什么。
- 安装或本地运行方式。
- MVP 快速开始。
- 目录结构说明。
- 根清单和应用清单示例。
- Gitea 示例。
- 网关和 hosts 工作方式。
- 证书和私钥注意事项。
- 已实现命令列表。
- 未实现或实验性命令列表。
- 故障排查。

快速开始必须能从空目录执行：

```bash
llstk init
llstk app create gitea --template gitea-postgres
llstk gateway render
llstk hosts plan
```

## 版本控制要求

`.gitignore` 至少覆盖：

```gitignore
.locallab/
target/
```

如果决定允许提交 `.locallab` 示例文件，则必须至少排除：

```gitignore
.locallab/certs/issued/*.pem
.locallab/**/data/
```

第一版建议直接忽略 `.locallab/`，避免误提交本地数据和私钥。

## 发布配置

`Cargo.toml` 应包含：

- 正确的 package name。
- 正确的 binary name `llstk`。
- license 字段；如果许可证尚未决定，应在 README 中明确记录暂不发布到公共 registry。
- repository 如果尚未确定可以暂缺。

Release profile 可以先保持默认；如果添加配置，必须说明目的。

## 手动验证脚本

在 README 或单独文档中记录以下手动验证步骤：

```bash
tmpdir="$(mktemp -d)"
cd "$tmpdir"
llstk init
llstk app create gitea --template gitea-postgres
llstk gateway render
llstk hosts plan
llstk doctor
```

如果 Docker 可用：

```bash
llstk app up gitea
llstk app logs gitea --tail 50
llstk app down gitea
```

如果 mkcert 可用：

```bash
llstk cert init
llstk cert issue
llstk cert status
```

## 具体任务

- [ ] 补全 README。
- [ ] 梳理命令 help 文案。
- [ ] 确认 `.gitignore` 安全。
- [ ] 补齐关键单元测试。
- [ ] 补齐 MVP 集成测试。
- [ ] 运行格式化、clippy 和测试。
- [ ] 修复所有 warnings 或记录无法修复原因。
- [ ] 标记未完成命令为 experimental、upcoming 或 not implemented。
- [ ] 准备第一个版本标签说明。

## 错误场景

- README 中的命令不能执行时，必须修正文档或实现。
- help 中出现未实现但未标记的命令时，必须补充说明。
- 测试依赖真实 Docker 或 mkcert 导致默认失败时，必须改为 mock、fixture 或 ignored test。
- `.gitignore` 无法保护私钥或数据目录时，必须修正。

## 验收标准

本阶段完成时：

- `cargo fmt --check` 通过。
- `cargo clippy --all-targets -- -D warnings` 通过，或有明确环境原因记录。
- `cargo test` 通过。
- README 快速开始命令在临时目录中可以执行。
- `.gitignore` 不会让 `.locallab` 本地数据和私钥进入版本库。
- 已实现命令和未实现命令在文档中边界清楚。

## 本阶段不做

- 不要求发布到包管理器。
- 不要求构建安装器。
- 不要求实现 Web 控制台。
- 不要求支持 Windows。
- 不要求支持 Podman。

## 交付物

- 完整 README。
- 测试补齐。
- 通过的质量门禁。
- 安全的 `.gitignore`。
- 第一个可发布版本的说明。
