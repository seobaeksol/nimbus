use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use filetime::{FileTime, set_file_times};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use zip::{ZipWriter, write::SimpleFileOptions};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    Overwrite,
    Skip,
    #[default]
    Rename,
}

impl ConflictResolution {
    pub fn label(self) -> &'static str {
        match self {
            Self::Overwrite => "Overwrite",
            Self::Skip => "Skip",
            Self::Rename => "Keep both",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationKind {
    Copy,
    Move,
    DeleteToTrash,
    Rename,
    CreateFolder,
    Compress,
}

impl OperationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Copy => "Copy",
            Self::Move => "Move",
            Self::DeleteToTrash => "Move to Recycle Bin",
            Self::Rename => "Rename",
            Self::CreateFolder => "Create folder",
            Self::Compress => "Compress",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlannedItem {
    pub source: Option<PathBuf>,
    pub destination: Option<PathBuf>,
    pub conflict: bool,
    pub skipped: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationPlan {
    pub kind: OperationKind,
    pub resolution: ConflictResolution,
    pub items: Vec<PlannedItem>,
    pub requested_destinations: Vec<Option<PathBuf>>,
}

impl OperationPlan {
    pub fn transfer(
        kind: OperationKind,
        sources: Vec<PathBuf>,
        destination_directory: &Path,
        resolution: ConflictResolution,
    ) -> io::Result<Self> {
        if !matches!(kind, OperationKind::Copy | OperationKind::Move) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "transfer plan must be copy or move",
            ));
        }
        if sources.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "no source items were selected",
            ));
        }
        if !destination_directory.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "destination folder does not exist: {}",
                    destination_directory.display()
                ),
            ));
        }

        let mut reserved = Vec::<PathBuf>::new();
        let mut items = Vec::with_capacity(sources.len());
        let mut requested_destinations = Vec::with_capacity(sources.len());
        for source in sources {
            if !source.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("source does not exist: {}", source.display()),
                ));
            }
            let Some(name) = source.file_name() else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("cannot transfer the path root: {}", source.display()),
                ));
            };
            let requested = destination_directory.join(name);
            requested_destinations.push(Some(requested.clone()));
            if requested == source {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "source and destination are the same",
                ));
            }
            let conflict = requested.exists() || reserved.iter().any(|path| path == &requested);
            let (destination, skipped) = match (conflict, resolution) {
                (true, ConflictResolution::Skip) => (requested, true),
                (true, ConflictResolution::Rename) => {
                    (unique_destination(&requested, &reserved), false)
                }
                _ => (requested, false),
            };
            reserved.push(destination.clone());
            items.push(PlannedItem {
                source: Some(source),
                destination: Some(destination),
                conflict,
                skipped,
            });
        }
        Ok(Self {
            kind,
            resolution,
            items,
            requested_destinations,
        })
    }

    pub fn delete(sources: Vec<PathBuf>) -> io::Result<Self> {
        if sources.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "no items were selected",
            ));
        }
        Ok(Self {
            kind: OperationKind::DeleteToTrash,
            resolution: ConflictResolution::Skip,
            requested_destinations: vec![None; sources.len()],
            items: sources
                .into_iter()
                .map(|source| PlannedItem {
                    source: Some(source),
                    destination: None,
                    conflict: false,
                    skipped: false,
                })
                .collect(),
        })
    }

    pub fn rename(
        source: PathBuf,
        new_name: &str,
        resolution: ConflictResolution,
    ) -> io::Result<Self> {
        validate_file_name(new_name)?;
        let parent = source.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "cannot rename a path root")
        })?;
        let requested = parent.join(new_name);
        Self::rename_destinations(vec![(source, requested)], resolution)
    }

    pub fn batch_rename_prefix(
        sources: Vec<PathBuf>,
        prefix: &str,
        resolution: ConflictResolution,
    ) -> io::Result<Self> {
        if sources.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "select at least two items for batch rename",
            ));
        }
        if prefix.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "batch rename prefix cannot be empty",
            ));
        }
        let mut renames = Vec::with_capacity(sources.len());
        for source in sources {
            let name = source
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "item has no valid file name")
                })?;
            let new_name = format!("{prefix}{name}");
            validate_file_name(&new_name)?;
            let requested = source
                .parent()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "cannot rename a path root")
                })?
                .join(new_name);
            renames.push((source, requested));
        }
        Self::rename_destinations(renames, resolution)
    }

    fn rename_destinations(
        renames: Vec<(PathBuf, PathBuf)>,
        resolution: ConflictResolution,
    ) -> io::Result<Self> {
        let mut items = Vec::with_capacity(renames.len());
        let mut requested_destinations = Vec::with_capacity(renames.len());
        let mut reserved = Vec::<PathBuf>::new();
        for (source, requested) in renames {
            if requested == source {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "the new name is unchanged",
                ));
            }
            let conflict =
                requested.exists() || reserved.iter().any(|destination| destination == &requested);
            let (destination, skipped) = match (conflict, resolution) {
                (true, ConflictResolution::Skip) => (requested.clone(), true),
                (true, ConflictResolution::Rename) => {
                    (unique_destination(&requested, &reserved), false)
                }
                _ => (requested.clone(), false),
            };
            reserved.push(destination.clone());
            requested_destinations.push(Some(requested));
            items.push(PlannedItem {
                source: Some(source),
                destination: Some(destination),
                conflict,
                skipped,
            });
        }
        Ok(Self {
            kind: OperationKind::Rename,
            resolution,
            requested_destinations,
            items,
        })
    }

    pub fn create_folder(parent: &Path, name: &str) -> io::Result<Self> {
        validate_file_name(name)?;
        let destination = parent.join(name);
        Ok(Self {
            kind: OperationKind::CreateFolder,
            resolution: ConflictResolution::Skip,
            requested_destinations: vec![Some(destination.clone())],
            items: vec![PlannedItem {
                source: None,
                conflict: destination.exists(),
                skipped: destination.exists(),
                destination: Some(destination),
            }],
        })
    }

    pub fn compress(
        sources: Vec<PathBuf>,
        archive: PathBuf,
        resolution: ConflictResolution,
    ) -> io::Result<Self> {
        if sources.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "no items were selected",
            ));
        }
        let conflict = archive.exists();
        let (destination, skipped) = match (conflict, resolution) {
            (true, ConflictResolution::Skip) => (archive.clone(), true),
            (true, ConflictResolution::Rename) => (unique_destination(&archive, &[]), false),
            _ => (archive.clone(), false),
        };
        Ok(Self {
            kind: OperationKind::Compress,
            resolution,
            requested_destinations: vec![Some(archive)],
            items: vec![PlannedItem {
                source: None,
                destination: Some(destination),
                conflict,
                skipped,
            }],
        }
        .with_archive_sources(sources))
    }

    fn with_archive_sources(mut self, sources: Vec<PathBuf>) -> Self {
        let destination = self.items[0].destination.clone();
        let requested = self.requested_destinations[0].clone();
        let conflict = self.items[0].conflict;
        let skipped = self.items[0].skipped;
        self.items = sources
            .into_iter()
            .map(|source| PlannedItem {
                source: Some(source),
                destination: destination.clone(),
                conflict,
                skipped,
            })
            .collect();
        self.requested_destinations = vec![requested; self.items.len()];
        self
    }

    pub fn conflict_count(&self) -> usize {
        self.items.iter().filter(|item| item.conflict).count()
    }

    pub fn executable_count(&self) -> usize {
        self.items.iter().filter(|item| !item.skipped).count()
    }

    pub fn with_resolution(&self, resolution: ConflictResolution) -> io::Result<Self> {
        match self.kind {
            OperationKind::Copy | OperationKind::Move => {
                let sources = self
                    .items
                    .iter()
                    .filter_map(|item| item.source.clone())
                    .collect();
                let destination = self
                    .items
                    .first()
                    .and_then(|item| item.destination.as_deref())
                    .and_then(Path::parent)
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "missing destination")
                    })?;
                Self::transfer(self.kind, sources, destination, resolution)
            }
            OperationKind::Rename => Self::rename_destinations(
                self.items
                    .iter()
                    .zip(self.requested_destinations.iter())
                    .filter_map(|(item, requested)| {
                        Some((item.source.clone()?, requested.clone()?))
                    })
                    .collect(),
                resolution,
            ),
            OperationKind::Compress => {
                let sources = self
                    .items
                    .iter()
                    .filter_map(|item| item.source.clone())
                    .collect();
                Self::compress(
                    sources,
                    self.requested_destinations
                        .first()
                        .cloned()
                        .flatten()
                        .or_else(|| self.items[0].destination.clone())
                        .unwrap(),
                    resolution,
                )
            }
            _ => Ok(self.clone()),
        }
    }

    pub fn execute(
        &self,
        control: &OperationControl,
        undo_root: &Path,
    ) -> io::Result<OperationOutcome> {
        fs::create_dir_all(undo_root)?;
        let mut outcome = OperationOutcome::default();

        if self.kind == OperationKind::Compress {
            if self.items.first().is_some_and(|item| item.skipped) {
                outcome.skipped = self.items.len();
                return Ok(outcome);
            }
            control.checkpoint()?;
            let destination = self.items[0].destination.as_ref().unwrap();
            if destination.exists() && self.resolution == ConflictResolution::Overwrite {
                let backup = backup_existing(destination, undo_root)?;
                outcome.undo.push(UndoAction::RestoreBackup {
                    backup,
                    original: destination.clone(),
                });
            }
            let sources = self
                .items
                .iter()
                .filter_map(|item| item.source.as_deref())
                .collect::<Vec<_>>();
            create_zip_archive(&sources, destination, control)?;
            outcome.undo.push(UndoAction::RemoveCreated {
                path: destination.clone(),
            });
            outcome.completed = self.items.len();
            return Ok(outcome);
        }

        for item in &self.items {
            control.checkpoint()?;
            if item.skipped {
                outcome.skipped += 1;
                continue;
            }
            let result = match self.kind {
                OperationKind::Copy => {
                    let source = item.source.as_ref().unwrap();
                    let destination = item.destination.as_ref().unwrap();
                    backup_for_overwrite(self, item, undo_root, &mut outcome)?;
                    copy_path(source, destination, control)?;
                    outcome.undo.push(UndoAction::RemoveCreated {
                        path: destination.clone(),
                    });
                    Ok(())
                }
                OperationKind::Move | OperationKind::Rename => {
                    let source = item.source.as_ref().unwrap();
                    let destination = item.destination.as_ref().unwrap();
                    backup_for_overwrite(self, item, undo_root, &mut outcome)?;
                    move_path(source, destination, control)?;
                    outcome.undo.push(UndoAction::Move {
                        from: destination.clone(),
                        to: source.clone(),
                    });
                    Ok(())
                }
                OperationKind::DeleteToTrash => {
                    let source = item.source.as_ref().unwrap();
                    trash::delete(source).map_err(io::Error::other)
                }
                OperationKind::CreateFolder => {
                    let destination = item.destination.as_ref().unwrap();
                    fs::create_dir(destination)?;
                    outcome.undo.push(UndoAction::RemoveCreated {
                        path: destination.clone(),
                    });
                    Ok(())
                }
                OperationKind::Compress => unreachable!(),
            };
            match result {
                Ok(()) => outcome.completed += 1,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                    outcome.cancelled = true;
                    return Ok(outcome);
                }
                Err(error) => {
                    outcome.failed += 1;
                    outcome.errors.push(error.to_string());
                }
            }
        }
        Ok(outcome)
    }
}

