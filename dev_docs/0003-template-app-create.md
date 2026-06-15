# 阶段 0003：模板系统与应用创建

## 目标

实现最小可用模板系统，并通过 `llstk app create` 从模板创建应用目录。完成后，用户应能从一个已初始化工作区创建 `gitea` 应用，并得到可检查、可修改的 compose、env 和应用清单文件。

## 前置条件

- 阶段 0000、0001、0002 已完成。
- `.locallab/llstk.yml` 可以被读取和校验。
- `WorkspaceLayout` 可以定位应用目录和模板目录。

## 命令规格

```bash
llstk app create <name> --template <template> [--domain <domain>] [--force]
llstk app list
llstk app show <name>
llstk template list
llstk template show <name>
```

参数：

- `<name>`：应用名，必须通过阶段 0001 的应用名校验。
- `--template <template>`：模板名，第一版必须支持 `basic-http` 和 `gitea-postgres`。
- `--domain <domain>`：覆盖默认域名，默认 `<name>.locallab`。
- `--force`：允许覆盖本工具生成的应用文件；不得删除 `data/`。

## 模板范围

第一版必须内置两个模板：

```text
basic-http
gitea-postgres
```

以下模板只作为后续扩展，可以先不实现：

```text
postgres
mysql
redis
minio
```

## 模板文件模型

每个模板逻辑上应包含：

```text
template.yml
docker-compose.yml.tmpl
llstk.yml.tmpl
.env.tmpl
README.md.tmpl
```

内置模板可以通过 Rust 字符串常量、`include_str!` 或 `templates/` 目录实现。无论采用哪种方式，都必须保留清晰的模板边界，方便后续迁移到文件模板。

## 模板变量

必须支持：

```text
app_name
domain
global_network
private_network
container_prefix
data_dir
```

变量默认值：

```text
domain           <app_name>.locallab
global_network   根清单 network.global
private_network  llstk-<app_name>-private
container_prefix <app_name>
data_dir         ./data
```

第一版模板渲染可以使用简单安全的变量替换，但必须做到：

- 未知变量报错。
- 缺失变量报错。
- 不静默留下 `{{ variable }}`。

## 应用目录输出

运行：

```bash
llstk app create gitea --template gitea-postgres
```

应创建：

```text
.locallab/lab-app-gitea/
  docker-compose.yml
  .env
  llstk.yml
  README.md
  data/
    gitea/
    postgres/
  config/
```

运行：

```bash
llstk app create demo --template basic-http
```

应创建：

```text
.locallab/lab-app-demo/
  docker-compose.yml
  .env
  llstk.yml
  README.md
  data/
  config/
```

## `gitea-postgres` 模板要求

应用清单必须表达：

- `name: gitea`。
- `domain: gitea.locallab`。
- 公开 upstream 指向 `gitea:3000`。
- SSH 端口映射 `2222:22`。
- `gitea` 服务同时在 `global` 和 `private` 网络。
- `gitea-db` 服务只在 `private` 网络。
- data 包含 `./data/gitea` 和 `./data/postgres`。

Compose 文件必须表达：

- `gitea` service。
- `gitea-db` service。
- `gitea` 加入外部全局网络和私有网络。
- `gitea-db` 只加入私有网络。
- 数据目录挂载到 `./data/gitea` 和 `./data/postgres`。
- 密码等敏感配置通过 `.env` 引用。
- 不暴露 `3000:3000`。
- 保留 SSH `2222:22`。

## `basic-http` 模板要求

应用清单必须表达：

- 一个公开 web upstream。
- 默认域名 `<app_name>.locallab`。
- 一个公开服务加入全局网络。

Compose 文件可以使用稳定的小型 HTTP 镜像，例如 `nginx:alpine` 或 `caddy:2-alpine`。如果选择镜像，应在模板 README 中说明用途。

## 写入策略

- 应用目录不存在时创建。
- 应用目录存在且没有 `--force` 时失败，不做部分覆盖。
- 应用目录存在且传入 `--force` 时可以覆盖 `docker-compose.yml`、`.env`、`llstk.yml`、`README.md`，但不得删除 `data/` 和 `config/`。
- 如果渲染中途失败，应避免留下半成品；至少要保证不会破坏已有应用目录。

## 具体任务

- [ ] 实现模板注册表，至少包含 `basic-http` 和 `gitea-postgres`。
- [ ] 实现模板变量上下文。
- [ ] 实现模板渲染和未知变量检测。
- [ ] 实现 `llstk template list`。
- [ ] 实现 `llstk template show <name>`。
- [ ] 实现 `llstk app create <name> --template <template>`。
- [ ] 实现 `llstk app list`，扫描 `.locallab/lab-app-*/llstk.yml`。
- [ ] 实现 `llstk app show <name>`，输出应用清单摘要。
- [ ] 为模板渲染和 app create 添加测试。

## 错误场景

- 未初始化工作区时，`app create` 应失败并提示先运行 `llstk init`。
- 应用名非法时，应复用阶段 0001 的校验错误。
- 模板不存在时，应列出可用模板。
- 应用目录已存在且没有 `--force` 时，应失败。
- 模板渲染后 YAML 无法反序列化为 `AppManifest` 时，应失败。
- 渲染结果包含未替换变量时，应失败。

## 输出要求

成功创建应用时输出：

```text
created .locallab/lab-app-gitea/llstk.yml
created .locallab/lab-app-gitea/docker-compose.yml
created .locallab/lab-app-gitea/.env
created .locallab/lab-app-gitea/data/gitea/
created .locallab/lab-app-gitea/data/postgres/
```

`template list` 输出至少包含：

```text
basic-http
gitea-postgres
```

`app list` 输出至少包含应用名和域名：

```text
gitea  gitea.locallab
```

## 测试要求

至少覆盖：

- `template list` 包含两个必需模板。
- `template show gitea-postgres` 成功。
- 未初始化工作区时 `app create` 失败。
- 初始化后创建 `gitea` 成功。
- 创建后的应用清单可以被 `AppManifest` 读回并通过校验。
- Gitea compose 不包含 `3000:3000`。
- Gitea compose 包含 `2222:22`。
- 应用目录已存在时不带 `--force` 会失败。
- `app list` 能列出创建的应用。
- `app show gitea` 能输出域名和 upstream 信息。

## 验收标准

本阶段完成时，以下命令必须形成闭环：

```bash
cargo run --bin llstk -- init
cargo run --bin llstk -- template list
cargo run --bin llstk -- app create gitea --template gitea-postgres
cargo run --bin llstk -- app list
cargo run --bin llstk -- app show gitea
```

并且 `.locallab/lab-app-gitea/llstk.yml` 能被程序读回和校验。

## 本阶段不做

- 不启动 Docker Compose。
- 不验证镜像是否能拉取。
- 不生成 Caddyfile 应用路由。
- 不写 hosts。
- 不生成真实随机密码；第一版可以生成明确占位值，但 README 必须提醒用户修改。

## 交付物

- 内置模板注册表。
- 模板渲染逻辑。
- `app create`、`app list`、`app show`。
- `template list`、`template show`。
- 覆盖模板和应用创建的测试。
