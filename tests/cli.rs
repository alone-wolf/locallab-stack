use std::fs;
use std::os::unix::fs::PermissionsExt;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

fn llstk() -> Command {
    Command::cargo_bin("llstk").unwrap()
}

#[test]
fn help_outputs_command_tree() {
    llstk()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("LocalLabStack"));

    llstk()
        .args(["app", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("up"))
        .stdout(predicate::str::contains("down"));

    llstk()
        .args(["gateway", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("render"))
        .stdout(predicate::str::contains("up"))
        .stdout(predicate::str::contains("logs"));

    llstk()
        .args(["stack", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("up"))
        .stdout(predicate::str::contains("down"))
        .stdout(predicate::str::contains("logs"));
}

#[test]
fn init_creates_workspace_and_is_idempotent() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".lab");

    llstk()
        .current_dir(temp.path())
        .args(["--root", root.to_str().unwrap(), "init"])
        .assert()
        .success();

    assert!(root.join("llstk.yml").exists());
    assert!(root.join("docker-compose.yml").exists());
    assert!(root.join("gateway/Caddyfile").exists());

    let marker = "user edit";
    fs::write(root.join("README.md"), marker).unwrap();
    llstk()
        .current_dir(temp.path())
        .args(["--root", root.to_str().unwrap(), "init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("exists, skipped"));
    assert_eq!(fs::read_to_string(root.join("README.md")).unwrap(), marker);
}

#[test]
fn app_create_gateway_and_hosts_plan_form_mvp_loop() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".locallab");

    llstk()
        .current_dir(temp.path())
        .args(["--root", root.to_str().unwrap(), "init"])
        .assert()
        .success();

    llstk()
        .current_dir(temp.path())
        .args([
            "--root",
            root.to_str().unwrap(),
            "app",
            "create",
            "gitea",
            "--template",
            "gitea-postgres",
        ])
        .assert()
        .success();

    let compose = fs::read_to_string(root.join("lab-app-gitea/docker-compose.yml")).unwrap();
    assert!(!compose.contains("3000:3000"));
    assert!(compose.contains("2222:22"));

    llstk()
        .current_dir(temp.path())
        .args(["--root", root.to_str().unwrap(), "gateway", "render"])
        .assert()
        .success();

    let caddy = fs::read_to_string(root.join("gateway/Caddyfile")).unwrap();
    assert!(caddy.contains("gitea.locallab"));
    assert!(caddy.contains("reverse_proxy gitea:3000"));

    llstk()
        .current_dir(temp.path())
        .args(["--root", root.to_str().unwrap(), "hosts", "plan"])
        .assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1 gitea.locallab"));
}

#[test]
fn stack_status_reports_apps_and_routes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".locallab");

    llstk()
        .current_dir(temp.path())
        .args(["--root", root.to_str().unwrap(), "init"])
        .assert()
        .success();
    llstk()
        .current_dir(temp.path())
        .args([
            "--root",
            root.to_str().unwrap(),
            "app",
            "create",
            "gitea",
            "--template",
            "gitea-postgres",
        ])
        .assert()
        .success();
    llstk()
        .current_dir(temp.path())
        .args(["--root", root.to_str().unwrap(), "app", "show", "gitea"])
        .assert()
        .success()
        .stdout(predicate::str::contains("endpoints:"))
        .stdout(predicate::str::contains("web page"))
        .stdout(predicate::str::contains("https://gitea.locallab"))
        .stdout(predicate::str::contains("ssh access"))
        .stdout(predicate::str::contains("git@gitea.locallab:2222"));

    llstk()
        .current_dir(temp.path())
        .args(["--root", root.to_str().unwrap(), "stack", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("apps: 1"))
        .stdout(predicate::str::contains("gitea gitea.locallab"))
        .stdout(predicate::str::contains("gitea.locallab -> gitea:3000"));
}

#[test]
fn app_create_requires_initialized_workspace() {
    let temp = TempDir::new().unwrap();
    llstk()
        .current_dir(temp.path())
        .args(["app", "create", "gitea", "--template", "gitea-postgres"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("workspace is not initialized"));
}

#[test]
fn hosts_sync_uses_temp_hosts_file() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".locallab");
    let hosts = temp.path().join("hosts");
    fs::write(&hosts, "127.0.0.1 localhost\n").unwrap();

    llstk()
        .current_dir(temp.path())
        .args(["--root", root.to_str().unwrap(), "init"])
        .assert()
        .success();
    llstk()
        .current_dir(temp.path())
        .args([
            "--root",
            root.to_str().unwrap(),
            "app",
            "create",
            "gitea",
            "--template",
            "gitea-postgres",
        ])
        .assert()
        .success();
    llstk()
        .current_dir(temp.path())
        .args([
            "--root",
            root.to_str().unwrap(),
            "hosts",
            "sync",
            "--yes",
            "--hosts-file",
            hosts.to_str().unwrap(),
        ])
        .assert()
        .success();

    let content = fs::read_to_string(&hosts).unwrap();
    assert!(content.contains("# BEGIN LocalLabStack"));
    assert!(content.contains("127.0.0.1 gitea.locallab"));
}

#[test]
fn hosts_sync_permission_denied_explains_manual_privilege_path() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".locallab");
    let readonly_dir = temp.path().join("readonly");
    fs::create_dir(&readonly_dir).unwrap();
    let hosts = readonly_dir.join("hosts");
    fs::write(&hosts, "127.0.0.1 localhost\n").unwrap();
    let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
    perms.set_mode(0o555);
    fs::set_permissions(&readonly_dir, perms).unwrap();

    llstk()
        .current_dir(temp.path())
        .args(["--root", root.to_str().unwrap(), "init"])
        .assert()
        .success();
    llstk()
        .current_dir(temp.path())
        .args([
            "--root",
            root.to_str().unwrap(),
            "app",
            "create",
            "gitea",
            "--template",
            "gitea-postgres",
        ])
        .assert()
        .success();
    llstk()
        .current_dir(temp.path())
        .args([
            "--root",
            root.to_str().unwrap(),
            "hosts",
            "sync",
            "--yes",
            "--hosts-file",
            hosts.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "LocalLabStack does not call sudo automatically",
        ))
        .stderr(predicate::str::contains("hosts sync --dry-run"))
        .stderr(predicate::str::contains("sudo"));

    let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&readonly_dir, perms).unwrap();
}
