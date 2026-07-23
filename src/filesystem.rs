use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, HashMap},
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering as AtomicOrdering},
    },
    time::{Duration, SystemTime},
};

use globset::{Glob, GlobMatcher};
use humansize::{BINARY, format_size};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::model::{SortDirection, SortField, SortSpec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: EntryKind,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub readonly: bool,
    pub hidden: bool,
}

impl FileEntry {
    pub fn is_directory(&self) -> bool {
        self.kind == EntryKind::Directory
    }

    pub fn kind_label(&self) -> String {
        match self.kind {
            EntryKind::Directory => "File folder".to_string(),
            EntryKind::Symlink => "Shortcut / link".to_string(),
            EntryKind::Other => "Other".to_string(),
            EntryKind::File => self
                .path
                .extension()
                .and_then(|extension| extension.to_str())
                .filter(|extension| !extension.is_empty())
                .map(|extension| format!("{} file", extension.to_uppercase()))
                .unwrap_or_else(|| "File".to_string()),
        }
    }

    pub fn size_label(&self) -> String {
        if self.is_directory() {
            String::new()
        } else {
            format_size(self.size, BINARY)
        }
    }

    pub fn modified_label(&self) -> String {
        self.modified
            .map(relative_time_label)
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(path)?;
        let file_type = metadata.file_type();
        let kind = if file_type.is_dir() {
            EntryKind::Directory
        } else if file_type.is_file() {
            EntryKind::File
        } else if file_type.is_symlink() {
            EntryKind::Symlink
        } else {
            EntryKind::Other
        };
        Ok(Self {
            name: path
                .file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy()
                .into_owned(),
            path: path.to_path_buf(),
            kind,
            size: metadata.len(),
            modified: metadata.modified().ok(),
            readonly: metadata.permissions().readonly(),
            hidden: is_hidden(path, &metadata),
        })
    }
}

#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn cancel(&self) {
        self.cancelled.store(true, AtomicOrdering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(AtomicOrdering::Acquire)
    }
}

pub fn read_directory_sorted(
    path: &Path,
    sort: SortSpec,
    filter: &str,
    include_hidden: bool,
) -> io::Result<Vec<FileEntry>> {
    let normalized_filter = filter.trim().to_lowercase();
    let mut entries = fs::read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(entry) => match file_entry(entry) {
                Ok(entry)
                    if (include_hidden || !entry.hidden)
                        && (normalized_filter.is_empty()
                            || fuzzy_matches(&entry.name, &normalized_filter)) =>
                {
                    Some(entry)
                }
                Ok(_) => None,
                Err(error) => {
                    tracing::warn!(?error, directory = %path.display(), "failed to read entry metadata");
                    None
                }
            },
            Err(error) => {
                tracing::warn!(?error, directory = %path.display(), "failed to read an entry");
                None
            }
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| compare_entries(left, right, sort));
    Ok(entries)
}

fn file_entry(entry: fs::DirEntry) -> io::Result<FileEntry> {
    FileEntry::from_path(&entry.path())
}

fn compare_entries(left: &FileEntry, right: &FileEntry, sort: SortSpec) -> Ordering {
    let directory_order = match (left.is_directory(), right.is_directory()) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => Ordering::Equal,
    };
    if directory_order != Ordering::Equal {
        return directory_order;
    }

    let ordering = match sort.field {
        SortField::Name => natural_name_cmp(&left.name, &right.name),
        SortField::Modified => left.modified.cmp(&right.modified),
        SortField::Kind => left
            .kind_label()
            .cmp(&right.kind_label())
            .then_with(|| natural_name_cmp(&left.name, &right.name)),
        SortField::Size => left
            .size
            .cmp(&right.size)
            .then_with(|| natural_name_cmp(&left.name, &right.name)),
    };
    match sort.direction {
        SortDirection::Ascending => ordering,
        SortDirection::Descending => ordering.reverse(),
    }
}

fn natural_name_cmp(left: &str, right: &str) -> Ordering {
    left.to_lowercase()
        .cmp(&right.to_lowercase())
        .then_with(|| left.cmp(right))
}