fn backup_for_overwrite(
    plan: &OperationPlan,
    item: &PlannedItem,
    undo_root: &Path,
    outcome: &mut OperationOutcome,
) -> io::Result<()> {
    if item.conflict && plan.resolution == ConflictResolution::Overwrite {
        let destination = item.destination.as_ref().unwrap();
        let backup = backup_existing(destination, undo_root)?;
        outcome.undo.push(UndoAction::RestoreBackup {
            backup,
            original: destination.clone(),
        });
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub enum UndoAction {
    RemoveCreated { path: PathBuf },
    Move { from: PathBuf, to: PathBuf },
    RestoreBackup { backup: PathBuf, original: PathBuf },
}

#[derive(Clone, Debug, Default)]
pub struct OperationOutcome {
    pub completed: usize,
    pub skipped: usize,
    pub failed: usize,
    pub cancelled: bool,
    pub errors: Vec<String>,
    pub undo: Vec<UndoAction>,
}

impl OperationOutcome {
    pub fn summary(&self) -> String {
        if self.cancelled {
            format!(
                "Cancelled — {} completed, {} skipped, {} failed",
                self.completed, self.skipped, self.failed
            )
        } else {
            format!(
                "{} completed, {} skipped, {} failed",
                self.completed, self.skipped, self.failed
            )
        }
    }

    pub fn undo(self) -> io::Result<()> {
        for action in self.undo.into_iter().rev() {
            match action {
                UndoAction::RemoveCreated { path } => remove_path(&path)?,
                UndoAction::Move { from, to } => {
                    if let Some(parent) = to.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::rename(from, to)?;
                }
                UndoAction::RestoreBackup { backup, original } => {
                    if original.exists() {
                        remove_path(&original)?;
                    }
                    if let Some(parent) = original.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::rename(backup, original)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct OperationControl {
    cancelled: Arc<AtomicBool>,
    pause: Arc<(Mutex<bool>, Condvar)>,
}

impl OperationControl {
    pub fn pause(&self) {
        *self.pause.0.lock().expect("pause mutex poisoned") = true;
    }

    pub fn resume(&self) {
        let mut paused = self.pause.0.lock().expect("pause mutex poisoned");
        *paused = false;
        self.pause.1.notify_all();
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        self.resume();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    fn checkpoint(&self) -> io::Result<()> {
        if self.is_cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "operation cancelled",
            ));
        }
        let (mutex, condition) = &*self.pause;
        let mut paused = mutex.lock().expect("pause mutex poisoned");
        while *paused && !self.is_cancelled() {
            paused = condition.wait(paused).expect("pause mutex poisoned");
        }
        if self.is_cancelled() {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "operation cancelled",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct OperationJob {
    pub id: u64,
    pub plan: OperationPlan,
    pub status: JobStatus,
    pub summary: String,
    pub outcome: Option<OperationOutcome>,
}

#[derive(Default)]
pub struct OperationQueue {
    pub jobs: Vec<OperationJob>,
    pub active: Option<u64>,
    next_id: u64,
}

impl OperationQueue {
    pub fn enqueue(&mut self, plan: OperationPlan) -> u64 {
        self.next_id = self.next_id.max(1);
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.push(OperationJob {
            id,
            summary: format!(
                "{} — {} item(s), {} conflict(s)",
                plan.kind.label(),
                plan.items.len(),
                plan.conflict_count()
            ),
            plan,
            status: JobStatus::Queued,
            outcome: None,
        });
        id
    }

    pub fn next_queued(&mut self) -> Option<(u64, OperationPlan)> {
        if self.active.is_some() {
            return None;
        }
        let job = self
            .jobs
            .iter_mut()
            .find(|job| job.status == JobStatus::Queued)?;
        job.status = JobStatus::Running;
        self.active = Some(job.id);
        Some((job.id, job.plan.clone()))
    }

    pub fn complete(&mut self, id: u64, result: io::Result<OperationOutcome>) {
        let Some(job) = self.jobs.iter_mut().find(|job| job.id == id) else {
            return;
        };
        match result {
            Ok(outcome) => {
                job.status = if outcome.cancelled {
                    JobStatus::Cancelled
                } else if outcome.failed > 0 {
                    JobStatus::Failed
                } else {
                    JobStatus::Completed
                };
                job.summary = outcome.summary();
                job.outcome = Some(outcome);
            }
            Err(error) => {
                job.status = JobStatus::Failed;
                job.summary = error.to_string();
            }
        }
        self.active = None;
    }

    pub fn latest_undoable(&mut self) -> Option<&mut OperationJob> {
        self.jobs.iter_mut().rev().find(|job| {
            job.status == JobStatus::Completed
                && job
                    .outcome
                    .as_ref()
                    .is_some_and(|outcome| !outcome.undo.is_empty())
        })
    }
}

fn validate_file_name(name: &str) -> io::Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty()
        || trimmed == "."
        || trimmed == ".."
        || trimmed.ends_with(['.', ' '])
        || trimmed.chars().any(|character| {
            matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\0'
            )
        })
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the name contains characters Windows does not allow",
        ));
    }
    let stem = trimmed
        .split('.')
        .next()
        .unwrap_or(trimmed)
        .to_ascii_uppercase();
    if matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    ) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the name is reserved by Windows",
        ));
    }
    Ok(())
}

