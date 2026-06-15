use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::cli::{Context as CliContext, HostsCommand, HostsFormat, HostsPlanArgs, HostsSyncArgs};
use crate::gateway;
use crate::hosts as hosts_core;
use crate::manifest::RootManifest;

pub fn run(context: &CliContext, command: HostsCommand) -> Result<()> {
    match command {
        HostsCommand::Plan(args) => plan(context, args),
        HostsCommand::Status(args) => status(context, &args.hosts_file),
        HostsCommand::Sync(args) => sync(context, args),
    }
}

fn planned(context: &CliContext) -> Result<(RootManifest, Vec<hosts_core::HostRecord>)> {
    let root = RootManifest::read_from(&context.layout.root_manifest_path())?;
    let apps = context.layout.read_apps()?;
    let routes = gateway::collect_routes(&apps)?;
    let records = hosts_core::plan_records(root.hosts.enabled, &root.hosts.ip, &routes)?;
    Ok((root, records))
}

fn plan(context: &CliContext, args: HostsPlanArgs) -> Result<()> {
    let (root, records) = planned(context)?;
    if !root.hosts.enabled {
        println!("hosts management disabled");
        return Ok(());
    }
    if records.is_empty() {
        println!("no public app domains found");
        return Ok(());
    }
    match args.format {
        HostsFormat::Text => println!("{}", hosts_core::render_text(&records)),
        HostsFormat::Block => println!("{}", hosts_core::render_block(&records)),
    }
    Ok(())
}

fn status(context: &CliContext, hosts_file: &Path) -> Result<()> {
    let (root, records) = planned(context)?;
    let planned_block = hosts_core::render_block(&records);
    let content = hosts_core::read_hosts(hosts_file)?;
    let status = hosts_core::block_status(&content, &planned_block)?;
    println!("enabled: {}", root.hosts.enabled);
    println!("planned records: {}", records.len());
    println!(
        "managed block: {}",
        match status {
            hosts_core::BlockStatus::Missing => "missing",
            hosts_core::BlockStatus::Present => "present",
            hosts_core::BlockStatus::OutOfDate => "present",
        }
    );
    println!(
        "status: {}",
        match status {
            hosts_core::BlockStatus::Missing => "missing",
            hosts_core::BlockStatus::Present => "up to date",
            hosts_core::BlockStatus::OutOfDate => "out of date",
        }
    );
    Ok(())
}

fn sync(context: &CliContext, args: HostsSyncArgs) -> Result<()> {
    let (root, records) = planned(context)?;
    if !root.hosts.enabled {
        println!("hosts management disabled");
        return Ok(());
    }
    if !args.dry_run && !args.yes {
        bail!("hosts sync requires --yes in non-interactive mode, or use --dry-run");
    }
    let planned_block = hosts_core::render_block(&records);
    let content = hosts_core::read_hosts(&args.hosts_file)?;
    let updated = hosts_core::replace_block(&content, &planned_block)?;
    if args.dry_run {
        println!("would update {}", args.hosts_file.display());
        println!("{planned_block}");
        return Ok(());
    }
    if let Err(error) = hosts_core::write_hosts(&args.hosts_file, &updated) {
        if is_permission_denied(&error) {
            bail!("{}", permission_help(&args.hosts_file, context));
        }
        return Err(error);
    }
    println!("updated {}", args.hosts_file.display());
    Ok(())
}

fn is_permission_denied(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|io_error| io_error.kind() == io::ErrorKind::PermissionDenied)
    })
}

fn permission_help(hosts_file: &Path, context: &CliContext) -> String {
    let root_arg = absolutize(context.layout.root_dir());
    let exe = std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "llstk".to_string());
    format!(
        "permission denied while updating {hosts_file}. LocalLabStack does not call sudo automatically.\n\
         Preview the managed block first:\n  {exe} --root {root} hosts sync --dry-run --hosts-file {hosts_file}\n\
         Then rerun with appropriate privileges:\n  sudo {exe} --root {root} hosts sync --yes --hosts-file {hosts_file}\n\
         Only the # BEGIN LocalLabStack / # END LocalLabStack block will be added or replaced.",
        hosts_file = hosts_file.display(),
        root = root_arg.display()
    )
}

fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::WorkspaceLayout;

    #[test]
    fn permission_help_mentions_sudo_and_dry_run() {
        let context = CliContext {
            layout: WorkspaceLayout::new(PathBuf::from(".locallab")),
        };
        let message = permission_help(Path::new("/etc/hosts"), &context);
        assert!(message.contains("does not call sudo automatically"));
        assert!(message.contains("hosts sync --dry-run"));
        assert!(message.contains("sudo"));
        assert!(message.contains("# BEGIN LocalLabStack"));
    }
}
