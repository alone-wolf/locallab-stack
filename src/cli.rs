use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::commands;
use crate::workspace::WorkspaceLayout;

#[derive(Debug, Parser)]
#[command(
    name = "llstk",
    about = "LocalLabStack local Docker Compose stack manager"
)]
pub struct Cli {
    #[arg(long, global = true, default_value = ".locallab")]
    pub root: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init(InitArgs),
    Status,
    Doctor,
    App(AppArgs),
    Gateway(GatewayArgs),
    Stack(StackArgs),
    Cert(CertArgs),
    Hosts(HostsArgs),
    Template(TemplateArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long, default_value = "default")]
    pub name: String,
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct AppArgs {
    #[command(subcommand)]
    pub command: AppCommand,
}

#[derive(Debug, Subcommand)]
pub enum AppCommand {
    Create(AppCreateArgs),
    List,
    Show(AppNameArgs),
    Up(AppUpArgs),
    Down(AppNameArgs),
    Restart(AppNameArgs),
    Logs(AppLogsArgs),
    Remove(AppRemoveArgs),
    ImportCompose(AppImportComposeArgs),
    MigrateGitea(AppMigrateGiteaArgs),
}

#[derive(Debug, Args)]
pub struct AppCreateArgs {
    pub name: String,
    #[arg(long)]
    pub template: String,
    #[arg(long)]
    pub domain: Option<String>,
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct AppNameArgs {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct AppUpArgs {
    pub name: String,
    #[arg(long)]
    pub detach: bool,
}

#[derive(Debug, Args)]
pub struct AppLogsArgs {
    pub name: String,
    #[arg(long)]
    pub follow: bool,
    #[arg(long)]
    pub tail: Option<u32>,
}

#[derive(Debug, Args)]
pub struct AppRemoveArgs {
    pub name: String,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct AppImportComposeArgs {
    pub path: PathBuf,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct AppMigrateGiteaArgs {
    #[arg(long)]
    pub source: Option<PathBuf>,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct GatewayArgs {
    #[command(subcommand)]
    pub command: GatewayCommand,
}

#[derive(Debug, Subcommand)]
pub enum GatewayCommand {
    Up(GatewayUpArgs),
    Down,
    Restart,
    Logs(GatewayLogsArgs),
    Render(GatewayRenderArgs),
    Reload,
    Status,
}

#[derive(Debug, Args)]
pub struct GatewayUpArgs {
    #[arg(long)]
    pub detach: bool,
}

#[derive(Debug, Args)]
pub struct GatewayLogsArgs {
    #[arg(long)]
    pub follow: bool,
    #[arg(long)]
    pub tail: Option<u32>,
}

#[derive(Debug, Args)]
pub struct GatewayRenderArgs {
    #[arg(long)]
    pub check: bool,
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct StackArgs {
    #[command(subcommand)]
    pub command: StackCommand,
}

#[derive(Debug, Subcommand)]
pub enum StackCommand {
    Up(StackUpArgs),
    Down,
    Restart(StackRestartArgs),
    Logs(StackLogsArgs),
    Status,
}

#[derive(Debug, Args)]
pub struct StackUpArgs {
    #[arg(long)]
    pub render: bool,
    #[arg(long)]
    pub hosts: bool,
    #[arg(long)]
    pub cert: bool,
}

#[derive(Debug, Args)]
pub struct StackRestartArgs {
    #[arg(long)]
    pub render: bool,
}

#[derive(Debug, Args)]
pub struct StackLogsArgs {
    #[arg(long)]
    pub follow: bool,
    #[arg(long)]
    pub tail: Option<u32>,
}

#[derive(Debug, Args)]
pub struct CertArgs {
    #[command(subcommand)]
    pub command: CertCommand,
}

#[derive(Debug, Subcommand)]
pub enum CertCommand {
    Init,
    Issue(CertIssueArgs),
    Status,
}

#[derive(Debug, Args)]
pub struct CertIssueArgs {
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct HostsArgs {
    #[command(subcommand)]
    pub command: HostsCommand,
}

#[derive(Debug, Subcommand)]
pub enum HostsCommand {
    Plan(HostsPlanArgs),
    Status(HostsFileArgs),
    Sync(HostsSyncArgs),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum HostsFormat {
    Text,
    Block,
}

#[derive(Debug, Args)]
pub struct HostsPlanArgs {
    #[arg(long, value_enum, default_value_t = HostsFormat::Text)]
    pub format: HostsFormat,
}

#[derive(Debug, Args)]
pub struct HostsFileArgs {
    #[arg(long, default_value = "/etc/hosts")]
    pub hosts_file: PathBuf,
}

#[derive(Debug, Args)]
pub struct HostsSyncArgs {
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub yes: bool,
    #[arg(long, default_value = "/etc/hosts")]
    pub hosts_file: PathBuf,
}

#[derive(Debug, Args)]
pub struct TemplateArgs {
    #[command(subcommand)]
    pub command: TemplateCommand,
}

#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    List,
    Show(AppNameArgs),
}

pub struct Context {
    pub layout: WorkspaceLayout,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let context = Context {
        layout: WorkspaceLayout::new(cli.root),
    };

    match cli.command {
        Command::Init(args) => commands::init::run(&context, args),
        Command::Status => commands::status::run(&context),
        Command::Doctor => commands::doctor::run(&context),
        Command::App(args) => commands::app::run(&context, args.command),
        Command::Gateway(args) => commands::gateway::run(&context, args.command),
        Command::Stack(args) => commands::stack::run(&context, args.command),
        Command::Cert(args) => commands::cert::run(&context, args.command),
        Command::Hosts(args) => commands::hosts::run(&context, args.command),
        Command::Template(args) => commands::template::run(args.command),
    }
}
