use std::fs;

use anyhow::{Result, bail};

use crate::cert as cert_core;
use crate::cli::{CertCommand, CertIssueArgs, Context as CliContext};
use crate::manifest::RootManifest;

pub fn run(context: &CliContext, command: CertCommand) -> Result<()> {
    match command {
        CertCommand::Init => init(),
        CertCommand::Issue(args) => issue(context, args),
        CertCommand::Status => status(context),
    }
}

fn init() -> Result<()> {
    cert_core::CertCommandSpec::mkcert_install().run()
}

fn issue(context: &CliContext, args: CertIssueArgs) -> Result<()> {
    let root = RootManifest::read_from(&context.layout.root_manifest_path())?;
    if root.cert.provider != "mkcert" {
        bail!("unsupported cert provider {}", root.cert.provider);
    }
    fs::create_dir_all(context.layout.certs_issued_dir())?;
    let cert = context.layout.cert_path();
    let key = context.layout.key_path();
    let apps = context.layout.read_apps()?;
    let domains = cert_core::effective_domains(&root.cert.domains, &apps);
    match cert_core::files_status(&cert, &key) {
        cert_core::CertFilesStatus::Present if !args.force => {
            println!("certificate already exists, skipped {}", cert.display());
            return Ok(());
        }
        cert_core::CertFilesStatus::Incomplete if !args.force => {
            bail!("certificate files are incomplete; use --force to regenerate");
        }
        _ => {}
    }
    cert_core::issue_command(cert, key, &domains).run()
}

fn status(context: &CliContext) -> Result<()> {
    let root = RootManifest::read_from(&context.layout.root_manifest_path())?;
    let apps = context.layout.read_apps()?;
    let domains = cert_core::effective_domains(&root.cert.domains, &apps);
    println!("provider: {}", root.cert.provider);
    println!("configured domains: {}", root.cert.domains.join(", "));
    println!("effective domains: {}", domains.join(", "));
    println!(
        "mkcert: {}",
        if cert_core::ensure_mkcert_available().is_ok() {
            "available"
        } else {
            "missing"
        }
    );
    let cert = context.layout.cert_path();
    let key = context.layout.key_path();
    println!(
        "certificate: {:?} {}",
        cert_core::files_status(&cert, &key),
        cert.display()
    );
    println!("private key: {}", key.display());
    Ok(())
}
