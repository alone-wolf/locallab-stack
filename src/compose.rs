use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposeCommand {
    pub dir: PathBuf,
    pub args: Vec<OsString>,
}

impl ComposeCommand {
    pub fn new(
        dir: impl Into<PathBuf>,
        args: impl IntoIterator<Item = impl Into<OsString>>,
    ) -> Self {
        Self {
            dir: dir.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    pub fn display(&self) -> String {
        format!(
            "cd {} && docker compose {}",
            self.dir.display(),
            self.args
                .iter()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }

    pub fn run(&self) -> Result<()> {
        let status = Command::new("docker")
            .arg("compose")
            .args(&self.args)
            .current_dir(&self.dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("failed to run {}", self.display()))?;
        if !status.success() {
            bail!("command failed with status {status}: {}", self.display());
        }
        Ok(())
    }
}

pub fn check_docker_compose() -> Result<()> {
    let status = Command::new("docker")
        .arg("compose")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to run docker compose version")?;
    if !status.success() {
        bail!("docker compose version failed with status {status}");
    }
    Ok(())
}

pub fn up(dir: &Path) -> ComposeCommand {
    ComposeCommand::new(dir, ["up", "-d"])
}

pub fn down(dir: &Path) -> ComposeCommand {
    ComposeCommand::new(dir, ["down"])
}

pub fn restart(dir: &Path) -> ComposeCommand {
    ComposeCommand::new(dir, ["restart"])
}

pub fn logs(dir: &Path, follow: bool, tail: Option<u32>) -> ComposeCommand {
    let mut args = vec![OsString::from("logs")];
    if follow {
        args.push(OsString::from("--follow"));
    }
    if let Some(tail) = tail {
        args.push(OsString::from("--tail"));
        args.push(OsString::from(tail.to_string()));
    }
    ComposeCommand::new(dir, args)
}

pub fn logs_service(dir: &Path, service: &str, follow: bool, tail: Option<u32>) -> ComposeCommand {
    let mut args = vec![OsString::from("logs")];
    if follow {
        args.push(OsString::from("--follow"));
    }
    if let Some(tail) = tail {
        args.push(OsString::from("--tail"));
        args.push(OsString::from(tail.to_string()));
    }
    args.push(OsString::from(service));
    ComposeCommand::new(dir, args)
}

pub fn remove(dir: &Path) -> ComposeCommand {
    ComposeCommand::new(dir, ["down", "--remove-orphans"])
}

pub fn gateway_reload(root: &Path) -> ComposeCommand {
    ComposeCommand::new(
        root,
        [
            "exec",
            "gateway",
            "caddy",
            "reload",
            "--config",
            "/etc/caddy/Caddyfile",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_logs_command() {
        let command = logs(Path::new(".locallab/app.gitea"), true, Some(50));
        assert_eq!(
            command.args,
            vec!["logs", "--follow", "--tail", "50"]
                .into_iter()
                .map(OsString::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn builds_service_logs_command() {
        let command = logs_service(Path::new(".locallab"), "gateway", false, Some(20));
        assert_eq!(
            command.args,
            vec!["logs", "--tail", "20", "gateway"]
                .into_iter()
                .map(OsString::from)
                .collect::<Vec<_>>()
        );
    }
}
