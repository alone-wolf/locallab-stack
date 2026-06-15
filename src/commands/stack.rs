use anyhow::{Result, bail};

use crate::cert;
use crate::cli::{
    Context as CliContext, StackCommand, StackLogsArgs, StackRestartArgs, StackUpArgs,
};
use crate::compose;
use crate::gateway as gateway_core;
use crate::hosts as hosts_core;
use crate::manifest::RootManifest;

pub fn run(context: &CliContext, command: StackCommand) -> Result<()> {
    match command {
        StackCommand::Up(args) => up(context, args),
        StackCommand::Down => down(context),
        StackCommand::Restart(args) => restart(context, args),
        StackCommand::Logs(args) => logs(context, args),
        StackCommand::Status => status(context),
    }
}

fn up(context: &CliContext, args: StackUpArgs) -> Result<()> {
    let root = RootManifest::read_from(&context.layout.root_manifest_path())?;
    if args.cert {
        issue_cert_if_needed(context, &root)?;
    }
    if args.render {
        render_gateway(context, &root)?;
    }
    ensure_gateway_compose(context)?;
    println!("starting gateway");
    compose::up(context.layout.root_dir()).run()?;

    for app in context.layout.read_apps()? {
        println!("starting app {}", app.name);
        compose::up(&context.layout.app_dir(&app.name)).run()?;
    }

    if args.hosts {
        print_hosts_plan(context, &root)?;
    }
    Ok(())
}

fn down(context: &CliContext) -> Result<()> {
    for app in context.layout.read_apps()?.into_iter().rev() {
        println!("stopping app {}", app.name);
        compose::down(&context.layout.app_dir(&app.name)).run()?;
    }
    ensure_gateway_compose(context)?;
    println!("stopping gateway");
    compose::down(context.layout.root_dir()).run()
}

fn restart(context: &CliContext, args: StackRestartArgs) -> Result<()> {
    let root = RootManifest::read_from(&context.layout.root_manifest_path())?;
    if args.render {
        render_gateway(context, &root)?;
    }
    ensure_gateway_compose(context)?;
    println!("restarting gateway");
    compose::restart(context.layout.root_dir()).run()?;
    for app in context.layout.read_apps()? {
        println!("restarting app {}", app.name);
        compose::restart(&context.layout.app_dir(&app.name)).run()?;
    }
    Ok(())
}

fn logs(context: &CliContext, args: StackLogsArgs) -> Result<()> {
    ensure_gateway_compose(context)?;
    println!("gateway logs:");
    compose::logs_service(context.layout.root_dir(), "gateway", args.follow, args.tail).run()?;
    for app in context.layout.read_apps()? {
        println!("app {} logs:", app.name);
        compose::logs(&context.layout.app_dir(&app.name), args.follow, args.tail).run()?;
    }
    Ok(())
}

fn status(context: &CliContext) -> Result<()> {
    let root_exists = context.layout.root_manifest_path().exists();
    println!("root: {}", context.layout.root_dir().display());
    println!(
        "root manifest: {}",
        if root_exists { "present" } else { "missing" }
    );
    if !root_exists {
        return Ok(());
    }
    let root = RootManifest::read_from(&context.layout.root_manifest_path())?;
    let apps = context.layout.read_apps()?;
    let routes = gateway_core::collect_routes(&apps)?;
    println!(
        "gateway compose: {}",
        context.layout.root_compose_path().display()
    );
    println!("gateway container: {}", root.gateway.container);
    println!("apps: {}", apps.len());
    for app in &apps {
        println!("  {} {}", app.name, app.domain);
    }
    println!("public routes: {}", routes.len());
    for route in &routes {
        println!("  {} -> {}:{}", route.domain, route.container, route.port);
    }
    Ok(())
}

fn render_gateway(context: &CliContext, root: &RootManifest) -> Result<()> {
    let apps = context.layout.read_apps()?;
    let routes = gateway_core::collect_routes(&apps)?;
    let content = gateway_core::render_caddyfile(root, &routes);
    gateway_core::write_caddyfile(&context.layout.gateway_caddyfile_path(), &content, false)?;
    println!(
        "rendered {}",
        context.layout.gateway_caddyfile_path().display()
    );
    Ok(())
}

fn issue_cert_if_needed(context: &CliContext, root: &RootManifest) -> Result<()> {
    if root.cert.provider != "mkcert" {
        bail!("unsupported cert provider {}", root.cert.provider);
    }
    std::fs::create_dir_all(context.layout.certs_issued_dir())?;
    match cert::files_status(&context.layout.cert_path(), &context.layout.key_path()) {
        cert::CertFilesStatus::Present => {
            println!("certificate already exists, skipped");
            Ok(())
        }
        cert::CertFilesStatus::Missing => {
            println!("issuing certificate");
            let apps = context.layout.read_apps()?;
            let domains = cert::effective_domains(&root.cert.domains, &apps);
            cert::issue_command(
                context.layout.cert_path(),
                context.layout.key_path(),
                &domains,
            )
            .run()
        }
        cert::CertFilesStatus::Incomplete => {
            bail!("certificate files are incomplete; run llstk cert issue --force");
        }
    }
}

fn print_hosts_plan(context: &CliContext, root: &RootManifest) -> Result<()> {
    let apps = context.layout.read_apps()?;
    let routes = gateway_core::collect_routes(&apps)?;
    let records = hosts_core::plan_records(root.hosts.enabled, &root.hosts.ip, &routes)?;
    if !root.hosts.enabled {
        println!("hosts management disabled");
    } else if records.is_empty() {
        println!("no public app domains found");
    } else {
        println!("hosts plan:");
        println!("{}", hosts_core::render_text(&records));
        println!("run `llstk hosts sync --yes` with appropriate privileges to update /etc/hosts");
    }
    Ok(())
}

fn ensure_gateway_compose(context: &CliContext) -> Result<()> {
    if !context.layout.root_compose_path().exists() {
        bail!("missing {}", context.layout.root_compose_path().display());
    }
    Ok(())
}