fn unique_destination(requested: &Path, reserved: &[PathBuf]) -> PathBuf {
    if !requested.exists() && reserved.iter().all(|path| path != requested) {
        return requested.to_path_buf();
    }
    let parent = requested.parent().unwrap_or_else(|| Path::new("."));
    let stem = requested
        .file_stem()
        .or_else(|| requested.file_name())
        .unwrap_or_default()
        .to_string_lossy();
    let extension = requested
        .extension()
        .map(|extension| extension.to_string_lossy().into_owned());
    for index in 2.. {
        let name = match &extension {
            Some(extension) => format!("{stem} ({index}).{extension}"),
            None => format!("{stem} ({index})"),
        };
        let candidate = parent.join(name);
        if !candidate.exists() && reserved.iter().all(|path| path != &candidate) {
            return candidate;
        }
    }
    unreachable!()
}

fn backup_existing(path: &Path, undo_root: &Path) -> io::Result<PathBuf> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    let backup = undo_root.join(format!("{stamp}-{name}"));
    move_path(path, &backup, &OperationControl::default())?;
    Ok(backup)
}

fn copy_path(source: &Path, destination: &Path, control: &OperationControl) -> io::Result<()> {
    control.checkpoint()?;
    if source.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in WalkDir::new(source).follow_links(false).into_iter().skip(1) {
            control.checkpoint()?;
            let entry = entry.map_err(io::Error::other)?;
            let relative = entry
                .path()
                .strip_prefix(source)
                .map_err(io::Error::other)?;
            let target = destination.join(relative);
            if entry.file_type().is_dir() {
                fs::create_dir_all(&target)?;
            } else if entry.file_type().is_file() {
                copy_file(entry.path(), &target, control)?;
            }
        }
        Ok(())
    } else {
        copy_file(source, destination, control)
    }
}

