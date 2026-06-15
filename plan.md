# LocalLabStack 规划

## 1. 项目定位

LocalLabStack 是一个围绕 Docker Compose 构建的本地开发栈管理器。
它提供一种可重复的方式，用于组织本地应用、共享网关路由、
HTTPS 证书、Docker 网络以及主机名解析。

CLI 命令为：

```bash
llstk
```

默认工作区根目录为：

```text
./.locallab
```

核心思路很简单：

- 架构级基础设施放在 `./.locallab` 中。
- 每个应用放在 `./.locallab/app.<app_name>` 中。
- 每个应用拥有自己的数据、配置、docker compose 文件，以及可选的私有网络。
- 共享网关通过 HTTPS 暴露应用域名。
- 共享的全局 Docker 网络连接网关和公开应用服务。
- 应用清单描述意图；生成文件可以由清单渲染得到。

## 2. 目标

### 主要目标

- 初始化一致的本地 Docker Compose 栈目录结构。
- 基于可复用模板创建应用目录。
- 管理用于本地 HTTPS 路由的共享网关。
- 管理用于跨 compose 通信的共享 Docker 网络。
- 为每个应用提供清单格式。
- 支持本地 CA 和 HTTPS 证书生成。
- 支持用于本地域名解析的 hosts 规划与同步。
- 保持系统易于理解，并且可由人工编辑。

### 后续目标

- 增加 Web 管理控制台。
- 展示应用状态、路由、端口和健康检查。
- 提供模板市场或模板注册表支持。
- 支持从既有 Docker Compose 文件导入应用。
- 支持多个本地栈根目录。
- 在可行的情况下支持 Podman 等替代运行时。

## 3. 非目标

- 替代 Docker Compose。
- 成为生产级编排器。
- 将所有配置都隐藏在生成文件之后。
- 管理远程服务器。
- 充当 Kubernetes 抽象层。
- 成为完整的密钥管理系统。

LocalLabStack 应该保持小型、可检查，并且以本地优先。

## 4. 目录布局

推荐的根目录结构：

```text
.locallab/
  docker-compose.yml
  .env
  llstk.yml
  README.md

  certs/
    ca/
    issued/

  gateway/
    Caddyfile
    data/
    config/

  templates/

  app.gitea/
    docker-compose.yml
    .env
    llstk.yml
    data/
      gitea/
      postgres/
    config/

  app.example/
    docker-compose.yml
    .env
    llstk.yml
    data/
    config/
```

根级 `llstk.yml` 描述栈级默认配置。
每个应用级 `llstk.yml` 描述一个本地应用。

## 5. 命名约定

```text
Project name:        LocalLabStack
CLI command:         llstk
Root directory:      ./.locallab
Root manifest:       ./.locallab/llstk.yml
App directory:       ./.locallab/app.<app_name>
App manifest:        ./.locallab/app.<app_name>/llstk.yml
Global network:      locallabstack-global
Private network:     llstk-<app_name>-private
Gateway container:   locallabstack-gateway
App domain:          <app_name>.locallab
```

容器名称应保持稳定且易读：

```text
<app_name>
<app_name>-db
<app_name>-redis
<app_name>-worker
```

示例：

```text
gitea
gitea-db
gitea-runner
```

## 6. 网络模型

LocalLabStack 使用两种网络作用域。

### 全局网络

```text
locallabstack-global
```

用于：

- 网关。
- 需要接收网关流量的应用服务。
- 有意允许跨应用访问的应用服务。

### 应用私有网络

```text
llstk-<app_name>-private
```

用于：

- 内部数据库。
- 内部缓存。
- 内部 worker。
- 不需要跨应用访问的服务。

### 访问规则

- 网关通过全局网络将流量路由到公开应用服务。
- 应用内部组件通过应用私有网络通信。
- 跨 compose 访问使用稳定的容器名称或网络别名。
- 内部服务除非必要，不应加入全局网络。

## 7. 网关

第一版网关实现应使用 Caddy。

原因：

- 配置面较小。
- HTTPS 支持良好。
- 反向代理语法简单。
- 本地开发流程容易。

架构级 compose 示例：

```yaml
services:
  gateway:
    image: caddy:2-alpine
    container_name: locallabstack-gateway
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./gateway/Caddyfile:/etc/caddy/Caddyfile:ro
      - ./gateway/data:/data
      - ./gateway/config:/config
      - ./certs:/certs:ro
    networks:
      - locallabstack-global

networks:
  locallabstack-global:
    name: locallabstack-global
    driver: bridge
```

生成的 Caddyfile 示例：

```caddyfile
{
  auto_https off
}

gitea.locallab {
  tls /certs/issued/locallab.pem /certs/issued/locallab-key.pem
  reverse_proxy gitea:3000
}

*.locallab {
  tls /certs/issued/locallab.pem /certs/issued/locallab-key.pem
  respond "Unknown LocalLabStack app" 404
}
```

## 8. HTTPS 与本地 CA

推荐的第一版实现应集成 `mkcert`。

命令：

```bash
llstk cert init
llstk cert issue
llstk cert status
```

生成文件：

```text
.locallab/certs/
  ca/
  issued/
    locallab.pem
    locallab-key.pem
```

默认证书域名：

```text
locallab
<app_name>.locallab
```

CLI 应支持两种模式：

- `mkcert` 模式，用于实用的本地信任。
- 后续按需提供手动 CA 模式。

私钥不得提交到版本库。

## 9. Hosts 管理

Hosts 管理应显式且保守。