pub fn fuzzy_matches(candidate: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let mut query = query.chars().flat_map(char::to_lowercase);
    let mut current = query.next();
    for character in candidate.chars().flat_map(char::to_lowercase) {
        if current == Some(character) {
            current = query.next();
            if current.is_none() {
                return true;
            }
        }
    }
    current.is_none()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchOptions {
    pub query: String,
    pub glob: Option<String>,
    pub extension: Option<String>,
    pub minimum_size: Option<u64>,
    pub maximum_size: Option<u64>,
    pub modified_after_unix_seconds: Option<u64>,
    pub include_hidden: bool,
    pub maximum_results: usize,
}

impl SearchOptions {
    pub fn parse(input: &str) -> Self {
        let mut options = Self {
            maximum_results: 5_000,
            ..Self::default()
        };
        let mut query = Vec::new();
        for token in input.split_whitespace() {
            if let Some(value) = token.strip_prefix("ext:") {
                options.extension = (!value.is_empty()).then(|| value.to_string());
            } else if let Some(value) = token.strip_prefix("glob:") {
                options.glob = (!value.is_empty()).then(|| value.to_string());
            } else if let Some(value) = token.strip_prefix("min:") {
                options.minimum_size = parse_size_filter(value);
            } else if let Some(value) = token.strip_prefix("max:") {
                options.maximum_size = parse_size_filter(value);
            } else if let Some(value) = token.strip_prefix("hidden:") {
                options.include_hidden = matches!(
                    value.to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                );
            } else {
                query.push(token);
            }
        }
        options.query = query.join(" ");
        options
    }
}

fn parse_size_filter(value: &str) -> Option<u64> {
    let normalized = value.trim().to_ascii_lowercase();
    let split = normalized
        .find(|character: char| !character.is_ascii_digit() && character != '.')
        .unwrap_or(normalized.len());
    let number = normalized[..split].parse::<f64>().ok()?;
    let multiplier = match normalized[split..].trim() {
        "" | "b" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((number * multiplier) as u64)
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub entry: FileEntry,
    pub root: PathBuf,
}

pub fn search_paths(
    roots: &[PathBuf],
    options: &SearchOptions,
    cancellation: &CancellationToken,
) -> io::Result<Vec<SearchResult>> {
    let maximum_results = if options.maximum_results == 0 {
        5_000
    } else {
        options.maximum_results
    };
    let glob = options
        .glob
        .as_deref()
        .filter(|pattern| !pattern.trim().is_empty())
        .map(|pattern| {
            Glob::new(pattern)
                .map(|glob| glob.compile_matcher())
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
        })
        .transpose()?;
    let extension = options
        .extension
        .as_deref()
        .map(|extension| extension.trim_start_matches('.').to_lowercase());
    let modified_after = options
        .modified_after_unix_seconds
        .map(|seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds));
    let query = options.query.trim().to_lowercase();
    let mut results = Vec::new();

    for root in roots {
        if cancellation.is_cancelled() || results.len() >= maximum_results {
            break;
        }
        let walker = WalkBuilder::new(root)
            .hidden(!options.include_hidden)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false)
            .build();
        for item in walker {
            if cancellation.is_cancelled() || results.len() >= maximum_results {
                break;
            }
            let entry = match item {
                Ok(entry) if entry.path() != root => entry,
                Ok(_) => continue,
                Err(error) => {
                    tracing::debug!(?error, root = %root.display(), "search skipped an inaccessible path");
                    continue;
                }
            };
            let path = entry.path();
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy())
                .unwrap_or_default();
            if !fuzzy_matches(&name, &query)
                || glob
                    .as_ref()
                    .is_some_and(|matcher| !matches_glob(matcher, root, path))
                || extension.as_ref().is_some_and(|wanted| {
                    path.extension()
                        .and_then(|value| value.to_str())
                        .map(str::to_lowercase)
                        .as_ref()
                        != Some(wanted)
                })
            {
                continue;
            }
            let file_entry = match FileEntry::from_path(path) {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            if options
                .minimum_size
                .is_some_and(|minimum| file_entry.size < minimum)
                || options
                    .maximum_size
                    .is_some_and(|maximum| file_entry.size > maximum)
                || modified_after
                    .is_some_and(|after| file_entry.modified.is_none_or(|time| time < after))
            {
                continue;
            }
            results.push(SearchResult {
                entry: file_entry,
                root: root.clone(),
            });
        }
    }

    results.sort_by(|left, right| natural_name_cmp(&left.entry.name, &right.entry.name));
    Ok(results)
}

