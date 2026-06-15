use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::cli::{Context as CliContext, InitArgs};
use crate::config::GENERATED_HEADER;
use crate::manifest::RootManifest;

pub fn run(context: &CliContext, args: InitArgs) -> Result<()> {
    if args.name.trim().is_empty() {
        bail!("--name cannot be empty");
    }
    let layout = &context.layout;
    create_dir(layout.root_dir())?;
    create_dir(&layout.gateway_dir())?;
    create_dir(&layout.gateway_data_dir())?;
    create_dir(&layout.gateway_config_dir())?;
    create_dir(&layout.certs_dir())?;
    create_dir(&layout.certs_ca_dir())?;
    create_dir(&layout.certs_issued_dir())?;
    create_dir(&layout.templates_dir())?;

    let manifest = RootManifest::default_named(args.name);
    write_manifest(&layout.root_manifest_path(), &manifest, args.force)?;
    write_generated(&layout.root_compose_path(), &root_compose(), args.force)?;
    write_generated(
        &layout.gateway_caddyfile_path(),
        &default_caddyfile(),
        args.force,
    )?;
    write_generated(&layout.readme_path(), README, args.force)?;
    Ok(())
}

fn create_dir(path: &Path) -> Result<()> {
    if path.exists() && !path.is_dir() {
        bail!("{} exists but is not a directory", path.display());
    }
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))?;
    println!("created {}", path.display());
    Ok(())
}

fn write_manifest(path: &Path, manifest: &RootManifest, force: bool) -> Result<()> {
    if path.exists() && !force {
        println!("exists, skipped {}", path.display());
        return Ok(());
    }
    manifest.write_to(path)?;
    println!("created {}", path.display());
    Ok(())
}

fn write_generated(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        println!("exists, skipped {}", path.display());
        return Ok(());
    }
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;
    println!("created {}", path.display());
    Ok(())
}

fn root_compose() -> String {
    r#"services:
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
"#
    .to_string()
}

fn default_caddyfile() -> String {
    format!(
        "# {GENERATED_HEADER}\n\n{{\n  auto_https off\n}}\n\n*.locallab {{\n  tls /certs/issued/locallab.pem /certs/issued/locallab-key.pem\n  respond \"Unknown LocalLabStack app\" 404\n}}\n"
    )
}

const README: &str = r#"# LocalLabStack Workspace

This directory is managed by LocalLabStack.

Files are intentionally readable and can be inspected by humans. Private keys and local data should not be committed to version control.

Next step:

```bash
llstk app create gitea --template gitea-postgres
```
"#;
