use std::{
    io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[derive(Clone, Debug)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub unix_time: i64,
    pub message: String,
    pub changed_files: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct GitRepositoryInfo {
    pub root: PathBuf,
    pub branch: String,
    pub changed_files: Vec<PathBuf>,
    pub commits: Vec<GitCommit>,
}

pub fn repository_info(
    path: &Path,
    selected_file: Option<&Path>,
) -> io::Result<Option<GitRepositoryInfo>> {
    let root_output = run_git(path, &["rev-parse", "--show-toplevel"])?;
    if !root_output.status.success() {
        return Ok(None);
    }
    let root = PathBuf::from(String::from_utf8_lossy(&root_output.stdout).trim());
    let branch = git_text(&root, &["branch", "--show-current"])?
        .trim()
        .to_string();
    let branch = if branch.is_empty() {
        git_text(&root, &["rev-parse", "--short", "HEAD"])?
            .trim()
            .to_string()
    } else {
        branch
    };
    let status = git_text(&root, &["status", "--porcelain=v1", "-z"])?;
    let changed_files = status
        .split('\0')
        .filter(|line| line.len() >= 4)
        .filter_map(|line| line.get(3..))
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    let mut arguments = vec![
        "log",
        "-n",
        "40",
        "--date=unix",
        "--pretty=format:%H%x1f%h%x1f%an%x1f%at%x1f%s%x1e",
    ];
    let selected_relative = selected_file.and_then(|file| file.strip_prefix(&root).ok());
    if selected_relative.is_some() {
        arguments.push("--");
    }
    let selected_string = selected_relative.map(|path| path.to_string_lossy().into_owned());
    if let Some(path) = selected_string.as_deref() {
        arguments.push(path);
    }
    let log = git_text(&root, &arguments)?;
    let commits = parse_log(&log);
    Ok(Some(GitRepositoryInfo {
        root,
        branch,
        changed_files,
        commits,
    }))
}

pub fn commit_files(root: &Path, hash: &str) -> io::Result<Vec<PathBuf>> {
    let text = git_text(
        root,
        &[
            "show",
            "--pretty=format:",
            "--name-only",
            "--no-renames",
            hash,
        ],
    )?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect())
}

fn parse_log(log: &str) -> Vec<GitCommit> {
    log.split('\u{1e}')
        .filter_map(|record| {
            let fields = record.trim().split('\u{1f}').collect::<Vec<_>>();
            if fields.len() != 5 {
                return None;
            }
            Some(GitCommit {
                hash: fields[0].to_string(),
                short_hash: fields[1].to_string(),
                author: fields[2].to_string(),
                unix_time: fields[3].parse().unwrap_or_default(),
                message: fields[4].to_string(),
                changed_files: Vec::new(),
            })
        })
        .collect()
}

fn git_text(path: &Path, arguments: &[&str]) -> io::Result<String> {
    let output = run_git(path, arguments)?;
    if !output.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_git(path: &Path, arguments: &[&str]) -> io::Result<Output> {
    let mut command = Command::new("git");
    command.arg("-C").arg(path).args(arguments);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command.output()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_git_log_records() {
        let commits = parse_log(
            "abcdef\u{1f}abc\u{1f}Nora\u{1f}123\u{1f}First commit\u{1e}\
             123456\u{1f}123\u{1f}Lee\u{1f}456\u{1f}Second commit\u{1e}",
        );
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].short_hash, "abc");
        assert_eq!(commits[1].message, "Second commit");
    }
}