fn copy_file(source: &Path, destination: &Path, control: &OperationControl) -> io::Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut reader = fs::File::open(source)?;
    let mut writer = fs::File::create(destination)?;
    let mut buffer = vec![0; 1024 * 1024];
    loop {
        control.checkpoint()?;
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        writer.write_all(&buffer[..read])?;
    }
    writer.flush()?;
    if let Ok(metadata) = fs::metadata(source) {
        let _ = fs::set_permissions(destination, metadata.permissions());
        let _ = set_file_times(
            destination,
            FileTime::from_last_access_time(&metadata),
            FileTime::from_last_modification_time(&metadata),
        );
    }
    Ok(())
}

fn move_path(source: &Path, destination: &Path, control: &OperationControl) -> io::Result<()> {
    control.checkpoint()?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(_) => {
            copy_path(source, destination, control)?;
            remove_path(source)
        }
    }
}

fn remove_path(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn create_zip_archive(
    sources: &[&Path],
    destination: &Path,
    control: &OperationControl,
) -> io::Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let output = fs::File::create(destination)?;
    let mut zip = ZipWriter::new(output);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    let directory_options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    for source in sources {
        control.checkpoint()?;
        let base = source.parent().unwrap_or_else(|| Path::new(""));
        if source.is_dir() {
            for entry in WalkDir::new(source).follow_links(false) {
                control.checkpoint()?;
                let entry = entry.map_err(io::Error::other)?;
                let relative = entry.path().strip_prefix(base).map_err(io::Error::other)?;
                let archive_name = relative.to_string_lossy().replace('\\', "/");
                if entry.file_type().is_dir() {
                    zip.add_directory(format!("{archive_name}/"), directory_options)?;
                } else if entry.file_type().is_file() {
                    zip.start_file(archive_name, options)?;
                    let mut file = fs::File::open(entry.path())?;
                    let mut buffer = vec![0; 1024 * 1024];
                    loop {
                        control.checkpoint()?;
                        let read = file.read(&mut buffer)?;
                        if read == 0 {
                            break;
                        }
                        zip.write_all(&buffer[..read])?;
                    }
                }
            }
        } else {
            let name = source
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            zip.start_file(name, options)?;
            let mut file = fs::File::open(source)?;
            io::copy(&mut file, &mut zip)?;
        }
    }
    zip.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("nimbus-operation-{name}-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn collision_rename_keeps_both_files() {
        let root = temp_dir("rename");
        let source_dir = root.join("source");
        let destination_dir = root.join("destination");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&destination_dir).unwrap();
        let source = source_dir.join("report.txt");
        fs::write(&source, b"new").unwrap();
        fs::write(destination_dir.join("report.txt"), b"old").unwrap();

        let plan = OperationPlan::transfer(
            OperationKind::Copy,
            vec![source],
            &destination_dir,
            ConflictResolution::Rename,
        )
        .unwrap();
        assert!(plan.items[0].conflict);
        assert_eq!(
            plan.items[0].destination.as_ref().unwrap(),
            &destination_dir.join("report (2).txt")
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn copy_can_be_undone() {
        let root = temp_dir("undo");
        let source = root.join("source.txt");
        let destination_dir = root.join("out");
        fs::write(&source, b"Nimbus").unwrap();
        fs::create_dir_all(&destination_dir).unwrap();
        let plan = OperationPlan::transfer(
            OperationKind::Copy,
            vec![source],
            &destination_dir,
            ConflictResolution::Rename,
        )
        .unwrap();
        let outcome = plan
            .execute(&OperationControl::default(), &root.join("undo"))
            .unwrap();
        let copied = destination_dir.join("source.txt");
        assert_eq!(fs::read(&copied).unwrap(), b"Nimbus");
        outcome.undo().unwrap();
        assert!(!copied.exists());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn changing_rename_resolution_targets_the_original_conflict() {
        let root = temp_dir("rename-resolution");
        let source = root.join("draft.txt");
        let requested = root.join("final.txt");
        fs::write(&source, b"draft").unwrap();
        fs::write(&requested, b"existing").unwrap();

        let keep_both =
            OperationPlan::rename(source, "final.txt", ConflictResolution::Rename).unwrap();
        assert_eq!(
            keep_both.items[0].destination.as_deref(),
            Some(root.join("final (2).txt").as_path())
        );

        let overwrite = keep_both
            .with_resolution(ConflictResolution::Overwrite)
            .unwrap();
        assert!(overwrite.items[0].conflict);
        assert_eq!(
            overwrite.items[0].destination.as_deref(),
            Some(requested.as_path())
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn batch_rename_previews_each_destination() {
        let root = temp_dir("batch-rename");
        let first = root.join("a.txt");
        let second = root.join("b.txt");
        fs::write(&first, b"a").unwrap();
        fs::write(&second, b"b").unwrap();

        let plan = OperationPlan::batch_rename_prefix(
            vec![first, second],
            "2026-",
            ConflictResolution::Rename,
        )
        .unwrap();
        let names = plan
            .items
            .iter()
            .map(|item| {
                item.destination
                    .as_deref()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();
        assert_eq!(names, ["2026-a.txt", "2026-b.txt"]);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_windows_names_are_rejected() {
        assert!(validate_file_name("valid folder").is_ok());
        assert!(validate_file_name("CON.txt").is_err());
        assert!(validate_file_name("bad?.txt").is_err());
    }
}
