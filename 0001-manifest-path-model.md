# 阶段 0001：清单与路径模型

## 目标

定义 LocalLabStack 的核心数据模型和路径模型，让根清单、应用清单和 `.locallab` 目录布局成为后续命令共享的稳定事实来源。

## 前置条件

- 阶段 0000 已完成。
- `llstk` CLI 骨架和模块边界已经存在。
- 项目可以通过 `cargo check` 和 `cargo test`。

## 关键决策

- 根工作区默认目录为当前目录下的 `.locallab`。
- 第一版允许通过全局参数 `--root <path>` 覆盖工作区目录；如果阶段 0000 没有实现该参数，本阶段应补上。
- `--root` 的相对路径基于当前工作目录解析。
- 应用目录直接位于 `.locallab/app.<app_name>`，暂不使用 `.locallab/apps/`。
- `llstk.yml` 是事实来源；生成文件可以从清单重新生成。
- YAML 字段使用 snake_case。
- 清单版本字段 `version` 必须存在，第一版只接受 `1`。

## 数据模型

根清单：

```yaml
version: 1
name: default
root: ./.locallab
network:
  global: locallabstack-global
gateway:
  provider: caddy
  container: locallabstack-gateway
  http_port: 80
  https_port: 443
cert:
  provider: mkcert
  domains:
    - locallab
hosts:
  enabled: true
  ip: 127.0.0.1
```

应用清单：

```yaml
version: 1
name: gitea
domain: gitea.locallab
upstreams:
  web:
    container: gitea
    port: 3000
    public: true
ports:
  ssh:
    host: 2222
    container: 22
services:
  gitea:
    public: true
    networks:
      - global
      - private
  gitea-db:
    public: false
    networks:
      - private
data:
  - ./data/gitea
  - ./data/postgres
```

## 路径模型

必须由 `workspace` 模块集中提供以下路径：

```text
root_dir                  .locallab
root_manifest_path        .locallab/llstk.yml
root_compose_path         .locallab/docker-compose.yml
gateway_dir               .locallab/gateway
gateway_caddyfile_path    .locallab/gateway/Caddyfile
certs_dir                 .locallab/certs
templates_dir             .locallab/templates
app_dir(name)             .locallab/app.<name>
app_manifest_path(name)   .locallab/app.<name>/llstk.yml
app_compose_path(name)    .locallab/app.<name>/docker-compose.yml
```

## 校验规则

应用名：

- 只允许小写字母、数字和单个连字符分隔。
- 不允许空字符串。
- 不允许以连字符开头或结尾。
- 不允许包含点号、斜杠、空格或下划线。

域名：

- 默认域名为 `<app_name>.locallab`。
- 必须是非空字符串。
- 第一版只要求不包含空格和斜杠。

端口：

- `host` 和 `container` 必须在 `1..=65535`。
- upstream `port` 必须在 `1..=65535`。

网络：

- 服务网络只允许 `global` 和 `private`。
- 公开 upstream 对应的服务应包含 `global` 网络；如果不满足，校验应报错或 warning。本阶段建议先报错。

## 具体任务

- [ ] 定义 `RootManifest`、`AppManifest` 及其嵌套结构体。
- [ ] 实现根清单默认值构造函数。
- [ ] 实现应用清单默认构造函数，输入 `app_name` 后生成默认域名。
- [ ] 实现 YAML 读写函数。
- [ ] 实现清单校验函数。
- [ ] 实现 `WorkspaceLayout` 或等价类型，集中提供路径。
- [ ] 为 `--root` 增加 CLI 支持并传入命令上下文。
- [ ] 添加 manifest 和 workspace 的单元测试。

## 错误场景

- 清单文件不存在时，读取函数应返回包含路径的错误。
- YAML 语法错误时，错误信息应包含文件路径。
- 版本不是 `1` 时，应明确提示 unsupported manifest version。
- 应用名非法时，应明确提示合法格式。
- 端口越界时，应指出字段名和非法值。

## 测试要求

至少覆盖：

- 根清单默认值序列化后字段完整。
- 根清单 YAML round-trip 后保持等价。
- 应用清单 YAML round-trip 后保持等价。
- 合法应用名通过校验。
- 非法应用名被拒绝。
- 公开 upstream 但服务没有 global 网络时校验失败。
- `--root custom-root` 生成的 workspace 路径正确。

## 验收标准

本阶段完成时：

- `cargo test` 通过。
- 清单模型可以直接用于阶段 0002 的 `llstk init`。
- workspace 路径模型可以直接用于阶段 0003、0004、0005。
- 所有清单读取错误都包含足够定位问题的路径信息。

## 本阶段不做

- 不创建实际目录结构。
- 不生成 Docker Compose 文件。
- 不渲染 Caddyfile。
- 不扫描已有应用目录。
- 不调用任何外部命令。

## 交付物

- `manifest` 模块中的根清单和应用清单模型。
- `workspace` 模块中的路径布局类型。
- 针对清单和路径的单元测试。
