# LocalLabStack

LocalLabStack is a local-first Docker Compose stack manager. It organizes local applications, a shared Caddy gateway, local HTTPS certificates, Docker networks, and hosts entries under a predictable `.locallab` workspace.

The CLI command is:

```bash
llstk
```

## Non Goals

- It does not replace Docker Compose.
- It is not a production orchestrator.
- It does not manage remote servers.
- It does not hide all generated files from users.
- It is not a Kubernetes abstraction.
- It is not a full secret manager.

## Local Run

From this repository:

```bash
cargo run --bin llstk -- --help
```

After building:

```bash
cargo build
./target/debug/llstk --help
```

## Quick Start

From an empty directory:

```bash
llstk init
llstk app create gitea --template gitea-postgres
llstk gateway render
llstk hosts plan
```

This creates:

```text
.locallab/
  docker-compose.yml
  llstk.yml
  gateway/
    Caddyfile
    data/
    config/
  certs/
    ca/
    issued/
  app.gitea/
    docker-compose.yml
    .env
    llstk.yml
    data/
      gitea/
      postgres/
    config/
```

## Manifests

Root manifest:

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
    - "*.locallab"
hosts:
  enabled: true
  ip: 127.0.0.1
```

App manifest:

```yaml
version: 1
name: gitea
domain: gitea.locallab
upstreams:
  web:
    container: gitea
    port: 3000
    public: true
```

## Gateway And Hosts

`llstk gateway render` scans `.locallab/app.*/llstk.yml` and writes `.locallab/gateway/Caddyfile`.

`llstk hosts plan` prints planned hosts records without writing `/etc/hosts`.

`llstk hosts sync --dry-run` previews the managed block. `llstk hosts sync --yes` writes only the block between:

```text
# BEGIN LocalLabStack
# END LocalLabStack
```

The tool does not call `sudo` for you.

## Certificates

The first certificate provider is `mkcert`.

```bash
llstk cert init
llstk cert issue
llstk cert status
```

Private keys are written under `.locallab/certs/issued/` and must not be committed.

## Implemented Commands

```bash
llstk init
llstk status
llstk doctor
llstk app create <name> --template <template>
llstk app list
llstk app show <name>
llstk app up <name>
llstk app down <name>
llstk app restart <name>
llstk app logs <name>
llstk app remove <name>
llstk app import-compose <path> --name <name>
llstk app migrate-gitea
llstk stack up
llstk stack down
llstk stack restart
llstk stack logs
llstk stack status
llstk gateway up
llstk gateway down
llstk gateway restart
llstk gateway logs
llstk gateway render
llstk gateway reload
llstk gateway status
llstk cert init
llstk cert issue
llstk cert status
llstk hosts plan
llstk hosts status
llstk hosts sync
llstk template list
llstk template show <name>
```

Docker and mkcert commands require those tools to be installed locally.

## Manual Verification

```bash
tmpdir="$(mktemp -d)"
cd "$tmpdir"
llstk init
llstk app create gitea --template gitea-postgres
llstk gateway render
llstk hosts plan
llstk doctor
```

If Docker is available:

```bash
llstk stack up --render
llstk stack logs --tail 50
llstk stack down
```

For gateway-only debugging:

```bash
llstk gateway render
llstk gateway up
llstk gateway logs --tail 50
llstk gateway down
```

For app-only debugging:

```bash
llstk app up gitea
llstk app logs gitea --tail 50
llstk app down gitea
```

If mkcert is available:

```bash
llstk cert init
llstk cert issue
llstk cert status
```

## Troubleshooting

- If `llstk doctor` reports Docker errors, check `docker compose version`.
- If hosts sync fails, rerun with a writable `--hosts-file` for testing or use appropriate permissions for `/etc/hosts`.
- If Caddy cannot find certificates, run `llstk cert issue` or inspect `.locallab/certs/issued/`.
- If an app already exists, inspect `.locallab/app.<name>/` before using `--force`.
