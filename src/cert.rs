use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertCommandSpec {
    pub program: OsString,
    pub args: Vec<OsString>,
}

impl CertCommandSpec {
    pub fn mkcert_install() -> Self {
        Self {
            program: OsString::from("mkcert"),
            args: vec![OsString::from("-install")],
        }
    }

    pub fn mkcert_issue(cert: &Path, key: &Path, domains: &[String]) -> Self {
        let mut args = vec![
            OsString::from("-cert-file"),
            cert.as_os_str().to_os_string(),
            OsString::from("-key-file"),
            key.as_os_str().to_os_string(),
        ];
        args.extend(domains.iter().map(OsString::from));
        Self {
            program: OsString::from("mkcert"),
            args,
        }
    }

    pub fn run(&self) -> Result<()> {
        let status = Command::new(&self.program)
            .args(&self.args)
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

    pub fn display(&self) -> String {
        format!(
            "{} {}",
            self.program.to_string_lossy(),
            self.args
                .iter()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CertFilesStatus {
    Missing,
    Present,
    Incomplete,
}

pub fn files_status(cert: &Path, key: &Path) -> CertFilesStatus {
    match (cert.exists(), key.exists()) {
        (true, true) => CertFilesStatus::Present,
        (false, false) => CertFilesStatus::Missing,
        _ => CertFilesStatus::Incomplete,
    }
}

pub fn ensure_mkcert_available() -> Result<()> {
    let status = Command::new("mkcert")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("mkcert is not available; install mkcert and try again")?;
    if !status.success() {
        bail!("mkcert -version failed with status {status}");
    }
    Ok(())
}

pub fn issue_command(cert: PathBuf, key: PathBuf, domains: &[String]) -> CertCommandSpec {
    CertCommandSpec::mkcert_issue(&cert, &key, domains)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_issue_without_shell() {
        let spec = CertCommandSpec::mkcert_issue(
            Path::new("/tmp/a cert.pem"),
            Path::new("/tmp/a key.pem"),
            &["locallab".to_string(), "*.locallab".to_string()],
        );
        assert_eq!(spec.program, OsString::from("mkcert"));
        assert!(spec.args.contains(&OsString::from("/tmp/a cert.pem")));
    }
}
