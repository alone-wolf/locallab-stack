use anyhow::Result;

use crate::cli::Context as CliContext;
use crate::gateway;
use crate::manifest::RootManifest;

pub fn run(context: &CliContext) -> Result<()> {
    println!("root: {}", context.layout.root_dir().display());
    let root_exists = context.layout.root_manifest_path().exists();
    println!(
        "root manifest: {}",
        if root_exists { "present" } else { "missing" }
    );
    if root_exists {
        let _root = RootManifest::read_from(&context.layout.root_manifest_path())?;
        let apps = context.layout.read_apps()?;
        let routes = gateway::collect_routes(&apps)?;
        println!("apps: {}", apps.len());
        println!("public routes: {}", routes.len());
    }
    println!(
        "gateway Caddyfile: {}",
        if context.layout.gateway_caddyfile_path().exists() {
            "present"
        } else {
            "missing"
        }
    );
    Ok(())
}