命令：

```bash
llstk hosts plan
llstk hosts sync
llstk hosts status
```

`hosts plan` 展示预期变更，但不写入。
`hosts sync` 向 `/etc/hosts` 写入一个受管理的区块。

受管理区块：

```text
# BEGIN LocalLabStack
127.0.0.1 gitea.locallab
127.0.0.1 minio.locallab
127.0.0.1 api.demo.locallab
# END LocalLabStack
```

该工具绝不应重写无关的 hosts 条目。

## 10. 清单设计

### 根清单

`.locallab/llstk.yml`

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

### 应用清单

`.locallab/app.gitea/llstk.yml`

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

应用清单应成为以下内容的稳定事实来源：

- 域名。
- 网关 upstream。
- 公开/私有服务。
- 主机端口。
- 数据目录。
- 模板元数据。

## 11. CLI 设计

### 栈命令

```bash
llstk init
llstk status
llstk doctor
```

### 应用命令

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

### 网关命令

```bash
llstk gateway render
llstk gateway reload
llstk gateway status
```

### 证书命令

```bash
llstk cert init
llstk cert issue
llstk cert status
```

### Hosts 命令

```bash
llstk hosts plan
llstk hosts sync
llstk hosts status
```

### 模板命令

```bash
llstk template list
llstk template show <name>
```

## 12. 模板系统

初始内置模板：

```text
basic-http
gitea-postgres
postgres
mysql
redis
minio
```

每个模板应是一个目录，并包含：

```text
template.yml
docker-compose.yml.tmpl
llstk.yml.tmpl
.env.tmpl
README.md.tmpl
```

模板变量：

```text
app_name
domain
global_network
private_network
container_prefix
data_dir
```

模板系统一开始应保持简单。
在出现真实用例之前，避免将其做成完整的插件系统。

## 13. Web 控制台方向

Web 控制台应在 CLI 核心稳定之后再添加。

可能的名称：

```text
LocalLabStack Console
```

初始视图：

- 应用列表。
- 应用详情。
- 网关路由。
- 网络概览。
- 证书状态。
- Hosts 状态。
- 模板创建流程。
- Compose 预览。

Web 控制台应调用与 CLI 相同的核心逻辑。
它不应拥有单独的配置模型。

## 14. 实现方向

推荐实现语言：Rust。

原因：

- 单二进制分发。
- 非常适合本地基础设施工具。
- 便于执行 Docker Compose 和 mkcert 等进程。
- YAML 和模板支持良好。
- 后续可以嵌入 Web 控制台。

可能的包结构：

```text
cmd/llstk/
internal/config/
internal/app/
internal/compose/
internal/gateway/
internal/cert/
internal/hosts/
internal/template/
internal/docker/
web/
templates/
```

替代方案：如果快速迭代和 Web UI 共享变得更重要，可以使用 TypeScript。

## 15. MVP

第一个里程碑应刻意保持小范围。

### MVP 命令

```bash
llstk init
llstk app create gitea --template gitea-postgres
llstk gateway render
llstk hosts plan
```

### MVP 输出

`llstk init` 创建：

```text
.locallab/
  docker-compose.yml
  llstk.yml
  gateway/Caddyfile
  certs/
```

`llstk app create gitea --template gitea-postgres` 创建：

```text
.locallab/app.gitea/
  docker-compose.yml
  .env
  llstk.yml
  data/
  config/
```

`llstk gateway render` 重新生成：

```text
.locallab/gateway/Caddyfile
```

`llstk hosts plan` 打印：

```text
127.0.0.1 gitea.locallab
```

## 16. 当前 Compose 的迁移计划

当前根级 compose 包含：

- `gitea`
- `gitea-db`
- 私有网络 `gitea-net`
- 数据目录 `./gitea` 和 `./postgres`

目标迁移：

```text
.locallab/app.gitea/
  docker-compose.yml
  .env
  llstk.yml
  data/
    gitea/
    postgres/
```

变更：

- 将 Gitea 应用数据移动到 `app.gitea/data/gitea` 下。
- 将 Postgres 数据移动到 `app.gitea/data/postgres` 下。
- 移除主机端口 `3000:3000`；由网关处理 HTTPS。
- 保留 SSH 端口 `2222:22`。
- 将 `gitea` 同时接入全局网络和私有网络。
- 默认保持 `gitea-db` 为私有。
- 使用 `gitea-db:5432` 作为数据库主机。
- 使用 `.env` 存放密码，而不是在 compose 中内联密钥。

## 17. 开放问题

- 应用目录应该直接放在 `.locallab/` 下，还是放在 `.locallab/apps/` 下？
- `container_name` 是否应强制要求，还是应优先使用网络别名？
- 生成文件应标记后整体覆盖，还是使用受保护区域进行就地编辑？
- `llstk app up` 应直接调用 Docker Compose，还是第一阶段只打印命令？
- 证书生成是否必须依赖 `mkcert`，还是应从第一天起提供手动 OpenSSL 模式？
- Hosts 同步是否应内置，还是委托给未来的 hosts manager？
- Web 控制台应运行在网关栈内，还是作为单独应用运行？

## 18. 设计原则

- 人类可读文件优先。
- 应用清单是事实来源。
- 生成输出应可预测。
- 默认私有，显式公开。
- 优先采用朴素的 Docker Compose 语义。
- 让本地 HTTPS 保持简单。
- 避免向懂 Docker 的用户隐藏 Docker。
- 让第一个应用容易创建，让第十个应用依然有序。