fn matches_glob(matcher: &GlobMatcher, root: &Path, path: &Path) -> bool {
    matcher.is_match(path)
        || path
            .strip_prefix(root)
            .is_ok_and(|relative| matcher.is_match(relative))
}

#[derive(Clone, Debug)]
pub struct FolderInfo {
    pub path: PathBuf,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub readonly: bool,
    pub hidden: bool,
    pub file_count: u64,
    pub folder_count: u64,
    pub total_size: u64,
    pub inaccessible_count: u64,
    pub cancelled: bool,
}

pub fn calculate_folder_info(
    path: &Path,
    cancellation: &CancellationToken,
) -> io::Result<FolderInfo> {
    let metadata = fs::metadata(path)?;
    let mut info = FolderInfo {
        path: path.to_path_buf(),
        created: metadata.created().ok(),
        modified: metadata.modified().ok(),
        readonly: metadata.permissions().readonly(),
        hidden: is_hidden(path, &metadata),
        file_count: 0,
        folder_count: 0,
        total_size: 0,
        inaccessible_count: 0,
        cancelled: false,
    };
    for entry in WalkDir::new(path).follow_links(false).into_iter().skip(1) {
        if cancellation.is_cancelled() {
            info.cancelled = true;
            break;
        }
        match entry {
            Ok(entry) if entry.file_type().is_dir() => info.folder_count += 1,
            Ok(entry) if entry.file_type().is_file() => {
                info.file_count += 1;
                if let Ok(metadata) = entry.metadata() {
                    info.total_size = info.total_size.saturating_add(metadata.len());
                }
            }
            Ok(_) => {}
            Err(_) => info.inaccessible_count += 1,
        }
    }
    Ok(info)
}

#[derive(Clone, Debug)]
pub struct TypeStatistic {
    pub extension: String,
    pub count: u64,
    pub total_size: u64,
}

#[derive(Clone, Debug)]
pub struct DuplicateCandidate {
    pub size: u64,
    pub paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Default)]
pub struct FolderStatistics {
    pub types: Vec<TypeStatistic>,
    pub largest: Vec<FileEntry>,
    pub recent: Vec<FileEntry>,
    pub old: Vec<FileEntry>,
    pub empty_folders: Vec<PathBuf>,
    pub duplicate_candidates: Vec<DuplicateCandidate>,
    pub scanned: u64,
    pub inaccessible: u64,
    pub cancelled: bool,
}

pub fn calculate_folder_statistics(
    path: &Path,
    cancellation: &CancellationToken,
) -> io::Result<FolderStatistics> {
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "statistics require a directory",
        ));
    }
    let mut statistics = FolderStatistics::default();
    let mut types = BTreeMap::<String, (u64, u64)>::new();
    let mut files = Vec::<FileEntry>::new();
    let mut files_by_size = HashMap::<u64, Vec<PathBuf>>::new();

    for entry in WalkDir::new(path).follow_links(false).into_iter().skip(1) {
        if cancellation.is_cancelled() {
            statistics.cancelled = true;
            break;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                statistics.inaccessible += 1;
                continue;
            }
        };
        statistics.scanned += 1;
        if entry.file_type().is_dir() {
            if fs::read_dir(entry.path())
                .map(|mut children| children.next().is_none())
                .unwrap_or(false)
            {
                statistics.empty_folders.push(entry.path().to_path_buf());
            }
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let file = match FileEntry::from_path(entry.path()) {
            Ok(file) => file,
            Err(_) => {
                statistics.inaccessible += 1;
                continue;
            }
        };
        let extension = file
            .path
            .extension()
            .and_then(|extension| extension.to_str())
            .filter(|extension| !extension.is_empty())
            .map(|extension| extension.to_lowercase())
            .unwrap_or_else(|| "(no extension)".to_string());
        let type_entry = types.entry(extension).or_default();
        type_entry.0 += 1;
        type_entry.1 = type_entry.1.saturating_add(file.size);
        if file.size > 0 {
            files_by_size
                .entry(file.size)
                .or_default()
                .push(file.path.clone());
        }
        files.push(file);
    }

    statistics.types = types
        .into_iter()
        .map(|(extension, (count, total_size))| TypeStatistic {
            extension,
            count,
            total_size,
        })
        .collect();
    statistics
        .types
        .sort_by_key(|statistic| Reverse(statistic.total_size));

    files.sort_by_key(|file| Reverse(file.size));
    statistics.largest = files.iter().take(20).cloned().collect();
    files.sort_by_key(|file| Reverse(file.modified));
    statistics.recent = files.iter().take(20).cloned().collect();
    files.sort_by_key(|file| file.modified);
    statistics.old = files.iter().take(20).cloned().collect();

    for (size, paths) in files_by_size
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
    {
        let mut by_fingerprint = HashMap::<u64, Vec<PathBuf>>::new();
        for path in paths {
            if cancellation.is_cancelled() {
                statistics.cancelled = true;
                break;
            }
            if let Ok(fingerprint) = quick_fingerprint(&path, size) {
                by_fingerprint.entry(fingerprint).or_default().push(path);
            }
        }
        statistics.duplicate_candidates.extend(
            by_fingerprint
                .into_values()
                .filter(|paths| paths.len() > 1)
                .map(|paths| DuplicateCandidate { size, paths }),
        );
    }
    statistics
        .duplicate_candidates
        .sort_by_key(|candidate| Reverse(candidate.size));
    statistics.empty_folders.truncate(50);
    Ok(statistics)
}

