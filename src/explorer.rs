use std::{
    collections::{HashMap, HashSet},
    env,
    path::{Path, PathBuf},
};

use gpui::{
    AnyElement, AppContext, ClickEvent, ClipboardItem, Context, Entity, FocusHandle, IntoElement,
    KeyDownEvent, ObjectFit, Render, ScrollStrategy, StyledImage, Subscription,
    UniformListScrollHandle, Window, div, img, prelude::*, px, uniform_list,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Sizable, StyledExt, TitleBar, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    resizable::{ResizablePanelEvent, ResizableState, h_resizable, resizable_panel, v_resizable},
    scroll::ScrollableElement,
    v_flex,
};
use humansize::{BINARY, format_size};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::{
    filesystem::{
        CancellationToken, FileEntry, FolderInfo, FolderStatistics, SearchOptions, SearchResult,
        calculate_folder_info, calculate_folder_statistics, format_system_time,
        read_directory_sorted, search_paths,
    },
    git::{GitRepositoryInfo, commit_files, repository_info},
    model::{
        AppState, LayoutPreset, PanelId, SavedSearch, SidebarTool, SplitAxis, SplitId, SplitNode,
        WorkspaceState,
    },
    operation::{
        ConflictResolution, JobStatus, OperationControl, OperationKind, OperationPlan,
        OperationQueue,
    },
    persistence::StateStore,
    preview::{PreviewContent, QuickPreview},
    windows_integration::{
        PathFormat, format_path, known_folders, open_terminal, reveal_in_file_explorer,
    },
};

#[derive(Default)]
struct PanelRuntime {
    entries: Vec<FileEntry>,
    loading: bool,
    error: Option<String>,
    request_id: u64,
    selected: HashSet<PathBuf>,
    selection_anchor: Option<usize>,
    scroll: UniformListScrollHandle,
}

#[derive(Default)]
enum ContextState<T> {
    #[default]
    Idle,
    Loading(PathBuf),
    Ready(T),
    Error(PathBuf, String),
}

#[derive(Clone)]
enum NameAction {
    Rename(PathBuf),
    BatchRename(Vec<PathBuf>),
    CreateFolder(PathBuf),
    SaveWorkspace,
    NewShelf,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DialogState {
    None,
    Palette,
    Name,
    Operation,
}

enum PreviewState {
    Loading(PathBuf),
    Ready(QuickPreview),
    Error(PathBuf, String),
}

#[derive(Clone)]
struct DraggedFiles {
    paths: Vec<PathBuf>,
    source_panel: PanelId,
}

impl Render for DraggedFiles {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(gpui::black().opacity(0.75))
            .text_color(gpui::white())
            .shadow_lg()
            .child(Icon::new(IconName::Copy).small())
            .child(format!("{} item(s)", self.paths.len()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CommandId {
    CopyWindowsPath,
    CopyPowerShellPath,
    CopyFileUri,
    CopyWslPath,
    OpenTerminal,
    Reveal,
    Refresh,
    NewFolder,
    NewTab,
    SplitColumns,
    SplitRows,
    GridLayout,
    AddToShelf,
    ShowPreview,
    CopyToTarget,
    MoveToTarget,
    Rename,
    BatchRename,
    Delete,
    FolderInfo,
    Statistics,
    Search,
    ToggleSidebar,
    Undo,
}

struct CommandSpec {
    id: CommandId,
    name: &'static str,
    description: &'static str,
    shortcut: &'static str,
}

const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        id: CommandId::CopyWindowsPath,
        name: "Copy path: Windows",
        description: "Copy selected paths using Windows separators",
        shortcut: "Ctrl+Shift+C",
    },
    CommandSpec {
        id: CommandId::CopyPowerShellPath,
        name: "Copy path: PowerShell",
        description: "Copy safely quoted PowerShell paths",
        shortcut: "",
    },
    CommandSpec {
        id: CommandId::CopyFileUri,
        name: "Copy path: File URI",
        description: "Copy selected paths as file:// URIs",
        shortcut: "",
    },
    CommandSpec {
        id: CommandId::CopyWslPath,
        name: "Copy path: WSL",
        description: "Convert drive paths to /mnt/<drive>",
        shortcut: "",
    },
    CommandSpec {
        id: CommandId::OpenTerminal,
        name: "Open terminal here",
        description: "Start PowerShell in the active folder",
        shortcut: "Ctrl+`",
    },
    CommandSpec {
        id: CommandId::Reveal,
        name: "Show in File Explorer",
        description: "Reveal the selection in Windows File Explorer",
        shortcut: "",
    },
    CommandSpec {
        id: CommandId::Refresh,
        name: "Refresh active panel",
        description: "Reload the active folder",
        shortcut: "F5",
    },
    CommandSpec {
        id: CommandId::NewFolder,
        name: "Create folder",
        description: "Create a folder in the active panel",
        shortcut: "Ctrl+Shift+N",
    },
    CommandSpec {
        id: CommandId::NewTab,
        name: "New tab",
        description: "Open the active location in another tab",
        shortcut: "Ctrl+T",
    },
    CommandSpec {
        id: CommandId::SplitColumns,
        name: "Split panel left / right",
        description: "Split the active panel into columns",
        shortcut: "Ctrl+Shift+V",
    },
    CommandSpec {
        id: CommandId::SplitRows,
        name: "Split panel top / bottom",
        description: "Split the active panel into rows",
        shortcut: "Ctrl+Shift+H",
    },
    CommandSpec {
        id: CommandId::GridLayout,
        name: "Use 2×2 panel layout",
        description: "Open the four-panel layout preset",
        shortcut: "",
    },
    CommandSpec {
        id: CommandId::AddToShelf,
        name: "Add selection to Shelf",
        description: "Collect references without moving files",
        shortcut: "Ctrl+Shift+S",
    },
    CommandSpec {
        id: CommandId::ShowPreview,
        name: "Quick Look",
        description: "Preview the selected file without opening its application",
        shortcut: "Space",
    },
    CommandSpec {
        id: CommandId::CopyToTarget,
        name: "Copy selection to target panel",
        description: "Preflight a copy into the designated target",
        shortcut: "",
    },
    CommandSpec {
        id: CommandId::MoveToTarget,
        name: "Move selection to target panel",
        description: "Preflight a move into the designated target",
        shortcut: "",
    },
    CommandSpec {
        id: CommandId::Rename,
        name: "Rename",
        description: "Rename the selected item with collision preview",
        shortcut: "F2",
    },
    CommandSpec {
        id: CommandId::BatchRename,
        name: "Batch rename: add prefix",
        description: "Preview a prefix applied to every selected file name",
        shortcut: "",
    },
    CommandSpec {
        id: CommandId::Delete,
        name: "Move to Recycle Bin",
        description: "Safely delete selected items using the Windows Recycle Bin",
        shortcut: "Delete",
    },
    CommandSpec {
        id: CommandId::FolderInfo,
        name: "Show folder info",
        description: "Calculate size, counts, attributes, and access errors",
        shortcut: "",
    },
    CommandSpec {
        id: CommandId::Statistics,
        name: "Show folder statistics",
        description: "Analyze file types, large files, empty folders, and duplicates",
        shortcut: "",
    },
    CommandSpec {
        id: CommandId::Search,
        name: "Search all open panels",
        description: "Run a cancellable recursive search across open locations",
        shortcut: "Ctrl+F",
    },
    CommandSpec {
        id: CommandId::ToggleSidebar,
        name: "Toggle Sidebar",
        description: "Show or hide the contextual Sidebar",
        shortcut: "Ctrl+B",
    },
    CommandSpec {
        id: CommandId::Undo,
        name: "Undo last file operation",
        description: "Reverse the newest operation that Nimbus can safely undo",
        shortcut: "Ctrl+Z",
    },
];

pub struct Explorer {
    state: AppState,
    store: StateStore,
    panels: HashMap<PanelId, PanelRuntime>,
    focus_handle: FocusHandle,
    address_input: Entity<InputState>,
    filter_input: Entity<InputState>,
    palette_input: Entity<InputState>,
    name_input: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
    split_states: HashMap<SplitId, Entity<ResizableState>>,
    sidebar_resize: Entity<ResizableState>,
    resize_subscriptions: Vec<Subscription>,
    watcher: Option<RecommendedWatcher>,
    watched_paths: HashSet<PathBuf>,
    message: Option<String>,
    dialog: DialogState,
    name_action: Option<NameAction>,
    pending_plan: Option<OperationPlan>,
    preview: Option<PreviewState>,
    preview_visible: bool,
    preview_pinned: bool,
    queue: OperationQueue,
    operation_control: Option<OperationControl>,
    operation_paused: bool,
    clipboard_paths: Vec<PathBuf>,
    clipboard_cut: bool,
    git_state: ContextState<Option<GitRepositoryInfo>>,
    info_state: ContextState<FolderInfo>,
    statistics_state: ContextState<FolderStatistics>,
    context_cancellation: Option<CancellationToken>,
    search_results: ContextState<Vec<SearchResult>>,
    search_cancellation: Option<CancellationToken>,
}

impl Explorer {
    pub fn new(
        requested_workspace: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let initial_directory = home_directory();
        let store = StateStore::for_current_user();
        let (state, warning) = store.load(initial_directory, requested_workspace.as_deref());
        let active_path = state
            .workspace()
            .active_panel()
            .active_tab()
            .path
            .display()
            .to_string();
        let address_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(active_path)
                .placeholder("Enter a folder path")
        });
        let filter_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Filter this folder"));
        let palette_input = cx.new(|cx| InputState::new(window, cx).placeholder("Type a command…"));
        let name_input = cx.new(|cx| InputState::new(window, cx).placeholder("Name"));
        let sidebar_resize = cx.new(|_| ResizableState::default());
        let focus_handle = cx.focus_handle();

        let mut this = Self {
            state,
            store,
            panels: HashMap::new(),
            focus_handle,
            address_input: address_input.clone(),
            filter_input: filter_input.clone(),
            palette_input: palette_input.clone(),
            name_input: name_input.clone(),
            _subscriptions: Vec::new(),
            split_states: HashMap::new(),
            sidebar_resize: sidebar_resize.clone(),
            resize_subscriptions: Vec::new(),
            watcher: None,
            watched_paths: HashSet::new(),
            message: warning,
            dialog: DialogState::None,
            name_action: None,
            pending_plan: None,
            preview: None,
            preview_visible: false,
            preview_pinned: false,
            queue: OperationQueue::default(),
            operation_control: None,
            operation_paused: false,
            clipboard_paths: Vec::new(),
            clipboard_cut: false,
            git_state: ContextState::Idle,
            info_state: ContextState::Idle,
            statistics_state: ContextState::Idle,
            context_cancellation: None,
            search_results: ContextState::Idle,
            search_cancellation: None,
        };

