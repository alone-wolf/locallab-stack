use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::config::{BEGIN_HOSTS_MARKER, END_HOSTS_MARKER};
use crate::gateway::Route;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostRecord {
    pub ip: String,
    pub domain: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockStatus {
    Missing,
    Present,
    OutOfDate,
}

pub fn plan_records(enabled: bool, ip: &str, routes: &[Route]) -> Result<Vec<HostRecord>> {
    if !enabled {
        return Ok(Vec::new());
    }
    if ip.trim().is_empty() {
        bail!("hosts.ip cannot be empty");
    }
    let domains = routes
        .iter()
        .map(|route| route.domain.clone())
        .collect::<BTreeSet<_>>();
    Ok(domains
        .into_iter()
        .map(|domain| HostRecord {
            ip: ip.to_string(),
            domain,
        })
        .collect())
}

pub fn render_text(records: &[HostRecord]) -> String {
    records
        .iter()
        .map(|record| format!("{} {}", record.ip, record.domain))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_block(records: &[HostRecord]) -> String {
    let mut output = String::new();
    output.push_str(BEGIN_HOSTS_MARKER);
    output.push('\n');
    let text = render_text(records);
    if !text.is_empty() {
        output.push_str(&text);
        output.push('\n');
    }
    output.push_str(END_HOSTS_MARKER);
    output
}

pub fn find_managed_blocks(content: &str) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let mut search_from = 0;
    while let Some(begin_rel) = content[search_from..].find(BEGIN_HOSTS_MARKER) {
        let begin = search_from + begin_rel;
        let after_begin = begin + BEGIN_HOSTS_MARKER.len();
        if let Some(end_rel) = content[after_begin..].find(END_HOSTS_MARKER) {
            let end_marker_start = after_begin + end_rel;
            let end = end_marker_start + END_HOSTS_MARKER.len();
            blocks.push((begin, end));
            search_from = end;
        } else {
            break;
        }
    }
    blocks
}

pub fn block_status(content: &str, planned_block: &str) -> Result<BlockStatus> {
    let blocks = find_managed_blocks(content);
    match blocks.len() {
        0 => Ok(BlockStatus::Missing),
        1 => {
            let (start, end) = blocks[0];
            if content[start..end].trim() == planned_block.trim() {
                Ok(BlockStatus::Present)
            } else {
                Ok(BlockStatus::OutOfDate)
            }
        }
        _ => bail!("multiple LocalLabStack hosts blocks found"),
    }
}

pub fn replace_block(content: &str, planned_block: &str) -> Result<String> {
    let blocks = find_managed_blocks(content);
    match blocks.len() {
        0 => {
            let mut output = content.to_string();
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(planned_block);
            output.push('\n');
            Ok(output)
        }
        1 => {
            let (start, end) = blocks[0];
            let mut output = String::new();
            output.push_str(&content[..start]);
            output.push_str(planned_block);
            output.push_str(&content[end..]);
            Ok(output)
        }
        _ => bail!("multiple LocalLabStack hosts blocks found"),
    }
}

pub fn read_hosts(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

pub fn write_hosts(path: &Path, content: &str) -> Result<()> {
    let tmp = path.with_extension("llstk.tmp");
    fs::write(&tmp, content).with_context(|| format!("failed to write {}", tmp.display()))?;
    if let Ok(metadata) = fs::metadata(path) {
        fs::set_permissions(&tmp, metadata.permissions())
            .with_context(|| format!("failed to set permissions on {}", tmp.display()))?;
    }
    fs::rename(&tmp, path).with_context(|| format!("failed to replace {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_block() {
        let block = render_block(&[HostRecord {
            ip: "127.0.0.1".to_string(),
            domain: "gitea.locallab".to_string(),
        }]);
        assert!(block.contains(BEGIN_HOSTS_MARKER));
        assert!(block.contains("127.0.0.1 gitea.locallab"));
    }

    #[test]
    fn replaces_existing_block() {
        let old = "127.0.0.1 localhost\n# BEGIN LocalLabStack\nold\n# END LocalLabStack\n";
        let new = render_block(&[]);
        let replaced = replace_block(old, &new).unwrap();
        assert!(replaced.contains("127.0.0.1 localhost"));
        assert!(!replaced.contains("old"));
    }
}
