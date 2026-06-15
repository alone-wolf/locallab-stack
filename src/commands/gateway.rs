use anyhow::{Result, bail};

use crate::cli::{
    Context as CliContext, GatewayCommand, GatewayLogsArgs, GatewayRenderArgs, GatewayUpArgs,
};
use crate::compose;
use crate::gateway as gateway_core;
use crate::manifest::RootManifest;

pub fn run(context: &CliContext, command: GatewayCommand) -> Result<()> {
    match command {
        GatewayCommand::Up(args) => up(context, args),
        GatewayCommand::Down => lifecycle(context, compose::down),
        GatewayCommand::Restart => lifecycle(context, compose::restart),
        GatewayCommand::Logs(args) => logs(context, args),
        GatewayCommand::Render(args) => render(context, args),
        GatewayCommand::Reload => reload(context),
        GatewayCommand::Status => status(context),
    }
}

fn load(context: &CliContext) -> Result<(RootManifest, Vec<gateway_core::Route>)> {
    let root = RootManifest::read_from(&context.layout.root_manifest_path())?;
    let apps = context.layout.read_apps()?;
    let routes = gateway_core::collect_routes(&apps)?;
    Ok((root, routes))
}

fn up(context: &CliContext, args: GatewayUpArgs) -> Result<()> {
    let _ = args.detach;
    lifecycle(context, compose::up)
}

fn lifecycle(
    context: &CliContext,
    builder: fn(&std::path::Path) -> compose::ComposeCommand,
) -> Result<()> {
    ensure_gateway_compose(context)?;
    builder(context.layout.root_dir()).run()
}

fn logs(context: &CliContext, args: GatewayLogsArgs) -> Result<()> {
    ensure_gateway_compose(context)?;
    compose::logs_service(context.layout.root_dir(), "gateway", args.follow, args.tail).run()
}

fn render(context: &CliContext, args: GatewayRenderArgs) -> Result<()> {
    let (root, routes) = load(context)?;
    let content = gateway_core::render_caddyfile(&root, &routes);
    let path = context.layout.gateway_caddyfile_path();
    if args.check {
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        if existing == content {
            println!("{} is up to date", path.display());
            return Ok(());
        }
        bail!(
            "{} is out of date; run llstk gateway render",
            path.display()
        );
    }
    gateway_core::write_caddyfile(&path, &content, args.force)?;
    println!("rendered {}", path.display());
    Ok(())
}

fn status(context: &CliContext) -> Result<()> {
    let (root, routes) = load(context)?;
    println!("provider: {}", root.gateway.provider);
    println!("container: {}", root.gateway.container);
    println!(
        "caddyfile: {}",
        context.layout.gateway_caddyfile_path().display()
    );
    println!("apps: {}", context.layout.read_apps()?.len());
    println!("routes:");
    for route in routes {
        println!("  {} -> {}:{}", route.domain, route.container, route.port);
    }
    Ok(())
}

fn reload(context: &CliContext) -> Result<()> {
    ensure_gateway_compose(context)?;
    if !context.layout.gateway_caddyfile_path().exists() {
        bail!(
            "missing {}",
            context.layout.gateway_caddyfile_path().display()
        );
    }
    compose::gateway_reload(context.layout.root_dir()).run()
}

fn ensure_gateway_compose(context: &CliContext) -> Result<()> {
    if !context.layout.root_compose_path().exists() {
        bail!("missing {}", context.layout.root_compose_path().display());
    }
    Ok(())
}
