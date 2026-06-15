# 阶段 0000：Rust CLI 骨架

## 目标

把当前 Rust 空项目推进到一个可扩展、可测试、命令名稳定的 `llstk` CLI 骨架。本阶段不实现真实业务写入，只建立后续阶段都会依赖的命令结构、错误处理、模块边界和测试入口。

## 前置条件

- 当前仓库可以用 `cargo check` 检查。
- 项目根目录存在 `Cargo.toml` 和 `src/main.rs`。
- 开发者已经阅读 [plan.md](plan.md) 中的项目定位、CLI 设计和 MVP 范围。

## 关键决策

- 第一版使用单 crate，不拆 workspace。
- 包名可以继续使用 `locallab-stack`，但二进制命令名必须是 `llstk`。
- CLI 使用 `clap` derive API。
- 命令入口层使用 `anyhow::Result` 简化错误传播。
- YAML 相关模型使用 `serde` 和 `serde_yaml`。
- 路径处理优先使用标准库 `PathBuf`；只有确实需要 UTF-8 路径语义时再引入 `camino`。
- 本阶段只建立模块文件和命令分派，不执行 Docker、mkcert 或 hosts 写入。

## 依赖要求

运行依赖：

```toml
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
anyhow = "1"
```

开发依赖：

```toml
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

如果实际实现时版本需要调整，应在提交说明或阶段记录中说明原因。

## 模块布局

本阶段应至少建立以下文件：

```text
src/
  main.rs
  cli.rs
  commands/
    mod.rs
    app.rs
    cert.rs
    gateway.rs
    hosts.rs
    init.rs
    template.rs
  config.rs
  manifest.rs
  workspace.rs
  template.rs
  gateway.rs
  hosts.rs
  compose.rs
  cert.rs
```

说明：

- `src/commands/*` 负责 CLI 子命令分派。
- `src/manifest.rs` 只放清单数据结构和 YAML 序列化相关逻辑。
- `src/workspace.rs` 负责 `.locallab` 路径发现和目录布局。
- `src/template.rs`、`src/gateway.rs`、`src/hosts.rs` 等业务模块可以先只放类型和占位函数。

## CLI 规格

本阶段必须提供以下命令结构：

```bash
llstk --help
llstk --root <path> --help
llstk init --help
llstk status --help
llstk doctor --help
llstk app --help
llstk gateway --help
llstk cert --help
llstk hosts --help
llstk template --help
```

应用子命令必须先声明出来：

```bash
llstk app create <name> --template <template>
llstk app list
llstk app show <name>
llstk app up <name>
llstk app down <name>
llstk app restart <name>
llstk app logs <name>
llstk app remove <name>
```

网关、证书、hosts、模板子命令必须先声明出来：

```bash
llstk gateway render
llstk gateway reload
llstk gateway status
llstk cert init
llstk cert issue
llstk cert status
llstk hosts plan
llstk hosts sync
llstk hosts status
llstk template list
llstk template show <name>
```

未实现的命令可以返回明确错误：

```text
not implemented yet: <command>
```

但必须通过 clap 参数解析，并且 help 输出完整。

全局参数：

- `--root <path>`：指定 LocalLabStack 工作区目录。阶段 0000 可以只完成参数定义和传递，具体路径语义在阶段 0001 实现。

## 具体任务

- [ ] 在 `Cargo.toml` 中配置 `[[bin]] name = "llstk"`。
- [ ] 添加本阶段要求的运行依赖和开发依赖。
- [ ] 将 `src/main.rs` 改为薄入口，只调用 CLI runner。
- [ ] 建立 `src/cli.rs`，定义顶层 parser 和子命令枚举。
- [ ] 建立 `src/commands/`，把命令分派从 parser 中分离出来。
- [ ] 建立后续阶段需要的业务模块文件。
- [ ] 为 `--help` 和主要子命令 help 添加集成测试。
- [ ] 确保 `cargo fmt`、`cargo check`、`cargo test` 通过。

## 错误场景

- 未知命令应由 clap 返回标准错误和非 0 exit code。
- 缺少必需参数时应由 clap 返回标准错误和非 0 exit code。
- 暂未实现的命令应返回非 0 exit code，并包含 `not implemented yet`。
- help 命令必须返回 0。

## 测试要求

至少添加以下集成测试：

- `llstk --help` 输出包含 `LocalLabStack` 或项目描述。
- `llstk app --help` 输出包含 `create`、`up`、`down`。
- `llstk gateway --help` 输出包含 `render`。
- `llstk app create` 缺少参数时返回失败。
- 一个暂未实现命令返回 `not implemented yet`。

## 验收标准

本阶段完成时，以下命令必须成功：

```bash
cargo fmt --check
cargo check
cargo test
cargo run --bin llstk -- --help
cargo run --bin llstk -- app --help
```

并且仓库中存在清晰的 CLI 模块边界，后续阶段可以直接在对应模块中填充实现。

## 本阶段不做

- 不创建 `.locallab` 目录。
- 不读写 YAML 清单。
- 不渲染模板。
- 不调用 Docker Compose。
- 不调用 mkcert。
- 不读写 `/etc/hosts`。

## 交付物

- 更新后的 `Cargo.toml`。
- 更新后的 `src/main.rs`。
- 新增 CLI、commands 和业务占位模块。
- 覆盖基础 help 行为的测试。
