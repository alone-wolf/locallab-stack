use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::manifest::{AppManifest, PortMapping, Service, ServiceNetwork, Upstream};

#[derive(Clone, Debug)]
pub struct BuiltinTemplate {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Debug)]
pub struct TemplateContext {
    pub app_name: String,
    pub domain: String,
    pub global_network: String,
    pub private_network: String,
    pub container_prefix: String,
    pub data_dir: String,
}

#[derive(Clone, Debug)]
pub struct RenderedTemplate {
    pub manifest: AppManifest,
    pub compose: String,
    pub env: String,
    pub readme: String,
    pub data_dirs: Vec<String>,
}

pub fn list_templates() -> Vec<BuiltinTemplate> {
    vec![
        BuiltinTemplate {
            name: "basic-http",
            description: "A minimal public HTTP service using nginx.",
        },
        BuiltinTemplate {
            name: "gitea-postgres",
            description: "Gitea with a private PostgreSQL database.",
        },
    ]
}

pub fn get_template(name: &str) -> Option<BuiltinTemplate> {
    list_templates()
        .into_iter()
        .find(|template| template.name == name)
}

pub fn render_template(template: &str, context: &TemplateContext) -> Result<RenderedTemplate> {
    match template {
        "basic-http" => render_basic_http(context),
        "gitea-postgres" => render_gitea_postgres(context),
        _ => bail!(
            "unknown template {template}; available templates: {}",
            list_templates()
                .into_iter()
                .map(|template| template.name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

pub fn render_variables(input: &str, context: &TemplateContext) -> Result<String> {
    let values = BTreeMap::from([
        ("app_name", context.app_name.as_str()),
        ("domain", context.domain.as_str()),
        ("global_network", context.global_network.as_str()),
        ("private_network", context.private_network.as_str()),
        ("container_prefix", context.container_prefix.as_str()),
        ("data_dir", context.data_dir.as_str()),
    ]);

    let mut output = String::new();
    let mut rest = input;
    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("}}") else {
            bail!("unclosed template variable");
        };
        let key = after_start[..end].trim();
        let Some(value) = values.get(key) else {
            bail!("unknown template variable {key}");
        };
        output.push_str(value);
        rest = &after_start[end + 2..];
    }
    output.push_str(rest);
    if output.contains("{{") || output.contains("}}") {
        bail!("template contains unreplaced variables");
    }
    Ok(output)
}

pub fn write_rendered_template(
    app_dir: &Path,
    rendered: &RenderedTemplate,
    force: bool,
) -> Result<()> {
    if app_dir.exists() && !force {
        bail!("app directory already exists: {}", app_dir.display());
    }
    fs::create_dir_all(app_dir.join("config"))
        .with_context(|| format!("failed to create {}", app_dir.join("config").display()))?;
    for data_dir in &rendered.data_dirs {
        fs::create_dir_all(app_dir.join(data_dir.trim_start_matches("./"))).with_context(|| {
            format!(
                "failed to create {}",
                app_dir.join(data_dir.trim_start_matches("./")).display()
            )
        })?;
    }
    rendered.manifest.write_to(&app_dir.join("llstk.yml"))?;
    write_file(&app_dir.join("docker-compose.yml"), &rendered.compose)?;
    write_file(&app_dir.join(".env"), &rendered.env)?;
    write_file(&app_dir.join("README.md"), &rendered.readme)?;
    Ok(())
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))
}

fn render_basic_http(context: &TemplateContext) -> Result<RenderedTemplate> {
    let manifest = AppManifest::default_for(&context.app_name, Some(context.domain.clone()))?;
    let compose = render_variables(BASIC_HTTP_COMPOSE, context)?;
    let env = render_variables(BASIC_HTTP_ENV, context)?;
    let readme = render_variables(BASIC_HTTP_README, context)?;
    Ok(RenderedTemplate {
        manifest,
        compose,
        env,
        readme,
        data_dirs: vec!["data".to_string()],
    })
}

fn render_gitea_postgres(context: &TemplateContext) -> Result<RenderedTemplate> {
    let mut upstreams = BTreeMap::new();
    upstreams.insert(
        "web".to_string(),
        Upstream {
            container: "gitea".to_string(),
            port: 3000,
            public: true,
        },
    );
    let mut ports = BTreeMap::new();
    ports.insert(
        "ssh".to_string(),
        PortMapping {
            host: 2222,
            container: 22,
        },
    );
    let mut services = BTreeMap::new();
    services.insert(
        "gitea".to_string(),
        Service {
            public: true,
            networks: vec![ServiceNetwork::Global, ServiceNetwork::Private],
        },
    );
    services.insert(
        "gitea-db".to_string(),
        Service {
            public: false,
            networks: vec![ServiceNetwork::Private],
        },
    );
    let manifest = AppManifest {
        version: 1,
        name: context.app_name.clone(),
        domain: context.domain.clone(),
        upstreams,
        ports,
        services,
        data: vec!["./data/gitea".to_string(), "./data/postgres".to_string()],
    };
    manifest.validate()?;
    Ok(RenderedTemplate {
        manifest,
        compose: render_variables(GITEA_COMPOSE, context)?,
        env: render_variables(GITEA_ENV, context)?,
        readme: render_variables(GITEA_README, context)?,
        data_dirs: vec!["data/gitea".to_string(), "data/postgres".to_string()],
    })
}

const BASIC_HTTP_COMPOSE: &str = r#"services:
  {{ app_name }}:
    image: nginx:alpine
    container_name: {{ app_name }}
    restart: unless-stopped
    networks:
      - {{ global_network }}

networks:
  {{ global_network }}:
    external: true
    name: {{ global_network }}
"#;

const BASIC_HTTP_ENV: &str = "APP_NAME={{ app_name }}\nDOMAIN={{ domain }}\n";

const BASIC_HTTP_README: &str = r#"# {{ app_name }}

Basic HTTP app for LocalLabStack.

Public domain: https://{{ domain }}
"#;

const GITEA_COMPOSE: &str = r#"services:
  gitea:
    image: gitea/gitea:latest
    container_name: gitea
    restart: unless-stopped
    environment:
      USER_UID: ${GITEA_USER_UID}
      USER_GID: ${GITEA_USER_GID}
      GITEA__database__DB_TYPE: postgres
      GITEA__database__HOST: gitea-db:5432
      GITEA__database__NAME: ${POSTGRES_DB}
      GITEA__database__USER: ${POSTGRES_USER}
      GITEA__database__PASSWD: ${POSTGRES_PASSWORD}
    volumes:
      - ./data/gitea:/data
    ports:
      - "2222:22"
    depends_on:
      - gitea-db
    networks:
      - {{ global_network }}
      - {{ private_network }}

  gitea-db:
    image: postgres:16-alpine
    container_name: gitea-db
    restart: unless-stopped
    environment:
      POSTGRES_DB: ${POSTGRES_DB}
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - ./data/postgres:/var/lib/postgresql/data
    networks:
      - {{ private_network }}

networks:
  {{ global_network }}:
    external: true
    name: {{ global_network }}
  {{ private_network }}:
    name: {{ private_network }}
    driver: bridge
"#;

const GITEA_ENV: &str = r#"GITEA_USER_UID=1000
GITEA_USER_GID=1000
POSTGRES_DB=gitea
POSTGRES_USER=gitea
POSTGRES_PASSWORD=change-me
"#;

const GITEA_README: &str = r#"# {{ app_name }}

Gitea with PostgreSQL for LocalLabStack.

Public domain: https://{{ domain }}

Change placeholder passwords in `.env` before long-lived use.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> TemplateContext {
        TemplateContext {
            app_name: "gitea".to_string(),
            domain: "gitea.locallab".to_string(),
            global_network: "locallabstack-global".to_string(),
            private_network: "llstk-gitea-private".to_string(),
            container_prefix: "gitea".to_string(),
            data_dir: "./data".to_string(),
        }
    }

    #[test]
    fn lists_required_templates() {
        let names = list_templates()
            .into_iter()
            .map(|template| template.name)
            .collect::<Vec<_>>();
        assert!(names.contains(&"basic-http"));
        assert!(names.contains(&"gitea-postgres"));
    }

    #[test]
    fn rejects_unknown_variable() {
        let result = render_variables("{{ nope }}", &context());
        assert!(result.is_err());
    }

    #[test]
    fn renders_gitea_without_public_http_port() {
        let rendered = render_template("gitea-postgres", &context()).unwrap();
        assert!(!rendered.compose.contains("3000:3000"));
        assert!(rendered.compose.contains("2222:22"));
        assert!(rendered.manifest.validate().is_ok());
    }
}
