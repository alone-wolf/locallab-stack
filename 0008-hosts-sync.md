# 阶段 0008：Hosts 同步

## 目标

在阶段 0005 的只读规划基础上，实现保守、可预览、只维护受管理区块的 `/etc/hosts` 同步能力。

## 前置条件

- 阶段 0000 到 0007 已完成。
- `hosts plan` 和受管理区块渲染函数已经稳定。
- `hosts status` 可以识别 managed block 的 present、missing、out of date 状态。

## 命令规格

```bash
llstk hosts sync [--dry-run] [--yes] [--hosts-file <path>]
```

参数：

- `--dry-run`：只显示将要写入的结果，不写文件。
- `--yes`：跳过交互确认。
- `--hosts-file <path>`：用于测试或高级场景，默认 `/etc/hosts`。

第一版在非交互环境中如果没有 `--yes`，应失败并提示添加 `--yes` 或使用 `--dry-run`。

## 同步规则

- 只新增或替换 `# BEGIN LocalLabStack` 到 `# END LocalLabStack` 区块。
- 无受管理区块时，将区块追加到文件末尾。
- 已有一个受管理区块时，替换该区块。
- 已有多个受管理区块时，失败并要求用户手动清理。
- 保留区块外所有内容、顺序和换行风格。
- 不对区块外 hosts 记录做去重或修改。

## 受管理区块格式

必须复用阶段 0005 的生成函数：

```text
# BEGIN LocalLabStack
127.0.0.1 gitea.locallab
# END LocalLabStack
```

如果没有计划记录，可以写入空区块，也可以删除区块。第一版建议写入空区块，行为更可预测：

```text
# BEGIN LocalLabStack
# END LocalLabStack
```

## 输出要求

`--dry-run` 输出：

```text
would update /etc/hosts
# BEGIN LocalLabStack
127.0.0.1 gitea.locallab
# END LocalLabStack
```

实际写入前，如果没有 `--yes`，交互确认：

```text
Update /etc/hosts managed block? [y/N]
```

写入成功：

```text
updated /etc/hosts
```

## 权限策略

- 程序不自行提权。
- 写入 `/etc/hosts` 权限不足时，显示清晰错误。
- 错误信息应建议用户用合适权限重新运行命令，而不是自动调用 sudo。
- 测试必须通过 `--hosts-file <tempfile>` 完成，不触碰真实 `/etc/hosts`。

## 原子写入策略

对可写 hosts 文件：

- 先读取完整文件。
- 在内存中生成新内容。
- 写入同目录临时文件。
- 尽量保留原文件权限。
- rename 替换目标文件。

如果目标是 `/etc/hosts` 且同目录临时文件无法创建，应返回错误，不退化为危险的部分写入。

## 具体任务

- [ ] 实现 managed block 替换函数。
- [ ] 实现 multiple block 检测。
- [ ] 实现 `hosts sync --dry-run`。
- [ ] 实现 `hosts sync --yes`。
- [ ] 实现 `--hosts-file <path>`。
- [ ] 实现原子写入辅助函数。
- [ ] 添加完整 hosts 内容替换测试。
- [ ] 添加 CLI 集成测试，全部使用临时 hosts 文件。

## 错误场景

- hosts 管理被禁用时，应不写入并提示。
- hosts 文件不存在时，应失败，除非未来明确支持创建。
- hosts 文件不可读时失败。
- hosts 文件不可写时失败。
- 检测到多个 LocalLabStack 区块时失败。
- 非交互环境未传 `--yes` 且非 `--dry-run` 时失败。
- 写入临时文件或 rename 失败时，应保留原文件不变。

## 测试要求

至少覆盖：

- 无 managed block 时追加区块。
- 已有 managed block 时替换区块。
- 区块外内容完全保留。
- 多个 managed block 时失败。
- `--dry-run` 不修改文件。
- `--hosts-file <tempfile>` 生效。
- hosts disabled 时不写入。
- 写入后再次 `hosts status` 显示 up to date。

## 验收标准

本阶段完成时，使用临时 hosts 文件可以运行：

```bash
cargo run --bin llstk -- hosts sync --dry-run --hosts-file /tmp/llstk-hosts
cargo run --bin llstk -- hosts sync --yes --hosts-file /tmp/llstk-hosts
cargo run --bin llstk -- hosts status --hosts-file /tmp/llstk-hosts
```

真实 `/etc/hosts` 写入需要用户显式选择合适权限执行，程序不得自动提权。

## 本阶段不做

- 不自动调用 sudo。
- 不修改区块外 hosts 内容。
- 不管理 DNS resolver。
- 不支持 wildcard hosts。
- 不做图形化确认。

## 交付物

- hosts managed block 替换逻辑。
- `hosts sync`。
- 安全写入逻辑。
- 覆盖临时 hosts 文件的测试。
