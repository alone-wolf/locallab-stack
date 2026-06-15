use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::cli::{
    AppCommand, AppCreateArgs, AppImportComposeArgs, AppLogsArgs, AppMigrateGiteaArgs, AppNameArgs,
    AppRemoveArgs, AppUpArgs, Context as CliContext,
};
use crate::compose;
use crate::manifest::{AppManifest, RootManifest, validate_app_name};
use crate::template::{TemplateContext, render_template, write_rendered_template};

pub fn run(context: &CliContext, command: AppCommand) -> Result<()> {
    match command {
        AppCommand::Create(args) => create(context, args),
        AppCommand::List => list(context),
        AppCommand::Show(args) => show(context, args),
        AppCommand::Up(args) => up(context, args),
        AppCommand::Down(args) => lifecycle(context, &args.name, compose::down),
        AppCommand::Restart(args) => lifecycle(context, &args.name, compose::restart),
        AppCommand::Logs(args) => logs(context, args),
        AppCommand::Remove(args) => remove(context, args),
        AppCommand::ImportCompose(args) => import_compose(args),
        AppCommand::MigrateGitea(args) => migrate_gitea(context, args),
    }
}

fn create(context: &CliContext, args: AppCreateArgs) -> Result<()> {
    validate_app_name(&args.name)?;
    let root = RootManifest::read_from(&context.layout.root_manifest_path())
        .context("workspace is not initialized; run llstk init first")?;
    let domain = args
        .domain
        .clone()
        .unwrap_or_else(|| format!("{}.locallab", args.name));
    let template_context = TemplateContext {
        app_name: args.name.clone(),
        domain,
        global_network: root.network.global,
        private_network: format!("llstk-{}-private", args.name),
        container_prefix: args.name.clone(),
        data_dir: "./data".to_string(),
    };
    let rendered = render_template(&args.template, &template_context)?;
    let app_dir = context.layout.app_dir(&args.name);
    write_rendered_template(&app_dir, &rendered, args.force)?;
    println!("created {}", app_dir.join("llstk.yml").display());
    println!("created {}", app_dir.join("docker-compose.yml").display());
    println!("created {}", app_dir.join(".env").display());
    Ok(())
}

fn list(context: &CliContext) -> Result<()> {
    let apps = context.layout.read_apps()?;
    for app in apps {
        println!("{}\t{}", app.name, app.domain);
    }
    Ok(())
}

fn show(context: &CliContext, args: AppNameArgs) -> Result<()> {
    let app = AppManifest::read_from(&context.layout.app_manifest_path(&args.name))?;
    println!("name: {}", app.name);
    println!("domain: {}", app.domain);
    for (name, upstream) in app.upstreams {
        println!(
            "upstream: {} -> {}:{} public={}",
            name, upstream.container, upstream.port, upstream.public
        );
    }
    Ok(())
}

fn up(context: &CliContext, args: AppUpArgs) -> Result<()> {
    let _ = args.detach;
    lifecycle(context, &args.name, compose::up)
}

fn lifecycle(
    context: &CliContext,
    name: &str,
    builder: fn(&Path) -> compose::ComposeCommand,
) -> Result<()> {
    let app_dir = context.layout.app_dir(name);
    ensure_app_exists(context, name)?;
    builder(&app_dir).run()
}

fn logs(context: &CliContext, args: AppLogsArgs) -> Result<()> {
    ensure_app_exists(context, &args.name)?;
    compose::logs(&context.layout.app_dir(&args.name), args.follow, args.tail).run()
}

fn remove(context: &CliContext, args: AppRemoveArgs) -> Result<()> {
    ensure_app_exists(context, &args.name)?;
    let command = compose::remove(&context.layout.app_dir(&args.name));
    if args.dry_run {
        println!("{}", command.display());
        return Ok(());
    }
    command.run()?;
    println!(
        "app directory preserved: {}",
        context.layout.app_dir(&args.name).display()
    );
    Ok(())
}

fn ensure_app_exists(context: &CliContext, name: &str) -> Result<()> {
    let path = context.layout.app_compose_path(name);
    if !path.exists() {
        bail!("app {name} does not exist; run llstk app list");
    }
    Ok(())
}

fn import_compose(args: AppImportComposeArgs) -> Result<()> {
    let text = fs::read_to_string(&args.path)
        .with_context(|| format!("failed to read {}", args.path.display()))?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&text)
        .with_context(|| format!("failed to parse {}", args.path.display()))?;
    println!("source compose: {}", args.path.display());
    println!("target app: .locallab/app.{}", args.name);
    if let Some(services) = yaml.get("services").and_then(|value| value.as_mapping()) {
        println!("services:");
        for key in services.keys().filter_map(|key| key.as_str()) {
            println!("  {key}");
        }
    }
    if args.dry_run {
        println!("dry run: no files written");
    }
    Ok(())
}

fn migrate_gitea(context: &CliContext, args: AppMigrateGiteaArgs) -> Result<()> {
    let source = args
        .source
        .unwrap_or_else(|| PathBuf::from("docker-compose.yml"));
    let text = fs::read_to_string(&source)
        .with_context(|| format!("failed to read {}", source.display()))?;
    if !text.contains("gitea") || !text.contains("gitea-db") {
        bail!(
            "{} does not look like a gitea/gitea-db compose file",
            source.display()
        );
    }
    println!("source compose: {}", source.display());
    println!("target app: {}", context.layout.app_dir("gitea").display());
    println!();
    println!("manual data moves:");
    println!(
        "  mv ./gitea {}",
        context.layout.app_dir("gitea").join("data/gitea").display()
    );
    println!(
        "  mv ./postgres {}",
        context
            .layout
            .app_dir("gitea")
            .join("data/postgres")
            .display()
    );
    println!();
    println!("verify:");
    println!("  llstk gateway render");
    println!("  llstk hosts plan");
    println!("  llstk app up gitea");
    if args.dry_run {
        println!("dry run: no files written");
    }
    Ok(())
}