fn quick_fingerprint(path: &Path, size: u64) -> io::Result<u64> {
    let mut file = fs::File::open(path)?;
    let mut buffer = vec![0; 64 * 1024];
    let read = file.read(&mut buffer)?;
    let mut hasher = DefaultHasher::new();
    size.hash(&mut hasher);
    buffer[..read].hash(&mut hasher);
    Ok(hasher.finish())
}

pub fn format_system_time(time: Option<SystemTime>) -> String {
    time.map(relative_time_label)
        .unwrap_or_else(|| "Unknown".to_string())
}

fn relative_time_label(time: SystemTime) -> String {
    let elapsed = SystemTime::now()
        .duration_since(time)
        .unwrap_or(Duration::ZERO);
    let seconds = elapsed.as_secs();

    match seconds {
        0..60 => "Just now".to_string(),
        60..3_600 => format!("{} min ago", seconds / 60),
        3_600..86_400 => format!("{} hr ago", seconds / 3_600),
        86_400..2_592_000 => format!("{} days ago", seconds / 86_400),
        2_592_000..31_536_000 => format!("{} months ago", seconds / 2_592_000),
        _ => format!("{} years ago", seconds / 31_536_000),
    }
}

#[cfg(windows)]
fn is_hidden(path: &Path, metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0
        || path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

#[cfg(not(windows))]
fn is_hidden(path: &Path, _metadata: &fs::Metadata) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, kind: EntryKind) -> FileEntry {
        FileEntry {
            name: name.to_string(),
            path: PathBuf::from(name),
            kind,
            size: 0,
            modified: None,
            readonly: false,
            hidden: false,
        }
    }

    #[test]
    fn directories_are_sorted_before_files() {
        let mut entries = [
            entry("z.txt", EntryKind::File),
            entry("beta", EntryKind::Directory),
            entry("Alpha", EntryKind::Directory),
            entry("a.txt", EntryKind::File),
        ];

        entries.sort_by(|left, right| compare_entries(left, right, SortSpec::default()));

        let names = entries.map(|entry| entry.name);
        assert_eq!(names, ["Alpha", "beta", "a.txt", "z.txt"]);
    }

    #[test]
    fn directory_size_is_not_displayed() {
        assert_eq!(entry("src", EntryKind::Directory).size_label(), "");
    }

    #[test]
    fn recent_time_is_human_readable() {
        let label = relative_time_label(SystemTime::now() - Duration::from_secs(120));
        assert_eq!(label, "2 min ago");
    }

    #[test]
    fn fuzzy_match_preserves_character_order() {
        assert!(fuzzy_matches("ProductRequirements.md", "prdm"));
        assert!(!fuzzy_matches("ProductRequirements.md", "zpr"));
    }

    #[test]
    fn parses_search_filter_chips_from_text() {
        let options = SearchOptions::parse(
            "quarterly report ext:pdf glob:**/reports/* min:1.5mb hidden:true",
        );
        assert_eq!(options.query, "quarterly report");
        assert_eq!(options.extension.as_deref(), Some("pdf"));
        assert_eq!(options.glob.as_deref(), Some("**/reports/*"));
        assert_eq!(options.minimum_size, Some(1_572_864));
        assert!(options.include_hidden);
    }
}