        this._subscriptions = vec![
            cx.subscribe(&address_input, |this, input, event, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    let path = PathBuf::from(input.read(cx).value().trim());
                    this.navigate_active(path, cx);
                }
            }),
            cx.subscribe(&filter_input, |this, input, event, cx| {
                if matches!(event, InputEvent::Change) {
                    let filter = input.read(cx).value().to_string();
                    this.state
                        .workspace_mut()
                        .active_panel_mut()
                        .active_tab_mut()
                        .filter = filter;
                    let id = this.state.workspace().active_panel;
                    this.schedule_panel_load(id, cx);
                    this.persist();
                } else if matches!(event, InputEvent::PressEnter { .. }) {
                    this.show_sidebar_tool(SidebarTool::Search, cx);
                    this.run_search(cx);
                }
            }),
            cx.subscribe(&palette_input, |this, _, event, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                } else if matches!(event, InputEvent::PressEnter { .. }) {
                    this.execute_first_palette_command(None, cx);
                }
            }),
            cx.subscribe(&name_input, |this, input, event, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    let value = input.read(cx).value().to_string();
                    this.apply_name_action(&value, cx);
                }
            }),
            cx.subscribe(
                &sidebar_resize,
                |this, state, _: &ResizablePanelEvent, cx| {
                    if let Some(width) = state.read(cx).sizes().first() {
                        this.state.workspace_mut().sidebar.width =
                            width.as_f32().clamp(220.0, 460.0);
                        this.persist();
                    }
                },
            ),
        ];

        let panel_ids = this
            .state
            .workspace()
            .panels
            .iter()
            .map(|panel| panel.id)
            .collect::<Vec<_>>();
        for id in panel_ids {
            this.panels.insert(id, PanelRuntime::default());
            this.schedule_panel_load(id, cx);
        }
        this.ensure_split_states(cx);
        this.install_watcher(cx);
        this.focus_handle.focus(window, cx);
        this
    }

    fn workspace(&self) -> &WorkspaceState {
        self.state.workspace()
    }

    fn active_path(&self) -> &Path {
        &self.workspace().active_panel().active_tab().path
    }

    fn active_panel_id(&self) -> PanelId {
        self.workspace().active_panel
    }

    fn selected_paths(&self) -> Vec<PathBuf> {
        self.panels
            .get(&self.active_panel_id())
            .map(|runtime| runtime.selected.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn selected_or_folder(&self) -> Vec<PathBuf> {
        let selected = self.selected_paths();
        if selected.is_empty() {
            vec![self.active_path().to_path_buf()]
        } else {
            selected
        }
    }

    fn persist(&mut self) {
        if let Err(error) = self.store.save(&self.state) {
            self.message = Some(format!("Could not save workspace state: {error}"));
        }
    }

    fn ensure_split_states(&mut self, cx: &mut Context<Self>) {
        let mut ids = Vec::new();
        self.workspace().layout.split_ids(&mut ids);
        let valid = ids.iter().copied().collect::<HashSet<_>>();
        self.split_states.retain(|id, _| valid.contains(id));
        for id in ids {
            if self.split_states.contains_key(&id) {
                continue;
            }
            let state = cx.new(|_| ResizableState::default());
            self.resize_subscriptions.push(cx.subscribe(
                &state,
                move |this, state, _: &ResizablePanelEvent, cx| {
                    let sizes = state.read(cx).sizes();
                    if sizes.len() >= 2 {
                        let total = sizes[0].as_f32() + sizes[1].as_f32();
                        if total > 0.0 {
                            this.state
                                .workspace_mut()
                                .layout
                                .set_ratio(id, sizes[0].as_f32() / total);
                            this.persist();
                        }
                    }
                },
            ));
            self.split_states.insert(id, state);
        }
    }

    fn install_watcher(&mut self, cx: &mut Context<Self>) {
        let (sender, receiver) = async_channel::unbounded();
        match notify::recommended_watcher(move |event| {
            let _ = sender.try_send(event);
        }) {
            Ok(watcher) => self.watcher = Some(watcher),
            Err(error) => {
                self.message = Some(format!("Folder change monitoring is unavailable: {error}"));
                return;
            }
        }
        self.sync_watches();
        cx.spawn(async move |this, cx| {
            while let Ok(event) = receiver.recv().await {
                let paths = match event {
                    Ok(event) => event.paths,
                    Err(error) => {
                        let _ = this.update(cx, |this, cx| {
                            this.message = Some(format!("Folder watcher error: {error}"));
                            cx.notify();
                        });
                        continue;
                    }
                };
                let _ = this.update(cx, |this, cx| {
                    let affected = this
                        .workspace()
                        .panels
                        .iter()
                        .filter(|panel| {
                            let folder = &panel.active_tab().path;
                            paths
                                .iter()
                                .any(|path| path.starts_with(folder) || folder.starts_with(path))
                        })
                        .map(|panel| panel.id)
                        .collect::<Vec<_>>();
                    for id in affected {
                        this.schedule_panel_load(id, cx);
                    }
                });
            }
            anyhow::Ok(())
        })
        .detach();
    }

    fn sync_watches(&mut self) {
        let Some(watcher) = &mut self.watcher else {
            return;
        };
        for path in self.watched_paths.drain() {
            let _ = watcher.unwatch(&path);
        }
        let paths = self
            .state
            .workspace()
            .panels
            .iter()
            .map(|panel| panel.active_tab().path.clone())
            .collect::<HashSet<_>>();
        for path in paths {
            if path.is_dir() && watcher.watch(&path, RecursiveMode::NonRecursive).is_ok() {
                self.watched_paths.insert(path);
            }
        }
    }

    fn schedule_panel_load(&mut self, panel_id: PanelId, cx: &mut Context<Self>) {
        let Some(panel) = self.workspace().panel(panel_id) else {
            return;
        };
        let tab = panel.active_tab();
        let path = tab.path.clone();
        let sort = tab.sort;
        let filter = tab.filter.clone();
        let scroll_item = tab.scroll_item;
        let runtime = self.panels.entry(panel_id).or_default();
        runtime.request_id += 1;
        runtime.loading = true;
        runtime.error = None;
        let request_id = runtime.request_id;
        cx.notify();

        let task = cx.background_spawn(async move {
            read_directory_sorted(&path, sort, &filter, true).map_err(|error| error.to_string())
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                let Some(runtime) = this.panels.get_mut(&panel_id) else {
                    return;
                };
                if runtime.request_id != request_id {
                    return;
                }
                runtime.loading = false;
                match result {
                    Ok(entries) => {
                        runtime.entries = entries;
                        runtime.error = None;
                        runtime
                            .selected
                            .retain(|path| runtime.entries.iter().any(|entry| &entry.path == path));
                        if scroll_item > 0 && !runtime.entries.is_empty() {
                            runtime.scroll.scroll_to_item(
                                scroll_item.min(runtime.entries.len() - 1),
                                ScrollStrategy::Top,
                            );
                        }
                    }
                    Err(error) => {
                        runtime.entries.clear();
                        runtime.error = Some(error);
                    }
                }
                cx.notify();
            })
        })
        .detach();
    }

    fn capture_panel_scroll(&mut self, panel_id: PanelId) {
        let scroll_item = self
            .panels
            .get(&panel_id)
            .map(|runtime| runtime.scroll.0.borrow().base_handle.logical_scroll_top().0);
        if let (Some(scroll_item), Some(panel)) =
            (scroll_item, self.state.workspace_mut().panel_mut(panel_id))
        {
            panel.active_tab_mut().scroll_item = scroll_item;
        }
    }

    fn refresh_all_panels(&mut self, cx: &mut Context<Self>) {
        let ids = self
            .workspace()
            .panels
            .iter()
            .map(|panel| panel.id)
            .collect::<Vec<_>>();
        for id in ids {
            self.schedule_panel_load(id, cx);
        }
    }

    fn navigate_active(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let id = self.active_panel_id();
        self.navigate_panel(id, path, true, cx);
    }

    fn navigate_panel(
        &mut self,
        panel_id: PanelId,
        path: PathBuf,
        record_history: bool,
        cx: &mut Context<Self>,
    ) {
        self.capture_panel_scroll(panel_id);
        let old_path = self
            .workspace()
            .panel(panel_id)
            .map(|panel| panel.active_tab().path.clone());
        let sync = self.workspace().synchronized_navigation && panel_id == self.active_panel_id();
        if let Some(panel) = self.state.workspace_mut().panel_mut(panel_id) {
            if record_history {
                panel.active_tab_mut().navigate(path.clone());
            } else {
                panel.active_tab_mut().path = path.clone();
            }
        }
        if let Some(runtime) = self.panels.get_mut(&panel_id) {
            runtime.selected.clear();
            runtime.selection_anchor = None;
        }
        self.schedule_panel_load(panel_id, cx);

        if sync && let Some(old_path) = old_path {
            let peers = self
                .workspace()
                .panels
                .iter()
                .filter(|panel| panel.id != panel_id)
                .map(|panel| (panel.id, panel.active_tab().path.clone()))
                .collect::<Vec<_>>();
            for (peer_id, peer_path) in peers {
                let target = if path.parent() == Some(old_path.as_path()) {
                    path.file_name().map(|name| peer_path.join(name))
                } else if old_path.parent() == Some(path.as_path()) {
                    peer_path.parent().map(Path::to_path_buf)
                } else {
                    None
                };
                if let Some(target) = target.filter(|target| target.exists()) {
                    if let Some(panel) = self.state.workspace_mut().panel_mut(peer_id) {
                        panel.active_tab_mut().navigate(target);
                    }
                    if let Some(runtime) = self.panels.get_mut(&peer_id) {
                        runtime.selected.clear();
                    }
                    self.schedule_panel_load(peer_id, cx);
                }
            }
        }
        self.sync_watches();
        self.refresh_context_tool(cx);
        self.persist();
        cx.notify();
    }

    fn go_back(&mut self, window: Option<&mut Window>, cx: &mut Context<Self>) {
        let id = self.active_panel_id();
        self.capture_panel_scroll(id);
        let target = self
            .state
            .workspace_mut()
            .active_panel_mut()
            .active_tab_mut()
            .go_back();
        if let Some(path) = target {
            if let Some(runtime) = self.panels.get_mut(&id) {
                runtime.selected.clear();
            }
            self.schedule_panel_load(id, cx);
            self.sync_watches();
            if let Some(window) = window {
                self.set_address_input(path, window, cx);
            }
            self.refresh_context_tool(cx);
            self.persist();
        }
    }

    fn go_forward(&mut self, window: Option<&mut Window>, cx: &mut Context<Self>) {
        let id = self.active_panel_id();
        self.capture_panel_scroll(id);
        let target = self
            .state
            .workspace_mut()
            .active_panel_mut()
            .active_tab_mut()
            .go_forward();
        if let Some(path) = target {
            if let Some(runtime) = self.panels.get_mut(&id) {
                runtime.selected.clear();
            }
            self.schedule_panel_load(id, cx);
            self.sync_watches();
            if let Some(window) = window {
                self.set_address_input(path, window, cx);
            }
            self.refresh_context_tool(cx);
            self.persist();
        }
    }

    fn go_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(parent) = self.active_path().parent().map(Path::to_path_buf) {
            self.navigate_active(parent.clone(), cx);
            self.set_address_input(parent, window, cx);
        }
    }

    fn set_address_input(&self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.address_input.update(cx, |input, cx| {
            input.set_value(path.display().to_string(), window, cx);
        });
    }

    fn activate_panel(&mut self, panel_id: PanelId, window: &mut Window, cx: &mut Context<Self>) {
        if self.workspace().panel(panel_id).is_none() {
            return;
        }
        self.state.workspace_mut().active_panel = panel_id;
        if self.workspace().target_panel == Some(panel_id) {
            self.state.workspace_mut().target_panel = self
                .workspace()
                .panels
                .iter()
                .map(|panel| panel.id)
                .find(|id| *id != panel_id);
        }
        let path = self.active_path().to_path_buf();
        let filter = self.workspace().active_panel().active_tab().filter.clone();
        self.address_input.update(cx, |input, cx| {
            input.set_value(path.display().to_string(), window, cx);
        });
        self.filter_input.update(cx, |input, cx| {
            input.set_value(filter, window, cx);
        });
        self.refresh_context_tool(cx);
        self.persist();
        cx.notify();
    }

    fn activate_entry(
        &mut self,
        panel_id: PanelId,
        entry: FileEntry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if entry.is_directory() {
            self.state.workspace_mut().active_panel = panel_id;
            self.navigate_panel(panel_id, entry.path.clone(), true, cx);
            self.set_address_input(entry.path, window, cx);
        } else if let Err(error) = opener::open(&entry.path) {
            self.message = Some(format!("Could not open {}: {error}", entry.path.display()));
            cx.notify();
        }
    }

    fn select_entry(
        &mut self,
        panel_id: PanelId,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.state.workspace_mut().active_panel = panel_id;
        let modifiers = window.modifiers();
        let Some(runtime) = self.panels.get_mut(&panel_id) else {
            return;
        };
        let Some(entry) = runtime.entries.get(index) else {
            return;
        };
        let path = entry.path.clone();
        if modifiers.shift {
            let anchor = runtime.selection_anchor.unwrap_or(index);
            let (start, end) = if anchor <= index {
                (anchor, index)
            } else {
                (index, anchor)
            };
            runtime.selected.clear();
            runtime.selected.extend(
                runtime.entries[start..=end]
                    .iter()
                    .map(|entry| entry.path.clone()),
            );
        } else if modifiers.control {
            if !runtime.selected.insert(path.clone()) {
                runtime.selected.remove(&path);
            }
            runtime.selection_anchor = Some(index);
        } else {
            runtime.selected.clear();
            runtime.selected.insert(path);
            runtime.selection_anchor = Some(index);
        }
        self.message = None;
        self.refresh_context_tool(cx);
        cx.notify();
    }

    fn select_relative(&mut self, delta: isize, cx: &mut Context<Self>) {
        let id = self.active_panel_id();
        let Some(runtime) = self.panels.get_mut(&id) else {
            return;
        };
        if runtime.entries.is_empty() {
            return;
        }
        let current = runtime
            .entries
            .iter()
            .position(|entry| runtime.selected.contains(&entry.path))
            .unwrap_or(if delta >= 0 {
                0
            } else {
                runtime.entries.len() - 1
            });
        let next = (current as isize + delta).clamp(0, runtime.entries.len() as isize - 1) as usize;
        runtime.selected.clear();
        runtime.selected.insert(runtime.entries[next].path.clone());
        runtime.selection_anchor = Some(next);
        runtime.scroll.scroll_to_item(next, ScrollStrategy::Nearest);
        let preview_path = self
            .preview_visible
            .then(|| runtime.entries[next].path.clone());
        if let Some(path) = preview_path {
            self.load_preview(path, cx);
        }
        cx.notify();
    }

    fn switch_tab(
        &mut self,
        panel_id: PanelId,
        tab_index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.capture_panel_scroll(panel_id);
        let Some(panel) = self.state.workspace_mut().panel_mut(panel_id) else {
            return;
        };
        if tab_index >= panel.tabs.len() {
            return;
        }
        panel.active_tab = tab_index;
        self.state.workspace_mut().active_panel = panel_id;
        if let Some(runtime) = self.panels.get_mut(&panel_id) {
            runtime.selected.clear();
        }
        let path = self
            .workspace()
            .panel(panel_id)
            .unwrap()
            .active_tab()
            .path
            .clone();
        self.schedule_panel_load(panel_id, cx);
        self.set_address_input(path, window, cx);
        self.sync_watches();
        self.refresh_context_tool(cx);
        self.persist();
    }

    fn new_tab(&mut self, window: Option<&mut Window>, cx: &mut Context<Self>) {
        let path = self.active_path().to_path_buf();
        let id = self.active_panel_id();
        self.state
            .workspace_mut()
            .active_panel_mut()
            .open_tab(path.clone());
        self.schedule_panel_load(id, cx);
        if let Some(window) = window {
            self.set_address_input(path, window, cx);
        }
        self.sync_watches();
        self.persist();
    }

    fn close_active_tab(&mut self, cx: &mut Context<Self>) {
        let id = self.active_panel_id();
        let active = self.workspace().active_panel().active_tab;
        if self
            .state
            .workspace_mut()
            .active_panel_mut()
            .close_tab(active)
        {
            self.schedule_panel_load(id, cx);
            self.sync_watches();
            self.persist();
        } else {
            self.message = Some("Pinned or final tabs cannot be closed.".to_string());
        }
    }

    fn toggle_pin_active_tab(&mut self, cx: &mut Context<Self>) {
        let tab = self
            .state
            .workspace_mut()
            .active_panel_mut()
            .active_tab_mut();
        tab.pinned = !tab.pinned;
        self.persist();
        cx.notify();
    }

    fn split_active(&mut self, axis: SplitAxis, cx: &mut Context<Self>) {
        if let Some(id) = self.state.workspace_mut().split_active(axis) {
            self.panels.insert(id, PanelRuntime::default());
            self.schedule_panel_load(id, cx);
            self.ensure_split_states(cx);
            self.sync_watches();
            self.persist();
        } else {
            self.message = Some("The initial UI supports up to four panels.".to_string());
        }
    }

    fn apply_layout(&mut self, preset: LayoutPreset, cx: &mut Context<Self>) {
        self.state.workspace_mut().apply_preset(preset);
        let valid = self
            .workspace()
            .panels
            .iter()
            .map(|panel| panel.id)
            .collect::<HashSet<_>>();
        self.panels.retain(|id, _| valid.contains(id));
        for id in valid {
            self.panels.entry(id).or_default();
            self.schedule_panel_load(id, cx);
        }
        self.ensure_split_states(cx);
        self.sync_watches();
        self.persist();
        cx.notify();
    }

    fn close_panel(&mut self, panel_id: PanelId, cx: &mut Context<Self>) {
        if self.state.workspace_mut().close_panel(panel_id) {
            self.panels.remove(&panel_id);
            self.ensure_split_states(cx);
            self.sync_watches();
            self.persist();
            cx.notify();
        }
    }

    fn set_sort(&mut self, field: crate::model::SortField, cx: &mut Context<Self>) {
        let tab = self
            .state
            .workspace_mut()
            .active_panel_mut()
            .active_tab_mut();
        if tab.sort.field == field {
            tab.sort.direction = match tab.sort.direction {
                crate::model::SortDirection::Ascending => crate::model::SortDirection::Descending,
                crate::model::SortDirection::Descending => crate::model::SortDirection::Ascending,
            };
        } else {
            tab.sort.field = field;
            tab.sort.direction = crate::model::SortDirection::Ascending;
        }
        let id = self.active_panel_id();
        self.schedule_panel_load(id, cx);
        self.persist();
    }

    fn show_sidebar_tool(&mut self, tool: SidebarTool, cx: &mut Context<Self>) {
        let sidebar = &mut self.state.workspace_mut().sidebar;
        if sidebar.open && sidebar.active_tool == tool {
            sidebar.open = false;
        } else {
            sidebar.open = true;
            sidebar.active_tool = tool;
        }
        self.refresh_context_tool(cx);
        self.persist();
        cx.notify();
    }

    fn refresh_context_tool(&mut self, cx: &mut Context<Self>) {
        if !self.workspace().sidebar.open {
            return;
        }
        if let Some(token) = self.context_cancellation.take() {
            token.cancel();
        }
        match self.workspace().sidebar.active_tool {
            SidebarTool::Git => self.load_git(cx),
            SidebarTool::Info => self.load_folder_info(cx),
            SidebarTool::Statistics => self.load_statistics(cx),
            SidebarTool::Search | SidebarTool::Navigation | SidebarTool::Shelf => {}
        }
    }

    fn load_git(&mut self, cx: &mut Context<Self>) {
        let path = self.active_path().to_path_buf();
        let request_path = path.clone();
        let selected = self.selected_paths().into_iter().next();
        self.git_state = ContextState::Loading(path.clone());
        let task = cx.background_spawn(async move {
            repository_info(&path, selected.as_deref()).map_err(|e| e.to_string())
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                if this.active_path() != request_path {
                    return;
                }
                this.git_state = match result {
                    Ok(value) => ContextState::Ready(value),
                    Err(error) => ContextState::Error(this.active_path().to_path_buf(), error),
                };
                cx.notify();
            })
        })
        .detach();
    }

    fn load_commit_files(&mut self, root: PathBuf, hash: String, cx: &mut Context<Self>) {
        let task = cx.background_spawn(async move {
            commit_files(&root, &hash)
                .map(|files| (hash, files))
                .map_err(|error| error.to_string())
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok((hash, files)) => {
                        if let ContextState::Ready(Some(repository)) = &mut this.git_state
                            && let Some(commit) = repository
                                .commits
                                .iter_mut()
                                .find(|commit| commit.hash == hash)
                        {
                            commit.changed_files = files;
                        }
                    }
                    Err(error) => {
                        this.message = Some(format!("Could not list commit files: {error}"))
                    }
                }
                cx.notify();
            })
        })
        .detach();
    }

    fn load_folder_info(&mut self, cx: &mut Context<Self>) {
        let path = self.active_path().to_path_buf();
        let request_path = path.clone();
        let cancellation = CancellationToken::default();
        self.context_cancellation = Some(cancellation.clone());
        self.info_state = ContextState::Loading(path.clone());
        let task = cx.background_spawn(async move {
            calculate_folder_info(&path, &cancellation).map_err(|error| error.to_string())
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                if this.active_path() != request_path {
                    return;
                }
                this.info_state = match result {
                    Ok(info) => ContextState::Ready(info),
                    Err(error) => ContextState::Error(this.active_path().to_path_buf(), error),
                };
                cx.notify();
            })
        })
        .detach();
    }

    fn load_statistics(&mut self, cx: &mut Context<Self>) {
        let path = self.active_path().to_path_buf();
        let request_path = path.clone();
        let cancellation = CancellationToken::default();
        self.context_cancellation = Some(cancellation.clone());
        self.statistics_state = ContextState::Loading(path.clone());
        let task = cx.background_spawn(async move {
            calculate_folder_statistics(&path, &cancellation).map_err(|error| error.to_string())
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                if this.active_path() != request_path {
                    return;
                }
                this.statistics_state = match result {
                    Ok(statistics) => ContextState::Ready(statistics),
                    Err(error) => ContextState::Error(this.active_path().to_path_buf(), error),
                };
                cx.notify();
            })
        })
        .detach();
    }

    fn run_search(&mut self, cx: &mut Context<Self>) {
        if let Some(token) = self.search_cancellation.take() {
            token.cancel();
        }
        let query = self.filter_input.read(cx).value().to_string();
        let roots = self
            .workspace()
            .panels
            .iter()
            .map(|panel| panel.active_tab().path.clone())
            .collect::<Vec<_>>();
        let path = self.active_path().to_path_buf();
        let request_path = path.clone();
        let cancellation = CancellationToken::default();
        self.search_cancellation = Some(cancellation.clone());
        self.search_results = ContextState::Loading(path.clone());
        let options = SearchOptions::parse(&query);
        let task = cx.background_spawn(async move {
            search_paths(&roots, &options, &cancellation).map_err(|error| error.to_string())
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                if this.active_path() != request_path {
                    return;
                }
                this.search_results = match result {
                    Ok(results) => ContextState::Ready(results),
                    Err(error) => ContextState::Error(this.active_path().to_path_buf(), error),
                };
                cx.notify();
            })
        })
        .detach();
    }

    fn cancel_context_work(&mut self, cx: &mut Context<Self>) {
        if let Some(token) = self.context_cancellation.take() {
            token.cancel();
        }
        if let Some(token) = self.search_cancellation.take() {
            token.cancel();
        }
        self.message = Some("Background analysis cancellation requested.".to_string());
        cx.notify();
    }

    fn add_selection_to_shelf(&mut self, cx: &mut Context<Self>) {
        let selected = self.selected_paths();
        if selected.is_empty() {
            self.message = Some("Select one or more items to add to the Shelf.".to_string());
            return;
        }
        let added = self.state.shelf_mut().add_paths(selected);
        self.message = Some(format!(
            "Added {added} item(s) to {}.",
            self.state.shelf().name
        ));
        self.persist();
        cx.notify();
    }

    fn shelf_transfer(&mut self, kind: OperationKind, cx: &mut Context<Self>) {
        let sources = self
            .state
            .shelf()
            .items
            .iter()
            .filter(|item| item.path.exists())
            .map(|item| item.path.clone())
            .collect();
        self.prepare_transfer(kind, sources, self.active_path().to_path_buf(), cx);
    }

    fn compress_shelf(&mut self, cx: &mut Context<Self>) {
        let sources = self
            .state
            .shelf()
            .items
            .iter()
            .filter(|item| item.path.exists())
            .map(|item| item.path.clone())
            .collect::<Vec<_>>();
        let archive = self
            .active_path()
            .join(format!("{}.zip", self.state.shelf().name));
        match OperationPlan::compress(sources, archive, ConflictResolution::Rename) {
            Ok(plan) => {
                self.pending_plan = Some(plan);
                self.dialog = DialogState::Operation;
                cx.notify();
            }
            Err(error) => self.message = Some(error.to_string()),
        }
    }

    fn copy_shelf_paths(&mut self, cx: &mut Context<Self>) {
        let text = self
            .state
            .shelf()
            .items
            .iter()
            .map(|item| item.path.display().to_string())
            .collect::<Vec<_>>()
            .join("\r\n");
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.message = Some("Shelf paths copied.".to_string());
    }

    fn remove_shelf_item(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.state.shelf().items.len() {
            self.state.shelf_mut().items.remove(index);
            self.persist();
            cx.notify();
        }
    }

    fn toggle_shelf_persistence(&mut self, cx: &mut Context<Self>) {
        let shelf = self.state.shelf_mut();
        shelf.persistent = !shelf.persistent;
        self.persist();
        cx.notify();
    }

    fn select_shelf(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.state.shelves.len() {
            self.state.active_shelf = index;
            self.persist();
            cx.notify();
        }
    }

    fn begin_name_action(
        &mut self,
        action: NameAction,
        default_value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.name_action = Some(action);
        self.dialog = DialogState::Name;
        self.name_input.update(cx, |input, cx| {
            input.set_value(default_value, window, cx);
            input.focus(window, cx);
        });
        cx.notify();
    }

    fn apply_name_action(&mut self, value: &str, cx: &mut Context<Self>) {
        let Some(action) = self.name_action.take() else {
            return;
        };
        let result =
            match action {
                NameAction::Rename(source) => {
                    OperationPlan::rename(source, value, ConflictResolution::Rename).map(|plan| {
                        self.pending_plan = Some(plan);
                        self.dialog = DialogState::Operation;
                    })
                }
                NameAction::BatchRename(sources) => {
                    OperationPlan::batch_rename_prefix(sources, value, ConflictResolution::Rename)
                        .map(|plan| {
                            self.pending_plan = Some(plan);
                            self.dialog = DialogState::Operation;
                        })
                }
                NameAction::CreateFolder(parent) => OperationPlan::create_folder(&parent, value)
                    .map(|plan| {
                        self.pending_plan = Some(plan);
                        self.dialog = DialogState::Operation;
                    }),
                NameAction::SaveWorkspace => {
                    let name = value.trim();
                    if name.is_empty() {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "workspace name cannot be empty",
                        ))
                    } else {
                        let mut workspace = self.workspace().clone();
                        workspace.name = name.to_string();
                        if let Some(existing) = self
                            .state
                            .workspaces
                            .iter()
                            .position(|workspace| workspace.name.eq_ignore_ascii_case(name))
                        {
                            self.state.workspaces[existing] = workspace;
                            self.state.active_workspace = existing;
                        } else {
                            self.state.workspaces.push(workspace);
                            self.state.active_workspace = self.state.workspaces.len() - 1;
                        }
                        self.dialog = DialogState::None;
                        self.persist();
                        Ok(())
                    }
                }
                NameAction::NewShelf => {
                    let name = value.trim();
                    if name.is_empty() {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Shelf name cannot be empty",
                        ))
                    } else {
                        self.state.shelves.push(crate::model::Shelf {
                            name: name.to_string(),
                            ..crate::model::Shelf::default()
                        });
                        self.state.active_shelf = self.state.shelves.len() - 1;
                        self.dialog = DialogState::None;
                        self.persist();
                        Ok(())
                    }
                }
            };
        if let Err(error) = result {
            self.message = Some(error.to_string());
            self.dialog = DialogState::Name;
        }
        cx.notify();
    }

    fn select_workspace(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.state.workspaces.len() {
            return;
        }
        self.state.active_workspace = index;
        let valid = self
            .workspace()
            .panels
            .iter()
            .map(|panel| panel.id)
            .collect::<HashSet<_>>();
        self.panels.retain(|id, _| valid.contains(id));
        for id in valid {
            self.panels.entry(id).or_default();
            self.schedule_panel_load(id, cx);
        }
        self.ensure_split_states(cx);
        self.sync_watches();
        self.set_address_input(self.active_path().to_path_buf(), window, cx);
        self.refresh_context_tool(cx);
        self.persist();
        cx.notify();
    }

    fn prepare_target_transfer(&mut self, kind: OperationKind, cx: &mut Context<Self>) {
        let sources = self.selected_paths();
        let target_id = self.workspace().target_panel.or_else(|| {
            self.workspace()
                .panels
                .iter()
                .map(|panel| panel.id)
                .find(|id| *id != self.active_panel_id())
        });
        let Some(destination) = target_id
            .and_then(|id| self.workspace().panel(id))
            .map(|panel| panel.active_tab().path.clone())
        else {
            self.message = Some("Open another panel and designate it as the target.".to_string());
            return;
        };
        self.prepare_transfer(kind, sources, destination, cx);
    }

    fn prepare_transfer(
        &mut self,
        kind: OperationKind,
        sources: Vec<PathBuf>,
        destination: PathBuf,
        cx: &mut Context<Self>,
    ) {
        match OperationPlan::transfer(kind, sources, &destination, ConflictResolution::Rename) {
            Ok(plan) => {
                self.pending_plan = Some(plan);
                self.dialog = DialogState::Operation;
                cx.notify();
            }
            Err(error) => {
                self.message = Some(error.to_string());
                cx.notify();
            }
        }
    }

    fn prepare_delete(&mut self, cx: &mut Context<Self>) {
        match OperationPlan::delete(self.selected_paths()) {
            Ok(plan) => {
                self.pending_plan = Some(plan);
                self.dialog = DialogState::Operation;
                cx.notify();
            }
            Err(error) => self.message = Some(error.to_string()),
        }
    }

    fn set_pending_resolution(&mut self, resolution: ConflictResolution, cx: &mut Context<Self>) {
        if let Some(plan) = self.pending_plan.clone() {
            match plan.with_resolution(resolution) {
                Ok(plan) => self.pending_plan = Some(plan),
                Err(error) => self.message = Some(error.to_string()),
            }
        }
        cx.notify();
    }

    fn confirm_operation(&mut self, cx: &mut Context<Self>) {
        let Some(plan) = self.pending_plan.take() else {
            return;
        };
        if plan.executable_count() == 0 {
            self.message = Some("Every planned item is skipped; nothing was queued.".to_string());
            self.dialog = DialogState::None;
            return;
        }
        let id = self.queue.enqueue(plan);
        self.message = Some(format!("Operation #{id} queued."));
        self.dialog = DialogState::None;
        self.start_next_operation(cx);
        cx.notify();
    }

    fn start_next_operation(&mut self, cx: &mut Context<Self>) {
        let Some((id, plan)) = self.queue.next_queued() else {
            return;
        };
        let control = OperationControl::default();
        self.operation_control = Some(control.clone());
        self.operation_paused = false;
        let undo_root = self
            .store
            .path()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("undo")
            .join(id.to_string());
        let task = cx.background_spawn(async move { plan.execute(&control, &undo_root) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                this.queue.complete(id, result);
                this.operation_control = None;
                this.operation_paused = false;
                this.refresh_all_panels(cx);
                this.start_next_operation(cx);
                cx.notify();
            })
        })
        .detach();
    }

    fn toggle_operation_pause(&mut self, cx: &mut Context<Self>) {
        let Some(control) = &self.operation_control else {
            return;
        };
        if self.operation_paused {
            control.resume();
            self.operation_paused = false;
        } else {
            control.pause();
            self.operation_paused = true;
        }
        if let Some(id) = self.queue.active
            && let Some(job) = self.queue.jobs.iter_mut().find(|job| job.id == id)
        {
            job.status = if self.operation_paused {
                JobStatus::Paused
            } else {
                JobStatus::Running
            };
        }
        cx.notify();
    }

    fn cancel_operation(&mut self, cx: &mut Context<Self>) {
        if let Some(control) = &self.operation_control {
            control.cancel();
            self.message =
                Some("Cancellation requested. The current safe boundary will finish.".to_string());
            cx.notify();
        }
    }

    fn undo_last_operation(&mut self, cx: &mut Context<Self>) {
        let Some(job) = self.queue.latest_undoable() else {
            self.message = Some("There is no operation Nimbus can safely undo.".to_string());
            return;
        };
        let id = job.id;
        let outcome = job.outcome.take().unwrap();
        job.summary = "Undoing…".to_string();
        let task = cx.background_spawn(async move { outcome.undo() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                if let Some(job) = this.queue.jobs.iter_mut().find(|job| job.id == id) {
                    job.summary = match result {
                        Ok(()) => "Undone".to_string(),
                        Err(error) => format!("Undo failed: {error}"),
                    };
                }
                this.refresh_all_panels(cx);
                cx.notify();
            })
        })
        .detach();
    }

    fn copy_selection_to_internal_clipboard(&mut self, cut: bool, cx: &mut Context<Self>) {
        let paths = self.selected_paths();
        if paths.is_empty() {
            self.message = Some("Select items first.".to_string());
            return;
        }
        let text = paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\r\n");
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.clipboard_paths = paths;
        self.clipboard_cut = cut;
        self.message = Some(
            if cut {
                "Cut selection ready to move."
            } else {
                "Selection copied."
            }
            .to_string(),
        );
    }

    fn paste_internal_clipboard(&mut self, cx: &mut Context<Self>) {
        if self.clipboard_paths.is_empty() {
            self.message = Some("Nimbus has no copied file selection to paste.".to_string());
            return;
        }
        let kind = if self.clipboard_cut {
            OperationKind::Move
        } else {
            OperationKind::Copy
        };
        self.prepare_transfer(
            kind,
            self.clipboard_paths.clone(),
            self.active_path().to_path_buf(),
            cx,
        );
    }

    fn copy_formatted_paths(&mut self, format: PathFormat, cx: &mut Context<Self>) {
        let text = self
            .selected_or_folder()
            .iter()
            .map(|path| format_path(path, format))
            .collect::<Vec<_>>()
            .join("\r\n");
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.message = Some("Path copied.".to_string());
    }

    fn load_preview(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.preview_visible = true;
        self.preview = Some(PreviewState::Loading(path.clone()));
        let task = cx.background_spawn(async move {
            QuickPreview::load(&path).map_err(|error| (path, error.to_string()))
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                this.preview = Some(match result {
                    Ok(preview) => PreviewState::Ready(preview),
                    Err((path, error)) => PreviewState::Error(path, error),
                });
                cx.notify();
            })
        })
        .detach();
    }

    fn toggle_preview(&mut self, cx: &mut Context<Self>) {
        if self.preview_visible {
            self.preview_visible = false;
            self.preview_pinned = false;
            cx.notify();
            return;
        }
        let Some(path) = self.selected_paths().into_iter().next() else {
            self.message = Some("Select a file to preview.".to_string());
            return;
        };
        self.load_preview(path, cx);
    }

    fn open_previewed_file(&mut self, cx: &mut Context<Self>) {
        if let Some(PreviewState::Ready(preview)) = &self.preview
            && let Err(error) = opener::open(&preview.path)
        {
            self.message = Some(format!(
                "Could not open {}: {error}",
                preview.path.display()
            ));
            cx.notify();
        }
    }

    fn toggle_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.dialog == DialogState::Palette {
            self.dialog = DialogState::None;
            self.focus_handle.focus(window, cx);
        } else {
            self.dialog = DialogState::Palette;
            self.palette_input.update(cx, |input, cx| {
                input.set_value("", window, cx);
                input.focus(window, cx);
            });
        }
        cx.notify();
    }

    fn command_enabled(&self, command: CommandId) -> bool {
        let has_selection = !self.selected_paths().is_empty();
        match command {
            CommandId::AddToShelf
            | CommandId::ShowPreview
            | CommandId::Rename
            | CommandId::Delete
            | CommandId::Reveal => has_selection,
            CommandId::BatchRename => self.selected_paths().len() >= 2,
            CommandId::CopyToTarget | CommandId::MoveToTarget => {
                has_selection && self.workspace().panels.len() > 1
            }
            CommandId::Undo => self.queue.jobs.iter().any(|job| {
                job.outcome
                    .as_ref()
                    .is_some_and(|outcome| !outcome.undo.is_empty())
            }),
            _ => true,
        }
    }

    fn matching_commands(&self, cx: &Context<Self>) -> Vec<&'static CommandSpec> {
        let query = self.palette_input.read(cx).value().trim().to_lowercase();
        COMMANDS
            .iter()
            .filter(|command| {
                crate::filesystem::fuzzy_matches(command.name, &query)
                    || command.description.to_lowercase().contains(&query)
            })
            .collect()
    }

    fn execute_first_palette_command(
        &mut self,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        if let Some(command) = self.matching_commands(cx).first() {
            let id = command.id;
            self.execute_command(id, window, cx);
        }
    }

    fn execute_command(
        &mut self,
        command: CommandId,
        mut window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        if !self.command_enabled(command) {
            self.message = Some("That command is unavailable in the current context.".to_string());
            return;
        }
        if let Some(spec) = COMMANDS.iter().find(|spec| spec.id == command) {
            self.state.command_history.retain(|name| name != spec.name);
            self.state.command_history.insert(0, spec.name.to_string());
            self.state.command_history.truncate(50);
        }
        self.dialog = DialogState::None;
        match command {
            CommandId::CopyWindowsPath => self.copy_formatted_paths(PathFormat::Windows, cx),
            CommandId::CopyPowerShellPath => self.copy_formatted_paths(PathFormat::PowerShell, cx),
            CommandId::CopyFileUri => self.copy_formatted_paths(PathFormat::FileUri, cx),
            CommandId::CopyWslPath => self.copy_formatted_paths(PathFormat::Wsl, cx),
            CommandId::OpenTerminal => {
                if let Err(error) = open_terminal(self.active_path()) {
                    self.message = Some(format!("Could not open terminal: {error}"));
                }
            }
            CommandId::Reveal => {
                if let Some(path) = self.selected_paths().into_iter().next()
                    && let Err(error) = reveal_in_file_explorer(&path)
                {
                    self.message = Some(format!("Could not reveal file: {error}"));
                }
            }
            CommandId::Refresh => {
                let id = self.active_panel_id();
                self.schedule_panel_load(id, cx);
            }
            CommandId::NewFolder => {
                if let Some(window) = window.take() {
                    self.begin_name_action(
                        NameAction::CreateFolder(self.active_path().to_path_buf()),
                        "New folder".to_string(),
                        window,
                        cx,
                    );
                }
            }
            CommandId::NewTab => self.new_tab(window.take(), cx),
            CommandId::SplitColumns => self.split_active(SplitAxis::Columns, cx),
            CommandId::SplitRows => self.split_active(SplitAxis::Rows, cx),
            CommandId::GridLayout => self.apply_layout(LayoutPreset::Grid, cx),
            CommandId::AddToShelf => self.add_selection_to_shelf(cx),
            CommandId::ShowPreview => self.toggle_preview(cx),
            CommandId::CopyToTarget => self.prepare_target_transfer(OperationKind::Copy, cx),
            CommandId::MoveToTarget => self.prepare_target_transfer(OperationKind::Move, cx),
            CommandId::Rename => {
                if let (Some(window), Some(path)) =
                    (window.take(), self.selected_paths().into_iter().next())
                {
                    let name = path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    self.begin_name_action(NameAction::Rename(path), name, window, cx);
                } else {
                    self.message = Some(
                        "Choose Rename from the file list or toolbar to enter a new name."
                            .to_string(),
                    );
                }
            }
            CommandId::BatchRename => {
                if let Some(window) = window.take() {
                    self.begin_name_action(
                        NameAction::BatchRename(self.selected_paths()),
                        "renamed_".to_string(),
                        window,
                        cx,
                    );
                }
            }
            CommandId::Delete => self.prepare_delete(cx),
            CommandId::FolderInfo => self.show_sidebar_tool(SidebarTool::Info, cx),
            CommandId::Statistics => self.show_sidebar_tool(SidebarTool::Statistics, cx),
            CommandId::Search => {
                self.show_sidebar_tool(SidebarTool::Search, cx);
                self.run_search(cx);
            }
            CommandId::ToggleSidebar => {
                self.state.workspace_mut().sidebar.open = !self.workspace().sidebar.open;
                self.persist();
            }
            CommandId::Undo => self.undo_last_operation(cx),
        }
        self.persist();
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        if window.focused_input(cx).is_some() {
            if key == "escape" {
                self.dialog = DialogState::None;
                self.name_action = None;
                self.focus_handle.focus(window, cx);
                cx.notify();
            }
            return;
        }

        let mut handled = true;
        if modifiers.control && modifiers.shift && key == "p" {
            self.toggle_palette(window, cx);
        } else if modifiers.control && modifiers.shift && key == "v" {
            self.split_active(SplitAxis::Columns, cx);
        } else if modifiers.control && modifiers.shift && key == "h" {
            self.split_active(SplitAxis::Rows, cx);
        } else if modifiers.control && modifiers.shift && key == "s" {
            self.add_selection_to_shelf(cx);
        } else if modifiers.control && modifiers.shift && key == "n" {
            self.begin_name_action(
                NameAction::CreateFolder(self.active_path().to_path_buf()),
                "New folder".to_string(),
                window,
                cx,
            );
        } else if modifiers.control && modifiers.shift && key == "c" {
            self.copy_formatted_paths(PathFormat::Windows, cx);
        } else if modifiers.control && key == "l" {
            self.address_input
                .update(cx, |input, cx| input.focus(window, cx));
        } else if modifiers.control && key == "f" {
            self.state.workspace_mut().sidebar.open = true;
            self.state.workspace_mut().sidebar.active_tool = SidebarTool::Search;
            self.filter_input
                .update(cx, |input, cx| input.focus(window, cx));
        } else if modifiers.control && key == "t" {
            self.new_tab(Some(window), cx);
        } else if modifiers.control && key == "w" {
            self.close_active_tab(cx);
        } else if modifiers.control && key == "c" {
            self.copy_selection_to_internal_clipboard(false, cx);
        } else if modifiers.control && key == "x" {
            self.copy_selection_to_internal_clipboard(true, cx);
        } else if modifiers.control && key == "v" {
            self.paste_internal_clipboard(cx);
        } else if modifiers.control && key == "z" {
            self.undo_last_operation(cx);
        } else if modifiers.control && key == "b" {
            self.state.workspace_mut().sidebar.open = !self.workspace().sidebar.open;
            self.persist();
        } else if modifiers.alt && key == "left" {
            self.go_back(Some(window), cx);
        } else if modifiers.alt && key == "right" {
            self.go_forward(Some(window), cx);
        } else if key == "f5" {
            let id = self.active_panel_id();
            self.schedule_panel_load(id, cx);
        } else if key == "f2" {
            if let Some(path) = self.selected_paths().into_iter().next() {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.begin_name_action(NameAction::Rename(path), name, window, cx);
            }
        } else if key == "delete" {
            self.prepare_delete(cx);
        } else if key == "space" && !event.is_held {
            self.toggle_preview(cx);
        } else if key == "up" {
            self.select_relative(-1, cx);
        } else if key == "down" {
            self.select_relative(1, cx);
        } else if key == "enter" {
            let id = self.active_panel_id();
            if let Some(entry) = self
                .panels
                .get(&id)
                .and_then(|runtime| {
                    runtime
                        .entries
                        .iter()
                        .find(|entry| runtime.selected.contains(&entry.path))
                })
                .cloned()
            {
                self.activate_entry(id, entry, window, cx);
            }
        } else if key == "escape" {
            if self.preview_visible {
                self.preview_visible = false;
                self.preview_pinned = false;
            } else {
                self.dialog = DialogState::None;
            }
            cx.notify();
        } else {
            handled = false;
        }
        if handled {
            cx.stop_propagation();
        }
    }

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let tab = self.workspace().active_panel().active_tab();
        let panels = self.workspace().panels.len();
        h_flex()
            .h_12()
            .flex_shrink_0()
            .gap_1()
            .px_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                Button::new("navigate-back")
                    .ghost()
                    .small()
                    .icon(IconName::ArrowLeft)
                    .tooltip("Back · Alt+Left")
                    .disabled(!tab.can_go_back())
                    .on_click(cx.listener(|this, _, window, cx| this.go_back(Some(window), cx))),
            )
            .child(
                Button::new("navigate-forward")
                    .ghost()
                    .small()
                    .icon(IconName::ArrowRight)
                    .tooltip("Forward · Alt+Right")
                    .disabled(!tab.can_go_forward())
                    .on_click(cx.listener(|this, _, window, cx| this.go_forward(Some(window), cx))),
            )
            .child(
                Button::new("navigate-up")
                    .ghost()
                    .small()
                    .icon(IconName::ChevronUp)
                    .tooltip("Parent folder")
                    .disabled(self.active_path().parent().is_none())
                    .on_click(cx.listener(|this, _, window, cx| this.go_up(window, cx))),
            )
            .child(
                Button::new("refresh")
                    .ghost()
                    .small()
                    .icon(IconName::Redo)
                    .tooltip("Refresh · F5")
                    .on_click(cx.listener(|this, _, _, cx| {
                        let id = this.active_panel_id();
                        this.schedule_panel_load(id, cx);
                    })),
            )
            .child(
                div()
                    .ml_1()
                    .h_8()
                    .min_w(px(180.))
                    .flex_1()
                    .child(Input::new(&self.address_input).small().cleanable(false)),
            )
            .child(
                div().h_8().w(px(210.)).child(
                    Input::new(&self.filter_input)
                        .small()
                        .prefix(Icon::new(IconName::Search).small())
                        .cleanable(true),
                ),
            )
            .child(
                Button::new("new-folder")
                    .ghost()
                    .small()
                    .icon(IconName::Plus)
                    .tooltip("Create folder · Ctrl+Shift+N")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.begin_name_action(
                            NameAction::CreateFolder(this.active_path().to_path_buf()),
                            "New folder".to_string(),
                            window,
                            cx,
                        )
                    })),
            )
            .child(
                Button::new("copy-to-target")
                    .ghost()
                    .small()
                    .icon(IconName::Copy)
                    .tooltip("Copy selection to target panel")
                    .disabled(!self.command_enabled(CommandId::CopyToTarget))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.prepare_target_transfer(OperationKind::Copy, cx)
                    })),
            )
            .child(
                Button::new("move-to-target")
                    .ghost()
                    .small()
                    .icon(IconName::ArrowRight)
                    .tooltip("Move selection to target panel")
                    .disabled(!self.command_enabled(CommandId::MoveToTarget))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.prepare_target_transfer(OperationKind::Move, cx)
                    })),
            )
            .child(
                Button::new("quick-look")
                    .ghost()
                    .small()
                    .icon(IconName::Eye)
                    .tooltip("Quick Look · Space")
                    .disabled(!self.command_enabled(CommandId::ShowPreview))
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_preview(cx))),
            )
            .child(
                Button::new("split-columns")
                    .ghost()
                    .small()
                    .icon(IconName::PanelRightOpen)
                    .tooltip("Split left/right · Ctrl+Shift+V")
                    .disabled(panels >= 4)
                    .on_click(
                        cx.listener(|this, _, _, cx| this.split_active(SplitAxis::Columns, cx)),
                    ),
            )
            .child(
                Button::new("split-rows")
                    .ghost()
                    .small()
                    .icon(IconName::PanelBottomOpen)
                    .tooltip("Split top/bottom · Ctrl+Shift+H")
                    .disabled(panels >= 4)
                    .on_click(cx.listener(|this, _, _, cx| this.split_active(SplitAxis::Rows, cx))),
            )
            .child(
                Button::new("sync-navigation")
                    .ghost()
                    .small()
                    .icon(IconName::Replace)
                    .tooltip("Synchronize relative navigation across panels")
                    .disabled(panels < 2)
                    .when(self.workspace().synchronized_navigation, |button| {
                        button.primary()
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        let enabled = !this.workspace().synchronized_navigation;
                        this.state.workspace_mut().synchronized_navigation = enabled;
                        this.message = Some(if enabled {
                            "Synchronized navigation enabled.".to_string()
                        } else {
                            "Synchronized navigation disabled.".to_string()
                        });
                        this.persist();
                        cx.notify();
                    })),
            )
            .children(
                LayoutPreset::ALL
                    .into_iter()
                    .enumerate()
                    .map(|(index, preset)| {
                        Button::new(("layout-preset", index))
                            .ghost()
                            .xsmall()
                            .label(preset.label())
                            .tooltip(format!("{} panel layout", preset.panel_count()))
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.apply_layout(preset, cx)),
                            )
                    }),
            )
            .child(
                Button::new("command-palette")
                    .ghost()
                    .small()
                    .icon(IconName::Asterisk)
                    .tooltip("Command Palette · Ctrl+Shift+P")
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_palette(window, cx))),
            )
    }

    fn render_activity_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.workspace().sidebar.active_tool;
        let open = self.workspace().sidebar.open;
        v_flex()
            .w_11()
            .h_full()
            .flex_shrink_0()
            .items_center()
            .gap_1()
            .py_2()
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .children(
                SidebarTool::ALL
                    .into_iter()
                    .enumerate()
                    .map(|(index, tool)| {
                        let icon = match tool {
                            SidebarTool::Navigation => IconName::FolderClosed,
                            SidebarTool::Shelf => IconName::Inbox,
                            SidebarTool::Git => IconName::Github,
                            SidebarTool::Info => IconName::Info,
                            SidebarTool::Statistics => IconName::ChartPie,
                            SidebarTool::Search => IconName::Search,
                        };
                        Button::new(("sidebar-tool", index))
                            .ghost()
                            .small()
                            .icon(icon)
                            .tooltip(tool.label())
                            .when(open && active == tool, |button| button.primary())
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.show_sidebar_tool(tool, cx)),
                            )
                    }),
            )
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let tool = self.workspace().sidebar.active_tool;
        v_flex()
            .size_full()
            .min_w_0()
            .bg(cx.theme().sidebar)
            .border_r_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .h_10()
                    .flex_shrink_0()
                    .justify_between()
                    .px_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .font_semibold()
                    .child(tool.label())
                    .child(
                        Button::new("sidebar-close")
                            .ghost()
                            .xsmall()
                            .icon(IconName::PanelLeftClose)
                            .tooltip("Collapse Sidebar · Ctrl+B")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.state.workspace_mut().sidebar.open = false;
                                this.persist();
                                cx.notify();
                            })),
                    ),
            )
            .child(match tool {
                SidebarTool::Navigation => self.render_navigation_sidebar(cx),
                SidebarTool::Shelf => self.render_shelf_sidebar(cx),
                SidebarTool::Git => self.render_git_sidebar(cx),
                SidebarTool::Info => self.render_info_sidebar(cx),
                SidebarTool::Statistics => self.render_statistics_sidebar(cx),
                SidebarTool::Search => self.render_search_sidebar(cx),
            })
            .into_any_element()
    }

    fn sidebar_section(title: &str, cx: &Context<Self>) -> AnyElement {
        div()
            .px_3()
            .pt_3()
            .pb_1()
            .text_xs()
            .font_semibold()
            .text_color(cx.theme().muted_foreground)
            .child(title.to_string())
            .into_any_element()
    }

    fn render_navigation_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let active_path = self.active_path().to_path_buf();
        let workspaces = self
            .state
            .workspaces
            .iter()
            .enumerate()
            .map(|(index, workspace)| (index, workspace.name.clone()))
            .collect::<Vec<_>>();
        let mut recent = Vec::new();
        for panel in &self.workspace().panels {
            for path in panel.active_tab().history.iter().rev() {
                if !recent.contains(path) {
                    recent.push(path.clone());
                }
                if recent.len() >= 8 {
                    break;
                }
            }
        }
        v_flex()
            .flex_1()
            .min_h_0()
            .overflow_y_scrollbar()
            .child(Self::sidebar_section("WORKSPACES", cx))
            .children(workspaces.into_iter().map(|(index, name)| {
                let selected = index == self.state.active_workspace;
                h_flex()
                    .id(("workspace", index))
                    .h_8()
                    .mx_2()
                    .gap_2()
                    .px_2()
                    .rounded_md()
                    .cursor_pointer()
                    .when(selected, |this| this.bg(cx.theme().accent))
                    .hover(|this| this.bg(cx.theme().accent.opacity(0.6)))
                    .child(Icon::new(IconName::LayoutDashboard).small())
                    .child(name)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.select_workspace(index, window, cx)
                    }))
            }))
            .child(
                Button::new("save-workspace")
                    .ghost()
                    .xsmall()
                    .label("Save current as…")
                    .icon(IconName::Plus)
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.begin_name_action(
                            NameAction::SaveWorkspace,
                            format!("Workspace {}", this.state.workspaces.len() + 1),
                            window,
                            cx,
                        )
                    })),
            )
            .child(Self::sidebar_section("QUICK ACCESS", cx))
            .children(
                known_folders()
                    .into_iter()
                    .enumerate()
                    .map(|(index, (label, path))| {
                        let selected = active_path == path;
                        h_flex()
                            .id(("quick-access", index))
                            .h_8()
                            .mx_2()
                            .gap_2()
                            .px_2()
                            .rounded_md()
                            .cursor_pointer()
                            .when(selected, |this| this.bg(cx.theme().accent))
                            .hover(|this| this.bg(cx.theme().accent.opacity(0.6)))
                            .child(Icon::new(IconName::Folder).small())
                            .child(label)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.navigate_active(path.clone(), cx);
                                this.set_address_input(path.clone(), window, cx);
                            }))
                    }),
            )
            .child(Self::sidebar_section("DRIVES", cx))
            .children(
                ('A'..='Z')
                    .map(|drive| PathBuf::from(format!("{drive}:\\")))
                    .filter(|path| path.exists())
                    .enumerate()
                    .map(|(index, path)| {
                        let label = path.display().to_string();
                        h_flex()
                            .id(("drive", index))
                            .h_8()
                            .mx_2()
                            .gap_2()
                            .px_2()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|this| this.bg(cx.theme().accent.opacity(0.6)))
                            .child(Icon::new(IconName::HardDrive).small())
                            .child(label)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.navigate_active(path.clone(), cx);
                                this.set_address_input(path.clone(), window, cx);
                            }))
                    }),
            )
            .child(Self::sidebar_section("RECENT LOCATIONS", cx))
            .children(recent.into_iter().enumerate().map(|(index, path)| {
                let label = path
                    .file_name()
                    .unwrap_or(path.as_os_str())
                    .to_string_lossy()
                    .into_owned();
                h_flex()
                    .id(("recent", index))
                    .h_8()
                    .mx_2()
                    .gap_2()
                    .px_2()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|this| this.bg(cx.theme().accent.opacity(0.6)))
                    .child(Icon::new(IconName::Redo2).small())
                    .child(div().min_w_0().truncate().child(label))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.navigate_active(path.clone(), cx);
                        this.set_address_input(path.clone(), window, cx);
                    }))
            }))
            .into_any_element()
    }

    fn render_shelf_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let shelves = self
            .state
            .shelves
            .iter()
            .enumerate()
            .map(|(index, shelf)| (index, shelf.name.clone(), shelf.items.len()))
            .collect::<Vec<_>>();
        let items = self.state.shelf().items.clone();
        let persistent = self.state.shelf().persistent;
        v_flex()
            .flex_1()
            .min_h_0()
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_1()
                    .p_2()
                    .children(shelves.into_iter().map(|(index, name, count)| {
                        Button::new(("shelf", index))
                            .xsmall()
                            .label(format!("{name} · {count}"))
                            .when(index != self.state.active_shelf, |button| button.ghost())
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.select_shelf(index, cx)),
                            )
                    }))
                    .child(
                        Button::new("new-shelf")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Plus)
                            .tooltip("Create named Shelf")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.begin_name_action(
                                    NameAction::NewShelf,
                                    format!("Shelf {}", this.state.shelves.len() + 1),
                                    window,
                                    cx,
                                )
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_1()
                    .px_2()
                    .pb_2()
                    .flex_wrap()
                    .child(
                        Button::new("shelf-add")
                            .small()
                            .label("Add selection")
                            .icon(IconName::Plus)
                            .on_click(
                                cx.listener(|this, _, _, cx| this.add_selection_to_shelf(cx)),
                            ),
                    )
                    .child(
                        Button::new("shelf-persist")
                            .ghost()
                            .small()
                            .label(if persistent {
                                "Persistent"
                            } else {
                                "Session only"
                            })
                            .icon(if persistent {
                                IconName::Check
                            } else {
                                IconName::Dash
                            })
                            .on_click(
                                cx.listener(|this, _, _, cx| this.toggle_shelf_persistence(cx)),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .when(items.is_empty(), |this| {
                        this.items_center().justify_center().p_5().child(
                            v_flex()
                                .items_center()
                                .gap_2()
                                .text_center()
                                .text_color(cx.theme().muted_foreground)
                                .child(Icon::new(IconName::Inbox))
                                .child("Collect files from any panel here.")
                                .child("Select files, then use “Add selection”."),
                        )
                    })
                    .children(items.into_iter().enumerate().map(|(index, item)| {
                        let missing = !item.path.exists();
                        let name = item
                            .path
                            .file_name()
                            .unwrap_or(item.path.as_os_str())
                            .to_string_lossy()
                            .into_owned();
                        h_flex()
                            .id(("shelf-item", index))
                            .min_h_10()
                            .mx_2()
                            .gap_2()
                            .px_2()
                            .border_b_1()
                            .border_color(cx.theme().border.opacity(0.5))
                            .child(
                                Icon::new(if missing {
                                    IconName::TriangleAlert
                                } else {
                                    IconName::File
                                })
                                .small(),
                            )
                            .child(
                                v_flex()
                                    .min_w_0()
                                    .flex_1()
                                    .child(div().truncate().child(name))
                                    .child(
                                        div()
                                            .text_xs()
                                            .truncate()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(if missing {
                                                format!("Missing · {}", item.path.display())
                                            } else {
                                                item.path.display().to_string()
                                            }),
                                    ),
                            )
                            .child(
                                Button::new(("remove-shelf-item", index))
                                    .ghost()
                                    .xsmall()
                                    .icon(IconName::Close)
                                    .tooltip("Remove reference")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.remove_shelf_item(index, cx)
                                    })),
                            )
                    })),
            )
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_1()
                    .p_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        Button::new("shelf-copy")
                            .small()
                            .label("Copy here")
                            .disabled(self.state.shelf().items.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.shelf_transfer(OperationKind::Copy, cx)
                            })),
                    )
                    .child(
                        Button::new("shelf-move")
                            .ghost()
                            .small()
                            .label("Move here")
                            .disabled(self.state.shelf().items.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.shelf_transfer(OperationKind::Move, cx)
                            })),
                    )
                    .child(
                        Button::new("shelf-zip")
                            .ghost()
                            .small()
                            .label("Zip")
                            .disabled(self.state.shelf().items.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| this.compress_shelf(cx))),
                    )
                    .child(
                        Button::new("shelf-rename")
                            .ghost()
                            .small()
                            .label("Rename")
                            .disabled(self.state.shelf().items.len() < 2)
                            .on_click(cx.listener(|this, _, window, cx| {
                                let sources = this
                                    .state
                                    .shelf()
                                    .items
                                    .iter()
                                    .filter(|item| item.path.exists())
                                    .map(|item| item.path.clone())
                                    .collect();
                                this.begin_name_action(
                                    NameAction::BatchRename(sources),
                                    "renamed_".to_string(),
                                    window,
                                    cx,
                                )
                            })),
                    )
                    .child(
                        Button::new("shelf-copy-paths")
                            .ghost()
                            .small()
                            .label("Paths")
                            .disabled(self.state.shelf().items.is_empty())
                            .on_click(cx.listener(|this, _, _, cx| this.copy_shelf_paths(cx))),
                    ),
            )
            .into_any_element()
    }

    fn render_context_loading(&self, label: &str, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .flex_1()
            .items_center()
            .justify_center()
            .gap_3()
            .text_color(cx.theme().muted_foreground)
            .child(Icon::new(IconName::LoaderCircle))
            .child(label.to_string())
            .child(
                Button::new("cancel-context")
                    .ghost()
                    .small()
                    .label("Cancel")
                    .on_click(cx.listener(|this, _, _, cx| this.cancel_context_work(cx))),
            )
            .into_any_element()
    }

    fn render_git_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        match &self.git_state {
            ContextState::Idle => v_flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_color(cx.theme().muted_foreground)
                .child("Git information loads when this tab opens.")
                .into_any_element(),
            ContextState::Loading(path) => self.render_context_loading(
                &format!("Reading Git history for {}…", path.display()),
                cx,
            ),
            ContextState::Error(path, error) => v_flex()
                .flex_1()
                .items_center()
                .justify_center()
                .gap_2()
                .p_4()
                .text_center()
                .child(Icon::new(IconName::TriangleAlert))
                .child(format!("Could not read Git data for {}", path.display()))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(error.clone()),
                )
                .into_any_element(),
            ContextState::Ready(None) => v_flex()
                .flex_1()
                .items_center()
                .justify_center()
                .gap_2()
                .p_4()
                .text_center()
                .text_color(cx.theme().muted_foreground)
                .child(Icon::new(IconName::Github))
                .child("The active folder is not inside a Git repository.")
                .into_any_element(),
            ContextState::Ready(Some(repository)) => {
                let root = repository.root.clone();
                let commits = repository.commits.clone();
                let changed = repository.changed_files.clone();
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .child(
                        v_flex()
                            .gap_1()
                            .p_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                h_flex()
                                    .gap_2()
                                    .font_semibold()
                                    .child(Icon::new(IconName::Github).small())
                                    .child(repository.branch.clone()),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(repository.root.display().to_string()),
                            )
                            .child(format!("{} changed file(s)", changed.len())),
                    )
                    .child(Self::sidebar_section("CHANGES", cx))
                    .children(
                        changed
                            .into_iter()
                            .take(10)
                            .enumerate()
                            .map(|(index, relative)| {
                                let path = root.join(&relative);
                                h_flex()
                                    .id(("git-change", index))
                                    .h_7()
                                    .mx_2()
                                    .gap_2()
                                    .px_2()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .hover(|this| this.bg(cx.theme().accent.opacity(0.6)))
                                    .child("M")
                                    .child(div().truncate().child(relative.display().to_string()))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        if let Some(parent) = path.parent() {
                                            this.navigate_active(parent.to_path_buf(), cx);
                                            this.set_address_input(
                                                parent.to_path_buf(),
                                                window,
                                                cx,
                                            );
                                            if let Some(runtime) =
                                                this.panels.get_mut(&this.active_panel_id())
                                            {
                                                runtime.selected.insert(path.clone());
                                            }
                                        }
                                    }))
                            }),
                    )
                    .child(Self::sidebar_section("RECENT COMMITS", cx))
                    .child(v_flex().flex_1().min_h_0().overflow_y_scrollbar().children(
                        commits.into_iter().enumerate().map(|(index, commit)| {
                            let hash = commit.hash.clone();
                            let root = root.clone();
                            let changed_files = commit.changed_files.clone();
                            v_flex()
                                .id(("commit", index))
                                .gap_1()
                                .px_3()
                                .py_2()
                                .border_b_1()
                                .border_color(cx.theme().border.opacity(0.5))
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(commit.short_hash),
                                        )
                                        .child(
                                            div()
                                                .min_w_0()
                                                .flex_1()
                                                .truncate()
                                                .child(commit.message),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("{} · {}", commit.author, commit.unix_time)),
                                )
                                .when(!changed_files.is_empty(), |this| {
                                    this.children(changed_files.into_iter().take(8).map(|path| {
                                        div()
                                            .pl_8()
                                            .text_xs()
                                            .truncate()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(path.display().to_string())
                                    }))
                                })
                                .cursor_pointer()
                                .hover(|this| this.bg(cx.theme().accent.opacity(0.5)))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.load_commit_files(root.clone(), hash.clone(), cx)
                                }))
                        }),
                    ))
                    .into_any_element()
            }
        }
    }

    fn render_info_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        match &self.info_state {
            ContextState::Idle => div()
                .child("Folder information is not loaded.")
                .into_any_element(),
            ContextState::Loading(path) => {
                self.render_context_loading(&format!("Calculating {}…", path.display()), cx)
            }
            ContextState::Error(path, error) => v_flex()
                .gap_2()
                .p_4()
                .child(format!("Could not inspect {}", path.display()))
                .child(error.clone())
                .into_any_element(),
            ContextState::Ready(info) => {
                let path = info.path.clone();
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .gap_3()
                    .p_3()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div().font_semibold().child(
                                    path.file_name()
                                        .unwrap_or(path.as_os_str())
                                        .to_string_lossy()
                                        .into_owned(),
                                ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(path.display().to_string()),
                            ),
                    )
                    .child(info_row("Files", info.file_count.to_string(), cx))
                    .child(info_row("Folders", info.folder_count.to_string(), cx))
                    .child(info_row(
                        "Total size",
                        format_size(info.total_size, BINARY),
                        cx,
                    ))
                    .child(info_row("Created", format_system_time(info.created), cx))
                    .child(info_row("Modified", format_system_time(info.modified), cx))
                    .child(info_row(
                        "Attributes",
                        format!(
                            "{}{}",
                            if info.readonly { "Read-only " } else { "" },
                            if info.hidden { "Hidden" } else { "Visible" }
                        ),
                        cx,
                    ))
                    .child(info_row(
                        "Access errors",
                        info.inaccessible_count.to_string(),
                        cx,
                    ))
                    .when(info.cancelled, |this| {
                        this.child(
                            div()
                                .text_color(cx.theme().warning)
                                .child("Calculation was cancelled; results are partial."),
                        )
                    })
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .child(
                                Button::new("copy-win-path")
                                    .small()
                                    .label("Windows")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_formatted_paths(PathFormat::Windows, cx)
                                    })),
                            )
                            .child(
                                Button::new("copy-ps-path")
                                    .ghost()
                                    .small()
                                    .label("PowerShell")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_formatted_paths(PathFormat::PowerShell, cx)
                                    })),
                            )
                            .child(
                                Button::new("copy-uri-path")
                                    .ghost()
                                    .small()
                                    .label("URI")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_formatted_paths(PathFormat::FileUri, cx)
                                    })),
                            )
                            .child(
                                Button::new("copy-wsl-path")
                                    .ghost()
                                    .small()
                                    .label("WSL")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.copy_formatted_paths(PathFormat::Wsl, cx)
                                    })),
                            ),
                    )
                    .into_any_element()
            }
        }
    }

    fn render_statistics_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        match &self.statistics_state {
            ContextState::Idle => div()
                .child("Folder statistics are not loaded.")
                .into_any_element(),
            ContextState::Loading(path) => {
                self.render_context_loading(&format!("Analyzing {}…", path.display()), cx)
            }
            ContextState::Error(path, error) => v_flex()
                .gap_2()
                .p_4()
                .child(format!("Could not analyze {}", path.display()))
                .child(error.clone())
                .into_any_element(),
            ContextState::Ready(statistics) => {
                let types = statistics.types.clone();
                let largest = statistics.largest.clone();
                let empty_count = statistics.empty_folders.len();
                let duplicate_count = statistics.duplicate_candidates.len();
                let duplicates = statistics.duplicate_candidates.clone();
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .child(
                        h_flex()
                            .gap_2()
                            .p_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(format!("{} entries scanned", statistics.scanned))
                            .when(statistics.cancelled, |this| this.child("· partial")),
                    )
                    .child(Self::sidebar_section("FILE TYPES BY SIZE", cx))
                    .children(
                        types
                            .into_iter()
                            .take(12)
                            .enumerate()
                            .map(|(index, statistic)| {
                                h_flex()
                                    .id(("type-statistic", index))
                                    .h_7()
                                    .mx_3()
                                    .justify_between()
                                    .child(format!(
                                        ".{} · {}",
                                        statistic.extension, statistic.count
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format_size(statistic.total_size, BINARY)),
                                    )
                            }),
                    )
                    .child(Self::sidebar_section("LARGEST FILES", cx))
                    .children(
                        largest
                            .into_iter()
                            .take(10)
                            .enumerate()
                            .map(|(index, entry)| {
                                let path = entry.path.clone();
                                let size_label = entry.size_label();
                                h_flex()
                                    .id(("largest", index))
                                    .h_8()
                                    .mx_2()
                                    .gap_2()
                                    .px_2()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .hover(|this| this.bg(cx.theme().accent.opacity(0.6)))
                                    .child(div().min_w_0().flex_1().truncate().child(entry.name))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(size_label),
                                    )
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        if let Some(parent) = path.parent() {
                                            this.navigate_active(parent.to_path_buf(), cx);
                                            this.set_address_input(
                                                parent.to_path_buf(),
                                                window,
                                                cx,
                                            );
                                            if let Some(runtime) =
                                                this.panels.get_mut(&this.active_panel_id())
                                            {
                                                runtime.selected.insert(path.clone());
                                            }
                                        }
                                    }))
                            }),
                    )
                    .child(Self::sidebar_section("CLEANUP CANDIDATES", cx))
                    .child(
                        v_flex()
                            .gap_1()
                            .px_3()
                            .pb_3()
                            .child(format!("{empty_count} empty folder(s)"))
                            .child(format!("{duplicate_count} possible duplicate group(s)"))
                            .child(format!("{} access error(s)", statistics.inaccessible)),
                    )
                    .children(duplicates.into_iter().take(8).enumerate().map(
                        |(index, candidate)| {
                            let first = candidate
                                .paths
                                .first()
                                .map(|path| path.display().to_string())
                                .unwrap_or_default();
                            v_flex()
                                .id(("duplicate-candidate", index))
                                .mx_3()
                                .mb_2()
                                .gap_1()
                                .child(format!(
                                    "{} matching files · {} each",
                                    candidate.paths.len(),
                                    format_size(candidate.size, BINARY)
                                ))
                                .child(
                                    div()
                                        .truncate()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(first),
                                )
                        },
                    ))
                    .into_any_element()
            }
        }
    }

    fn render_search_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let query = self.filter_input.read(cx).value().to_string();
        let saved = self.state.saved_searches.clone();
        v_flex()
            .flex_1()
            .min_h_0()
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(Input::new(&self.filter_input).small().cleanable(true))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Filters: ext:pdf · glob:**/*.rs · min:10mb · max:1gb · hidden:true"),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new("run-search")
                                    .small()
                                    .label("Search all panels")
                                    .icon(IconName::Search)
                                    .on_click(cx.listener(|this, _, _, cx| this.run_search(cx))),
                            )
                            .child(
                                Button::new("save-search")
                                    .ghost()
                                    .small()
                                    .label("Save")
                                    .disabled(query.trim().is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let query =
                                            this.filter_input.read(cx).value().to_string();
                                        this.state.saved_searches.push(SavedSearch {
                                            name: query.clone(),
                                            query,
                                            ..SavedSearch::default()
                                        });
                                        this.persist();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .when(!saved.is_empty(), |this| {
                        this.child(
                            h_flex()
                                .flex_wrap()
                                .gap_1()
                                .children(saved.into_iter().enumerate().map(
                                    |(index, saved)| {
                                        Button::new(("saved-search", index))
                                            .ghost()
                                            .xsmall()
                                            .label(saved.name)
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.filter_input.update(cx, |input, cx| {
                                                    input.set_value(
                                                        saved.query.clone(),
                                                        window,
                                                        cx,
                                                    )
                                                });
                                                this.run_search(cx);
                                            }))
                                    },
                                )),
                        )
                    }),
            )
            .child(match &self.search_results {
                ContextState::Idle => v_flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .text_color(cx.theme().muted_foreground)
                    .child(Icon::new(IconName::Search))
                    .child("Search the active folder or every open panel.")
                    .into_any_element(),
                ContextState::Loading(path) => self.render_context_loading(
                    &format!("Searching from {} and other panels…", path.display()),
                    cx,
                ),
                ContextState::Error(_, error) => v_flex()
                    .p_4()
                    .child("Search failed")
                    .child(error.clone())
                    .into_any_element(),
                ContextState::Ready(results) => {
                    let results = results.clone();
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .child(
                            div()
                                .px_3()
                                .py_2()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{} result(s)", results.len())),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scrollbar()
                                .children(results.into_iter().take(500).enumerate().map(
                                    |(index, result)| {
                                        let path = result.entry.path.clone();
                                        h_flex()
                                            .id(("search-result", index))
                                            .min_h_10()
                                            .mx_2()
                                            .gap_2()
                                            .px_2()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .hover(|this| {
                                                this.bg(cx.theme().accent.opacity(0.6))
                                            })
                                            .child(Icon::new(if result.entry.is_directory() {
                                                IconName::Folder
                                            } else {
                                                IconName::File
                                            }).small())
                                            .child(
                                                v_flex()
                                                    .min_w_0()
                                                    .flex_1()
                                                    .child(
                                                        div()
                                                            .truncate()
                                                            .child(result.entry.name),
                                                    )
                                                    .child(
                                                        div()
                                                            .truncate()
                                                            .text_xs()
                                                            .text_color(
                                                                cx.theme().muted_foreground,
                                                            )
                                                            .child(
                                                                result.root.display().to_string(),
                                                            ),
                                                    ),
                                            )
                                            .on_click(cx.listener(
                                                move |this, _, window, cx| {
                                                    if path.is_dir() {
                                                        this.navigate_active(path.clone(), cx);
                                                        this.set_address_input(
                                                            path.clone(),
                                                            window,
                                                            cx,
                                                        );
                                                    } else if let Some(parent) = path.parent() {
                                                        this.navigate_active(
                                                            parent.to_path_buf(),
                                                            cx,
                                                        );
                                                        this.set_address_input(
                                                            parent.to_path_buf(),
                                                            window,
                                                            cx,
                                                        );
                                                        if let Some(runtime) = this
                                                            .panels
                                                            .get_mut(&this.active_panel_id())
                                                        {
                                                            runtime
                                                                .selected
                                                                .insert(path.clone());
                                                        }
                                                    }
                                                },
                                            ))
                                    },
                                )),
                        )
                        .into_any_element()
                }
            })
            .into_any_element()
    }

    fn render_split_node(&mut self, node: SplitNode, cx: &mut Context<Self>) -> AnyElement {
        match node {
            SplitNode::Panel(panel_id) => self.render_panel(panel_id, cx),
            SplitNode::Split {
                id,
                axis,
                ratio,
                first,
                second,
            } => {
                let first = self.render_split_node(*first, cx);
                let second = self.render_split_node(*second, cx);
                let state = self.split_states.get(&id).cloned();
                let initial = match axis {
                    SplitAxis::Columns => px(900. * ratio),
                    SplitAxis::Rows => px(560. * ratio),
                };
                let first_panel = resizable_panel().size(initial).child(first);
                let second_panel = resizable_panel().child(second);
                match (axis, state) {
                    (SplitAxis::Columns, Some(state)) => h_resizable(("split-columns", id))
                        .with_state(&state)
                        .child(first_panel)
                        .child(second_panel)
                        .into_any_element(),
                    (SplitAxis::Rows, Some(state)) => v_resizable(("split-rows", id))
                        .with_state(&state)
                        .child(first_panel)
                        .child(second_panel)
                        .into_any_element(),
                    (SplitAxis::Columns, None) => h_resizable(("split-columns", id))
                        .child(first_panel)
                        .child(second_panel)
                        .into_any_element(),
                    (SplitAxis::Rows, None) => v_resizable(("split-rows", id))
                        .child(first_panel)
                        .child(second_panel)
                        .into_any_element(),
                }
            }
        }
    }

    fn render_panel(&self, panel_id: PanelId, cx: &mut Context<Self>) -> AnyElement {
        let Some(panel) = self.workspace().panel(panel_id) else {
            return div().into_any_element();
        };
        let active = self.active_panel_id() == panel_id;
        let target = self.workspace().target_panel == Some(panel_id);
        let tabs = panel.tabs.clone();
        let active_tab = panel.active_tab;
        let panel_count = self.workspace().panels.len();
        let runtime = self.panels.get(&panel_id);
        let entries = runtime
            .map(|runtime| runtime.entries.clone())
            .unwrap_or_default();
        let selected = runtime
            .map(|runtime| runtime.selected.clone())
            .unwrap_or_default();
        let loading = runtime.is_some_and(|runtime| runtime.loading);
        let error = runtime.and_then(|runtime| runtime.error.clone());
        let scroll = runtime
            .map(|runtime| runtime.scroll.clone())
            .unwrap_or_default();
        let drag_paths = if selected.is_empty() {
            Vec::new()
        } else {
            selected.iter().cloned().collect::<Vec<_>>()
        };
        let entry_count = entries.len();
        let selected_count = selected.len();
        let border_color = if active {
            cx.theme().primary
        } else if target {
            cx.theme().warning
        } else {
            cx.theme().border
        };

        v_flex()
            .id(("panel", panel_id))
            .size_full()
            .min_w_0()
            .border_2()
            .border_color(border_color)
            .bg(cx.theme().background)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, _, window, cx| this.activate_panel(panel_id, window, cx)),
            )
            .on_drop(
                cx.listener(move |this, dragged: &DraggedFiles, window, cx| {
                    if dragged.source_panel == panel_id {
                        return;
                    }
                    let kind = if window.modifiers().shift {
                        OperationKind::Move
                    } else {
                        OperationKind::Copy
                    };
                    if let Some(destination) = this
                        .workspace()
                        .panel(panel_id)
                        .map(|panel| panel.active_tab().path.clone())
                    {
                        this.prepare_transfer(kind, dragged.paths.clone(), destination, cx);
                    }
                }),
            )
            .child(
                h_flex()
                    .h_9()
                    .flex_shrink_0()
                    .gap_1()
                    .px_1()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().sidebar)
                    .children(tabs.into_iter().enumerate().map(|(index, tab)| {
                        let label = tab
                            .path
                            .file_name()
                            .unwrap_or(tab.path.as_os_str())
                            .to_string_lossy()
                            .into_owned();
                        h_flex()
                            .id(("panel-tab", panel_id as usize * 100 + index))
                            .h_7()
                            .max_w(px(180.))
                            .gap_1()
                            .px_2()
                            .rounded_md()
                            .cursor_pointer()
                            .when(index == active_tab, |this| this.bg(cx.theme().accent))
                            .hover(|this| this.bg(cx.theme().accent.opacity(0.6)))
                            .when(tab.pinned, |this| {
                                this.child(Icon::new(IconName::StarFill).xsmall())
                            })
                            .child(div().truncate().child(label))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.switch_tab(panel_id, index, window, cx)
                            }))
                    }))
                    .child(
                        Button::new(("new-panel-tab", panel_id))
                            .ghost()
                            .xsmall()
                            .icon(IconName::Plus)
                            .tooltip("New tab · Ctrl+T")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.new_tab(Some(window), cx)),
                            ),
                    )
                    .child(div().flex_1())
                    .when(active, |this| {
                        this.child(
                            div()
                                .px_2()
                                .text_xs()
                                .font_semibold()
                                .text_color(cx.theme().primary)
                                .child("ACTIVE"),
                        )
                    })
                    .when(target, |this| {
                        this.child(
                            div()
                                .px_2()
                                .text_xs()
                                .font_semibold()
                                .text_color(cx.theme().warning)
                                .child("TARGET"),
                        )
                    })
                    .child(
                        Button::new(("target-panel", panel_id))
                            .ghost()
                            .xsmall()
                            .icon(IconName::Frame)
                            .tooltip("Designate as transfer target")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.state.workspace_mut().target_panel = Some(panel_id);
                                this.persist();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new(("pin-tab", panel_id))
                            .ghost()
                            .xsmall()
                            .icon(IconName::Star)
                            .tooltip("Pin or unpin active tab")
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_pin_active_tab(cx))),
                    )
                    .child(
                        Button::new(("close-panel", panel_id))
                            .ghost()
                            .xsmall()
                            .icon(IconName::Close)
                            .tooltip("Close panel")
                            .disabled(panel_count <= 1)
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.close_panel(panel_id, cx)),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .h_8()
                    .flex_shrink_0()
                    .px_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().sidebar)
                    .text_xs()
                    .font_semibold()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        div()
                            .id(("sort-name", panel_id))
                            .min_w_0()
                            .flex_1()
                            .cursor_pointer()
                            .child("Name")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_sort(crate::model::SortField::Name, cx)
                            })),
                    )
                    .when(panel_count <= 2, |this| {
                        this.child(
                            div()
                                .id(("sort-modified", panel_id))
                                .w(px(112.))
                                .cursor_pointer()
                                .child("Modified")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_sort(crate::model::SortField::Modified, cx)
                                })),
                        )
                    })
                    .when(panel_count == 1, |this| {
                        this.child(
                            div()
                                .id(("sort-kind", panel_id))
                                .w_24()
                                .cursor_pointer()
                                .child("Type")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_sort(crate::model::SortField::Kind, cx)
                                })),
                        )
                    })
                    .child(
                        div()
                            .id(("sort-size", panel_id))
                            .w_20()
                            .text_right()
                            .cursor_pointer()
                            .child("Size")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_sort(crate::model::SortField::Size, cx)
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .relative()
                    .when(entries.is_empty() && !loading && error.is_none(), |this| {
                        this.child(
                            v_flex()
                                .absolute()
                                .inset_0()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .text_color(cx.theme().muted_foreground)
                                .child(Icon::new(IconName::FolderOpen))
                                .child("This folder is empty")
                                .child("Drop files here or create a new folder."),
                        )
                    })
                    .when_some(error.clone(), |this, error| {
                        this.child(
                            v_flex()
                                .absolute()
                                .inset_0()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .p_4()
                                .text_center()
                                .child(Icon::new(IconName::TriangleAlert))
                                .child("This location is unavailable")
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(error),
                                )
                                .child(
                                    Button::new(("retry-panel", panel_id))
                                        .small()
                                        .label("Try again")
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.schedule_panel_load(panel_id, cx)
                                        })),
                                ),
                        )
                    })
                    .when(loading, |this| {
                        this.child(
                            h_flex()
                                .absolute()
                                .top_2()
                                .right_3()
                                .gap_2()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(cx.theme().sidebar)
                                .text_xs()
                                .child(Icon::new(IconName::LoaderCircle).xsmall())
                                .child("Loading…"),
                        )
                    })
                    .child(
                        uniform_list(
                            ("panel-entries", panel_id),
                            entries.len(),
                            cx.processor(
                                move |_this, range: std::ops::Range<usize>, _window, cx| {
                                    range
                                        .map(|index| {
                                            let entry = entries[index].clone();
                                            let path = entry.path.clone();
                                            let is_selected = selected.contains(&path);
                                            let icon = if entry.is_directory() {
                                                IconName::Folder
                                            } else {
                                                IconName::File
                                            };
                                            let modified = entry.modified_label();
                                            let kind = entry.kind_label();
                                            let size = entry.size_label();
                                            let drag = DraggedFiles {
                                                paths: if drag_paths.contains(&path) {
                                                    drag_paths.clone()
                                                } else {
                                                    vec![path.clone()]
                                                },
                                                source_panel: panel_id,
                                            };
                                            h_flex()
                                                .id((
                                                    "file-entry",
                                                    panel_id as usize * 1_000_000 + index,
                                                ))
                                                .w_full()
                                                .h_9()
                                                .px_3()
                                                .border_b_1()
                                                .border_color(cx.theme().border.opacity(0.4))
                                                .cursor_pointer()
                                                .when(is_selected, |this| {
                                                    this.bg(cx.theme().accent)
                                                })
                                                .hover(|this| {
                                                    this.bg(cx.theme().accent.opacity(0.55))
                                                })
                                                .child(
                                                    h_flex()
                                                        .min_w_0()
                                                        .flex_1()
                                                        .gap_2()
                                                        .child(Icon::new(icon).small())
                                                        .when(entry.hidden, |this| {
                                                            this.opacity(0.65)
                                                        })
                                                        .child(
                                                            div()
                                                                .truncate()
                                                                .child(entry.name.clone()),
                                                        ),
                                                )
                                                .when(entry.readonly, |this| {
                                                    this.child(
                                                        div()
                                                            .mr_2()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child("READ-ONLY"),
                                                    )
                                                })
                                                .when(panel_count <= 2, |this| {
                                                    this.child(
                                                        div()
                                                            .w(px(112.))
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(modified),
                                                    )
                                                })
                                                .when(panel_count == 1, |this| {
                                                    this.child(
                                                        div()
                                                            .w_24()
                                                            .truncate()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(kind),
                                                    )
                                                })
                                                .child(
                                                    div()
                                                        .w_20()
                                                        .text_right()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(size),
                                                )
                                                .on_drag(drag, |drag: &DraggedFiles, _, _, cx| {
                                                    cx.new(|_| drag.clone())
                                                })
                                                .on_click(cx.listener(
                                                    move |this, event: &ClickEvent, window, cx| {
                                                        if event.click_count() >= 2 {
                                                            this.activate_entry(
                                                                panel_id,
                                                                entry.clone(),
                                                                window,
                                                                cx,
                                                            );
                                                        } else {
                                                            this.select_entry(
                                                                panel_id, index, window, cx,
                                                            );
                                                        }
                                                    },
                                                ))
                                        })
                                        .collect()
                                },
                            ),
                        )
                        .track_scroll(&scroll)
                        .size_full(),
                    ),
            )
            .child(
                h_flex()
                    .h_7()
                    .flex_shrink_0()
                    .justify_between()
                    .px_3()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().sidebar)
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{} item(s) · {} selected",
                        entry_count, selected_count
                    ))
                    .child(if target {
                        "Drop: copy · Shift+drop: move"
                    } else {
                        "Enter open · Space preview"
                    }),
            )
            .into_any_element()
    }

    fn render_preview_content(&self, cx: &mut Context<Self>) -> AnyElement {
        let header = h_flex()
            .h_10()
            .flex_shrink_0()
            .justify_between()
            .gap_2()
            .px_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .font_semibold()
            .child("Quick Look")
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Button::new("preview-open")
                            .ghost()
                            .xsmall()
                            .icon(IconName::ExternalLink)
                            .tooltip("Open with associated application")
                            .on_click(cx.listener(|this, _, _, cx| this.open_previewed_file(cx))),
                    )
                    .child(
                        Button::new("preview-pin")
                            .ghost()
                            .xsmall()
                            .icon(if self.preview_pinned {
                                IconName::PanelRightClose
                            } else {
                                IconName::PanelRightOpen
                            })
                            .tooltip("Pin preview as a side panel")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.preview_pinned = !this.preview_pinned;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("preview-close")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Close)
                            .tooltip("Close · Space or Escape")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.preview_visible = false;
                                this.preview_pinned = false;
                                cx.notify();
                            })),
                    ),
            );
        let body = match &self.preview {
            Some(PreviewState::Loading(path)) => v_flex()
                .flex_1()
                .items_center()
                .justify_center()
                .gap_2()
                .child(Icon::new(IconName::LoaderCircle))
                .child(format!("Generating preview for {}…", path.display()))
                .into_any_element(),
            Some(PreviewState::Error(path, error)) => v_flex()
                .flex_1()
                .items_center()
                .justify_center()
                .gap_2()
                .p_5()
                .text_center()
                .child(Icon::new(IconName::TriangleAlert))
                .child(format!("Could not preview {}", path.display()))
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(error.clone()),
                )
                .into_any_element(),
            Some(PreviewState::Ready(preview)) => {
                let metadata = h_flex()
                    .flex_shrink_0()
                    .gap_3()
                    .px_3()
                    .py_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .truncate()
                            .child(preview.path.display().to_string()),
                    )
                    .child(format_size(preview.size, BINARY))
                    .child(format_system_time(preview.modified));
                let content = match &preview.content {
                    PreviewContent::Image { width, height } => v_flex()
                        .flex_1()
                        .min_h_0()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .p_4()
                        .child(
                            img(preview.path.clone())
                                .max_w_full()
                                .max_h_full()
                                .object_fit(ObjectFit::Contain),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{width} × {height}")),
                        )
                        .into_any_element(),
                    PreviewContent::Text { content, truncated } => v_flex()
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scrollbar()
                        .p_4()
                        .font_family("Consolas")
                        .text_sm()
                        .child(div().child(content.clone()))
                        .when(*truncated, |this| {
                            this.child(
                                div()
                                    .mt_3()
                                    .text_color(cx.theme().warning)
                                    .child("Preview truncated at 512 KiB."),
                            )
                        })
                        .into_any_element(),
                    PreviewContent::Pdf => preview_placeholder(
                        IconName::File,
                        "PDF",
                        "Open the file to use the system PDF renderer.",
                        cx,
                    ),
                    PreviewContent::Video => preview_placeholder(
                        IconName::Play,
                        "Video",
                        "Video metadata is available; playback uses the associated player.",
                        cx,
                    ),
                    PreviewContent::OfficeDocument => preview_placeholder(
                        IconName::File,
                        "Office document",
                        "Office previews use the associated application in Nimbus v1.0.",
                        cx,
                    ),
                    PreviewContent::MetadataOnly { reason } => {
                        preview_placeholder(IconName::Eye, &preview.name, reason, cx)
                    }
                };
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .child(content)
                    .child(metadata)
                    .into_any_element()
            }
            None => div().child("No preview").into_any_element(),
        };
        v_flex()
            .size_full()
            .min_h_0()
            .bg(cx.theme().background)
            .child(header)
            .child(body)
            .into_any_element()
    }

    fn render_operation_dialog(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(plan) = &self.pending_plan else {
            return div().into_any_element();
        };
        let items = plan.items.clone();
        let kind = plan.kind;
        let resolution = plan.resolution;
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui::black().opacity(0.58))
            .child(
                v_flex()
                    .w(px(680.))
                    .max_h(px(560.))
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .shadow_lg()
                    .child(
                        v_flex()
                            .gap_1()
                            .p_4()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .child(format!("Review {}", kind.label())),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{} item(s) · {} conflict(s) · {} will run",
                                        items.len(),
                                        plan.conflict_count(),
                                        plan.executable_count()
                                    )),
                            ),
                    )
                    .when(plan.conflict_count() > 0, |this| {
                        this.child(
                            h_flex()
                                .gap_2()
                                .p_3()
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .child("For conflicts:")
                                .children(
                                    [
                                        ConflictResolution::Rename,
                                        ConflictResolution::Skip,
                                        ConflictResolution::Overwrite,
                                    ]
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, option)| {
                                        Button::new(("conflict-resolution", index))
                                            .small()
                                            .label(option.label())
                                            .when(option != resolution, |button| button.ghost())
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.set_pending_resolution(option, cx)
                                            }))
                                    }),
                                ),
                        )
                    })
                    .child(v_flex().flex_1().min_h_0().overflow_y_scrollbar().children(
                        items.into_iter().enumerate().map(|(index, item)| {
                            let source = item
                                .source
                                .as_deref()
                                .map(|path| path.display().to_string())
                                .unwrap_or_else(|| "(new item)".to_string());
                            let destination = item
                                .destination
                                .as_deref()
                                .map(|path| path.display().to_string())
                                .unwrap_or_else(|| "Recycle Bin".to_string());
                            h_flex()
                                .id(("planned-item", index))
                                .min_h_12()
                                .gap_3()
                                .px_4()
                                .py_2()
                                .border_b_1()
                                .border_color(cx.theme().border.opacity(0.5))
                                .child(
                                    Icon::new(if item.conflict {
                                        IconName::TriangleAlert
                                    } else {
                                        IconName::ArrowRight
                                    })
                                    .small(),
                                )
                                .child(
                                    v_flex()
                                        .min_w_0()
                                        .flex_1()
                                        .child(div().truncate().child(source))
                                        .child(
                                            div()
                                                .truncate()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!("→ {destination}")),
                                        ),
                                )
                                .when(item.skipped, |this| {
                                    this.child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().warning)
                                            .child("SKIP"),
                                    )
                                })
                        }),
                    ))
                    .child(
                        h_flex()
                            .justify_end()
                            .gap_2()
                            .p_4()
                            .border_t_1()
                            .border_color(cx.theme().border)
                            .child(Button::new("cancel-plan").ghost().label("Cancel").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.pending_plan = None;
                                    this.dialog = DialogState::None;
                                    cx.notify();
                                }),
                            ))
                            .child(
                                Button::new("confirm-plan")
                                    .label("Queue operation")
                                    .disabled(plan.executable_count() == 0)
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.confirm_operation(cx)),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_palette(&self, cx: &mut Context<Self>) -> AnyElement {
        let commands = self.matching_commands(cx);
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_start()
            .justify_center()
            .pt(px(90.))
            .bg(gpui::black().opacity(0.48))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.dialog = DialogState::None;
                    cx.notify();
                }),
            )
            .child(
                v_flex()
                    .w(px(640.))
                    .h(px(520.))
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .shadow_lg()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .p_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                Input::new(&self.palette_input)
                                    .large()
                                    .prefix(Icon::new(IconName::Asterisk))
                                    .cleanable(true),
                            ),
                    )
                    .child(v_flex().flex_1().min_h_0().overflow_y_scrollbar().children(
                        commands.into_iter().enumerate().map(|(index, command)| {
                            let id = command.id;
                            let enabled = self.command_enabled(id);
                            h_flex()
                                .id(("palette-command", index))
                                .min_h_12()
                                .gap_3()
                                .px_4()
                                .py_2()
                                .cursor_pointer()
                                .when(!enabled, |this| this.opacity(0.45))
                                .hover(|this| this.bg(cx.theme().accent.opacity(0.6)))
                                .child(
                                    v_flex().min_w_0().flex_1().child(command.name).child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(command.description),
                                    ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(command.shortcut),
                                )
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.execute_command(id, Some(window), cx)
                                }))
                        }),
                    ))
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .border_t_1()
                            .border_color(cx.theme().border)
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Enter runs the first match · Escape closes"),
                    ),
            )
            .into_any_element()
    }

    fn render_name_dialog(&self, cx: &mut Context<Self>) -> AnyElement {
        let title = match self.name_action {
            Some(NameAction::Rename(_)) => "Rename item",
            Some(NameAction::BatchRename(_)) => "Batch rename — add prefix",
            Some(NameAction::CreateFolder(_)) => "Create folder",
            Some(NameAction::SaveWorkspace) => "Save workspace",
            Some(NameAction::NewShelf) => "Create Shelf",
            None => "Enter a name",
        };
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui::black().opacity(0.55))
            .child(
                v_flex()
                    .w(px(440.))
                    .gap_3()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .shadow_lg()
                    .child(div().text_lg().font_semibold().child(title))
                    .child(Input::new(&self.name_input).cleanable(true))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Press Enter to review or save. Windows name rules apply."),
                    )
                    .child(
                        h_flex()
                            .justify_end()
                            .gap_2()
                            .child(Button::new("cancel-name").ghost().label("Cancel").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.dialog = DialogState::None;
                                    this.name_action = None;
                                    cx.notify();
                                }),
                            ))
                            .child(Button::new("apply-name").label("Continue").on_click(
                                cx.listener(|this, _, _, cx| {
                                    let value = this.name_input.read(cx).value().to_string();
                                    this.apply_name_action(&value, cx)
                                }),
                            )),
                    ),
            )
            .into_any_element()
    }

    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.selected_paths();
        let selected_size = selected
            .iter()
            .filter_map(|path| std::fs::metadata(path).ok())
            .filter(|metadata| metadata.is_file())
            .map(|metadata| metadata.len())
            .sum::<u64>();
        let latest = self.queue.jobs.last().map(|job| job.summary.clone());
        let running = self.queue.active.is_some();
        h_flex()
            .h_8()
            .flex_shrink_0()
            .justify_between()
            .gap_3()
            .px_3()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(
                h_flex()
                    .gap_3()
                    .child(match &self.message {
                        Some(message) => message.clone(),
                        None if selected.is_empty() => {
                            format!("{} · Ready", self.active_path().display())
                        }
                        None => format!(
                            "{} selected · {}",
                            selected.len(),
                            format_size(selected_size, BINARY)
                        ),
                    })
                    .when(self.workspace().synchronized_navigation, |this| {
                        this.child("· SYNC")
                    }),
            )
            .child(
                h_flex()
                    .gap_1()
                    .when_some(latest, |this, summary| {
                        this.child(div().max_w(px(360.)).truncate().child(summary))
                    })
                    .when(running, |this| {
                        this.child(
                            Button::new("pause-operation")
                                .ghost()
                                .xsmall()
                                .icon(if self.operation_paused {
                                    IconName::Play
                                } else {
                                    IconName::Pause
                                })
                                .tooltip(if self.operation_paused {
                                    "Resume operation"
                                } else {
                                    "Pause operation"
                                })
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.toggle_operation_pause(cx)),
                                ),
                        )
                        .child(
                            Button::new("cancel-operation")
                                .ghost()
                                .xsmall()
                                .icon(IconName::Close)
                                .tooltip("Cancel operation")
                                .on_click(cx.listener(|this, _, _, cx| this.cancel_operation(cx))),
                        )
                    })
                    .child(
                        Button::new("undo-operation")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Undo2)
                            .tooltip("Undo last safe operation · Ctrl+Z")
                            .disabled(!self.command_enabled(CommandId::Undo))
                            .on_click(cx.listener(|this, _, _, cx| this.undo_last_operation(cx))),
                    ),
            )
    }
}

impl Render for Explorer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_split_states(cx);
        let folder_name = self
            .active_path()
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("Nimbus");
        window.set_window_title(&format!("{folder_name} — Nimbus"));

        let layout = self.workspace().layout.clone();
        let workspace = self.render_split_node(layout, cx);
        let main_workspace = if self.preview_visible && self.preview_pinned {
            h_resizable("workspace-preview")
                .child(resizable_panel().child(workspace))
                .child(
                    resizable_panel()
                        .size(px(380.))
                        .size_range(px(280.)..px(620.))
                        .flex_none()
                        .child(self.render_preview_content(cx)),
                )
                .into_any_element()
        } else {
            workspace
        };
        let body_content = if self.workspace().sidebar.open {
            h_resizable("sidebar-workspace")
                .with_state(&self.sidebar_resize)
                .child(
                    resizable_panel()
                        .size(px(self.workspace().sidebar.width))
                        .size_range(px(220.)..px(460.))
                        .flex_none()
                        .child(self.render_sidebar(cx)),
                )
                .child(resizable_panel().child(main_workspace))
                .into_any_element()
        } else {
            main_workspace
        };

        let mut root = div()
            .id("nimbus-root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .relative()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                v_flex()
                    .size_full()
                    .child(
                        TitleBar::new().child(
                            h_flex()
                                .gap_2()
                                .px_2()
                                .font_semibold()
                                .child(Icon::new(IconName::FolderOpen).small())
                                .child("Nimbus")
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("· {}", self.workspace().name)),
                                ),
                        ),
                    )
                    .child(self.render_toolbar(cx))
                    .child(
                        h_flex()
                            .flex_1()
                            .min_h_0()
                            .child(self.render_activity_bar(cx))
                            .child(body_content),
                    )
                    .child(self.render_status_bar(cx)),
            );
        if self.preview_visible && !self.preview_pinned {
            root = root.child(
                div()
                    .absolute()
                    .top(px(76.))
                    .right(px(24.))
                    .bottom(px(48.))
                    .w(px(560.))
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .shadow_lg()
                    .overflow_hidden()
                    .child(self.render_preview_content(cx)),
            );
        }
        root = match self.dialog {
            DialogState::Palette => root.child(self.render_palette(cx)),
            DialogState::Name => root.child(self.render_name_dialog(cx)),
            DialogState::Operation => root.child(self.render_operation_dialog(cx)),
            DialogState::None => root,
        };
        root
    }
}

fn info_row(label: &str, value: String, cx: &Context<Explorer>) -> AnyElement {
    h_flex()
        .justify_between()
        .gap_3()
        .border_b_1()
        .border_color(cx.theme().border.opacity(0.5))
        .pb_2()
        .child(
            div()
                .text_color(cx.theme().muted_foreground)
                .child(label.to_string()),
        )
        .child(div().text_right().child(value))
        .into_any_element()
}

fn preview_placeholder(
    icon: IconName,
    title: &str,
    description: &str,
    cx: &Context<Explorer>,
) -> AnyElement {
    v_flex()
        .flex_1()
        .items_center()
        .justify_center()
        .gap_3()
        .p_5()
        .text_center()
        .child(Icon::new(icon))
        .child(div().text_lg().font_semibold().child(title.to_string()))
        .child(
            div()
                .max_w(px(360.))
                .text_color(cx.theme().muted_foreground)
                .child(description.to_string()),
        )
        .into_any_element()
}

fn home_directory() -> PathBuf {
    env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}
