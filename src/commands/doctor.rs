use anyhow::Result;

use crate::cli::Context as CliContext;
use crate::compose;
use crate::manifest::RootManifest;

pub fn run(context: &CliContext) -> Result<()> {
    let mut errors = 0usize;
    match compose::check_docker_compose() {
        Ok(()) => println!("ok docker compose"),
        Err(error) => {
            errors += 1;
            println!("error docker compose: {error:#}");
        }
    }
    match RootManifest::read_from(&context.layout.root_manifest_path()) {
        Ok(_) => println!(
            "ok root manifest {}",
            context.layout.root_manifest_path().display()
        ),
        Err(error) => {
            errors += 1;
            println!("error root manifest: {error:#}");
        }
    }
    check_path(
        "gateway compose",
        &context.layout.root_compose_path(),
        &mut errors,
    );
    check_path(
        "gateway Caddyfile",
        &context.layout.gateway_caddyfile_path(),
        &mut errors,
    );
    for (name, path) in context.layout.discover_app_manifests()? {
        match crate::manifest::AppManifest::read_from(&path) {
            Ok(_) => println!("ok app manifest {name} {}", path.display()),
            Err(error) => {
                errors += 1;
                println!("error app manifest {name}: {error:#}");
            }
        }
        check_path(
            "app compose",
            &context.layout.app_compose_path(&name),
            &mut errors,
        );
    }
    if !context.layout.cert_path().exists() {
        println!(
            "warn cert file missing {}",
            context.layout.cert_path().display()
        );
    }
    if errors > 0 {
        anyhow::bail!("doctor found {errors} error(s)");
    }
    Ok(())
}

fn check_path(label: &str, path: &std::path::Path, errors: &mut usize) {
    if path.exists() {
        println!("ok {label} {}", path.display());
    } else {
        *errors += 1;
        println!("error {label} missing {}", path.display());
    }
}
