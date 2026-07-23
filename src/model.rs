use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub type PanelId = u64;
pub type SplitId = u64;

pub const MAX_RECOMMENDED_PANELS: usize = 4;
pub const STATE_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortField {
    #[default]
    Name,
    Modified,
    Kind,
    Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortSpec {
    pub field: SortField,
    pub direction: SortDirection,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TabState {
    pub path: PathBuf,
    pub pinned: bool,
    pub history: Vec<PathBuf>,
    pub history_index: usize,
    pub sort: SortSpec,
    pub filter: String,
    pub scroll_item: usize,
}

impl Default for TabState {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl TabState {
    pub fn new(path: PathBuf) -> Self {
        Self {
            history: vec![path.clone()],
            path,
            pinned: false,
            history_index: 0,
            sort: SortSpec::default(),
            filter: String::new(),
            scroll_item: 0,
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    pub fn navigate(&mut self, path: PathBuf) {
        if path == self.path {
            return;
        }
        self.history.truncate(self.history_index + 1);
        self.history.push(path.clone());
        self.history_index = self.history.len() - 1;
        self.path = path;
        self.scroll_item = 0;
    }

    pub fn go_back(&mut self) -> Option<PathBuf> {
        if !self.can_go_back() {
            return None;
        }
        self.history_index -= 1;
        self.path = self.history[self.history_index].clone();
        self.scroll_item = 0;
        Some(self.path.clone())
    }

    pub fn go_forward(&mut self) -> Option<PathBuf> {
        if !self.can_go_forward() {
            return None;
        }
        self.history_index += 1;
        self.path = self.history[self.history_index].clone();
        self.scroll_item = 0;
        Some(self.path.clone())
    }

    pub fn normalize(&mut self, fallback: &Path) {
        if self.history.is_empty() {
            self.history.push(self.path.clone());
        }
        self.history_index = self.history_index.min(self.history.len() - 1);
        self.path = self.history[self.history_index].clone();
        if self.path.as_os_str().is_empty() {
            self.path = fallback.to_path_buf();
            self.history = vec![self.path.clone()];
            self.history_index = 0;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelState {
    pub id: PanelId,
    pub tabs: Vec<TabState>,
    pub active_tab: usize,
}

impl Default for PanelState {
    fn default() -> Self {
        Self::new(1, PathBuf::from("."))
    }
}

impl PanelState {
    pub fn new(id: PanelId, path: PathBuf) -> Self {
        Self {
            id,
            tabs: vec![TabState::new(path)],
            active_tab: 0,
        }
    }

    pub fn active_tab(&self) -> &TabState {
        &self.tabs[self.active_tab]
    }

    pub fn active_tab_mut(&mut self) -> &mut TabState {
        &mut self.tabs[self.active_tab]
    }

    pub fn open_tab(&mut self, path: PathBuf) {
        self.tabs.push(TabState::new(path));
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self, index: usize) -> bool {
        if self.tabs.len() <= 1 || index >= self.tabs.len() || self.tabs[index].pinned {
            return false;
        }
        self.tabs.remove(index);
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
        true
    }

    fn normalize(&mut self, fallback: &Path) {
        if self.tabs.is_empty() {
            self.tabs.push(TabState::new(fallback.to_path_buf()));
        }
        for tab in &mut self.tabs {
            tab.normalize(fallback);
        }
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitAxis {
    Columns,
    Rows,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SplitNode {
    Panel(PanelId),
    Split {
        id: SplitId,
        axis: SplitAxis,
        ratio: f32,
        first: Box<SplitNode>,
        second: Box<SplitNode>,
    },
}

impl SplitNode {
    pub fn panel_ids(&self, output: &mut Vec<PanelId>) {
        match self {
            SplitNode::Panel(id) => output.push(*id),
            SplitNode::Split { first, second, .. } => {
                first.panel_ids(output);
                second.panel_ids(output);
            }
        }
    }

    pub fn split_ids(&self, output: &mut Vec<SplitId>) {
        if let SplitNode::Split {
            id, first, second, ..
        } = self
        {
            output.push(*id);
            first.split_ids(output);
            second.split_ids(output);
        }
    }

    pub fn set_ratio(&mut self, split_id: SplitId, ratio: f32) -> bool {
        match self {
            SplitNode::Panel(_) => false,
            SplitNode::Split {
                id,
                ratio: current,
                first,
                second,
                ..
            } => {
                if *id == split_id {
                    *current = ratio.clamp(0.15, 0.85);
                    true
                } else {
                    first.set_ratio(split_id, ratio) || second.set_ratio(split_id, ratio)
                }
            }
        }
    }

    fn split_panel(
        &mut self,
        target: PanelId,
        axis: SplitAxis,
        new_panel: PanelId,
        split_id: SplitId,
    ) -> bool {
        match self {
            SplitNode::Panel(id) if *id == target => {
                *self = SplitNode::Split {
                    id: split_id,
                    axis,
                    ratio: 0.5,
                    first: Box::new(SplitNode::Panel(target)),
                    second: Box::new(SplitNode::Panel(new_panel)),
                };
                true
            }
            SplitNode::Panel(_) => false,
            SplitNode::Split { first, second, .. } => {
                first.split_panel(target, axis, new_panel, split_id)
                    || second.split_panel(target, axis, new_panel, split_id)
            }
        }
    }

    fn remove_panel(self, target: PanelId) -> Option<Self> {
        match self {
            SplitNode::Panel(id) => (id != target).then_some(SplitNode::Panel(id)),
            SplitNode::Split {
                id,
                axis,
                ratio,
                first,
                second,
            } => {
                let first = first.remove_panel(target);
                let second = second.remove_panel(target);
                match (first, second) {
                    (Some(first), Some(second)) => Some(SplitNode::Split {
                        id,
                        axis,
                        ratio,
                        first: Box::new(first),
                        second: Box::new(second),
                    }),
                    (Some(node), None) | (None, Some(node)) => Some(node),
                    (None, None) => None,
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutPreset {
    #[default]
    Single,
    TwoColumns,
    TwoRows,
    ThreeColumns,
    Grid,
}

impl LayoutPreset {
    pub const ALL: [Self; 5] = [
        Self::Single,
        Self::TwoColumns,
        Self::TwoRows,
        Self::ThreeColumns,
        Self::Grid,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Single => "1",
            Self::TwoColumns => "2 ↔",
            Self::TwoRows => "2 ↕",
            Self::ThreeColumns => "3",
            Self::Grid => "2×2",
        }
    }

    pub fn panel_count(self) -> usize {
        match self {
            Self::Single => 1,
            Self::TwoColumns | Self::TwoRows => 2,
            Self::ThreeColumns => 3,
            Self::Grid => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidebarTool {
    #[default]
    Navigation,
    Shelf,
    Git,
    Info,
    Statistics,
    Search,
}

impl SidebarTool {
    pub const ALL: [Self; 6] = [
        Self::Navigation,
        Self::Shelf,
        Self::Git,
        Self::Info,
        Self::Statistics,
        Self::Search,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Navigation => "Navigation",
            Self::Shelf => "Shelf",
            Self::Git => "Git history",
            Self::Info => "Folder info",
            Self::Statistics => "Statistics",
            Self::Search => "Search",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SidebarState {
    pub open: bool,
    pub pinned: bool,
    pub width: f32,
    pub active_tool: SidebarTool,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            open: true,
            pinned: true,
            width: 284.0,
            active_tool: SidebarTool::Navigation,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceState {
    pub name: String,
    pub layout: SplitNode,
    pub panels: Vec<PanelState>,
    pub active_panel: PanelId,
    pub target_panel: Option<PanelId>,
    pub sidebar: SidebarState,
    pub synchronized_navigation: bool,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self::new("Default", PathBuf::from("."))
    }
}

impl WorkspaceState {
    pub fn new(name: impl Into<String>, path: PathBuf) -> Self {
        let panel = PanelState::new(1, path);
        Self {
            name: name.into(),
            layout: SplitNode::Panel(panel.id),
            panels: vec![panel],
            active_panel: 1,
            target_panel: None,
            sidebar: SidebarState::default(),
            synchronized_navigation: false,
        }
    }

    pub fn panel(&self, id: PanelId) -> Option<&PanelState> {
        self.panels.iter().find(|panel| panel.id == id)
    }

    pub fn panel_mut(&mut self, id: PanelId) -> Option<&mut PanelState> {
        self.panels.iter_mut().find(|panel| panel.id == id)
    }

    pub fn active_panel(&self) -> &PanelState {
        self.panel(self.active_panel)
            .expect("active panel must be present")
    }

    pub fn active_panel_mut(&mut self) -> &mut PanelState {
        self.panel_mut(self.active_panel)
            .expect("active panel must be present")
    }

    pub fn next_panel_id(&self) -> PanelId {
        self.panels.iter().map(|panel| panel.id).max().unwrap_or(0) + 1
    }

    pub fn next_split_id(&self) -> SplitId {
        let mut ids = Vec::new();
        self.layout.split_ids(&mut ids);
        ids.into_iter().max().unwrap_or(0) + 1
    }

    pub fn split_active(&mut self, axis: SplitAxis) -> Option<PanelId> {
        if self.panels.len() >= MAX_RECOMMENDED_PANELS {
            return None;
        }
        let source = self.active_panel().active_tab().path.clone();
        let new_panel_id = self.next_panel_id();
        let split_id = self.next_split_id();
        if self
            .layout
            .split_panel(self.active_panel, axis, new_panel_id, split_id)
        {
            self.panels.push(PanelState::new(new_panel_id, source));
            self.target_panel = Some(new_panel_id);
            Some(new_panel_id)
        } else {
            None
        }
    }

    pub fn close_panel(&mut self, id: PanelId) -> bool {
        if self.panels.len() <= 1 || self.panel(id).is_none() {
            return false;
        }
        let Some(layout) = self.layout.clone().remove_panel(id) else {
            return false;
        };
        self.layout = layout;
        self.panels.retain(|panel| panel.id != id);
        if self.active_panel == id {
            self.active_panel = self.panels[0].id;
        }
        if self.target_panel == Some(id) {
            self.target_panel = None;
        }
        true
    }

    pub fn apply_preset(&mut self, preset: LayoutPreset) {
        let desired = preset.panel_count();
        let source = self.active_panel().active_tab().path.clone();
        while self.panels.len() < desired {
            let id = self.next_panel_id();
            self.panels.push(PanelState::new(id, source.clone()));
        }
        self.panels.truncate(desired);
        let ids = self.panels.iter().map(|panel| panel.id).collect::<Vec<_>>();
        let mut split_id = self.next_split_id();
        let mut split = |axis, first, second| {
            let id = split_id;
            split_id += 1;
            SplitNode::Split {
                id,
                axis,
                ratio: 0.5,
                first: Box::new(first),
                second: Box::new(second),
            }
        };
        self.layout = match preset {
            LayoutPreset::Single => SplitNode::Panel(ids[0]),
            LayoutPreset::TwoColumns => split(
                SplitAxis::Columns,
                SplitNode::Panel(ids[0]),
                SplitNode::Panel(ids[1]),
            ),
            LayoutPreset::TwoRows => split(
                SplitAxis::Rows,
                SplitNode::Panel(ids[0]),
                SplitNode::Panel(ids[1]),
            ),
            LayoutPreset::ThreeColumns => {
                let right = split(
                    SplitAxis::Columns,
                    SplitNode::Panel(ids[1]),
                    SplitNode::Panel(ids[2]),
                );
                split(SplitAxis::Columns, SplitNode::Panel(ids[0]), right)
            }
            LayoutPreset::Grid => {
                let top = split(
                    SplitAxis::Columns,
                    SplitNode::Panel(ids[0]),
                    SplitNode::Panel(ids[1]),
                );
                let bottom = split(
                    SplitAxis::Columns,
                    SplitNode::Panel(ids[2]),
                    SplitNode::Panel(ids[3]),
                );
                split(SplitAxis::Rows, top, bottom)
            }
        };
        if self.panel(self.active_panel).is_none() {
            self.active_panel = ids[0];
        }
        self.target_panel = ids.iter().copied().find(|id| *id != self.active_panel);
    }

    pub fn normalize(&mut self, fallback: &Path) {
        if self.panels.is_empty() {
            self.panels.push(PanelState::new(1, fallback.to_path_buf()));
        }
        for panel in &mut self.panels {
            panel.normalize(fallback);
        }

        let valid = self
            .panels
            .iter()
            .map(|panel| panel.id)
            .collect::<HashSet<_>>();
        let mut layout_ids = Vec::new();
        self.layout.panel_ids(&mut layout_ids);
        if layout_ids.is_empty()
            || layout_ids.iter().any(|id| !valid.contains(id))
            || layout_ids.iter().copied().collect::<HashSet<_>>().len() != self.panels.len()
        {
            self.layout = SplitNode::Panel(self.panels[0].id);
            self.panels.truncate(1);
        }
        if self.panel(self.active_panel).is_none() {
            self.active_panel = self.panels[0].id;
        }
        if self.target_panel.is_some_and(|id| self.panel(id).is_none()) {
            self.target_panel = None;
        }
        self.sidebar.width = self.sidebar.width.clamp(220.0, 460.0);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ShelfItem {
    pub path: PathBuf,
    pub note: String,
    pub color: Option<String>,
}

impl Default for ShelfItem {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            note: String::new(),
            color: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Shelf {
    pub name: String,
    pub persistent: bool,
    pub items: Vec<ShelfItem>,
}

impl Default for Shelf {
    fn default() -> Self {
        Self {
            name: "Shelf 1".to_string(),
            persistent: true,
            items: Vec::new(),
        }
    }
}

impl Shelf {
    pub fn add_paths(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> usize {
        let mut added = 0;
        for path in paths {
            if self.items.iter().all(|item| item.path != path) {
                self.items.push(ShelfItem {
                    path,
                    ..ShelfItem::default()
                });
                added += 1;
            }
        }
        added
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedSearch {
    pub name: String,
    pub query: String,
    pub include_hidden: bool,
    pub extension: Option<String>,
}

impl Default for SavedSearch {
    fn default() -> Self {
        Self {
            name: "Saved search".to_string(),
            query: String::new(),
            include_hidden: false,
            extension: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub restore_last_workspace: bool,
    pub confirm_trash: bool,
    pub default_conflict_resolution: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            restore_last_workspace: true,
            confirm_trash: true,
            default_conflict_resolution: "rename".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    pub version: u32,
    pub workspaces: Vec<WorkspaceState>,
    pub active_workspace: usize,
    pub shelves: Vec<Shelf>,
    pub active_shelf: usize,
    pub saved_searches: Vec<SavedSearch>,
    pub command_history: Vec<String>,
    pub settings: AppSettings,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl AppState {
    pub fn new(initial_path: PathBuf) -> Self {
        Self {
            version: STATE_VERSION,
            workspaces: vec![WorkspaceState::new("Default", initial_path)],
            active_workspace: 0,
            shelves: vec![Shelf::default()],
            active_shelf: 0,
            saved_searches: Vec::new(),
            command_history: Vec::new(),
            settings: AppSettings::default(),
        }
    }

    pub fn workspace(&self) -> &WorkspaceState {
        &self.workspaces[self.active_workspace]
    }

    pub fn workspace_mut(&mut self) -> &mut WorkspaceState {
        &mut self.workspaces[self.active_workspace]
    }

    pub fn shelf(&self) -> &Shelf {
        &self.shelves[self.active_shelf]
    }

    pub fn shelf_mut(&mut self) -> &mut Shelf {
        &mut self.shelves[self.active_shelf]
    }

    pub fn select_workspace(&mut self, name: &str) -> bool {
        if let Some(index) = self
            .workspaces
            .iter()
            .position(|workspace| workspace.name.eq_ignore_ascii_case(name))
        {
            self.active_workspace = index;
            true
        } else {
            false
        }
    }

    pub fn normalize(&mut self, fallback: &Path) {
        self.version = STATE_VERSION;
        if self.workspaces.is_empty() {
            self.workspaces
                .push(WorkspaceState::new("Default", fallback.to_path_buf()));
        }
        for workspace in &mut self.workspaces {
            workspace.normalize(fallback);
        }
        self.active_workspace = self.active_workspace.min(self.workspaces.len() - 1);
        if self.shelves.is_empty() {
            self.shelves.push(Shelf::default());
        }
        self.active_shelf = self.active_shelf.min(self.shelves.len() - 1);
        self.command_history.truncate(50);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_tree_supports_recursive_panels_and_removal() {
        let mut workspace = WorkspaceState::new("Test", PathBuf::from("C:\\"));
        let second = workspace.split_active(SplitAxis::Columns).unwrap();
        workspace.active_panel = second;
        let third = workspace.split_active(SplitAxis::Rows).unwrap();

        let mut ids = Vec::new();
        workspace.layout.panel_ids(&mut ids);
        assert_eq!(ids, vec![1, second, third]);

        assert!(workspace.close_panel(second));
        ids.clear();
        workspace.layout.panel_ids(&mut ids);
        assert_eq!(ids, vec![1, third]);
    }

    #[test]
    fn preset_reuses_paths_and_caps_the_initial_ui_at_four_panels() {
        let mut workspace = WorkspaceState::new("Test", PathBuf::from("C:\\work"));
        workspace.apply_preset(LayoutPreset::Grid);
        assert_eq!(workspace.panels.len(), MAX_RECOMMENDED_PANELS);
        assert!(
            workspace
                .panels
                .iter()
                .all(|panel| panel.active_tab().path == Path::new("C:\\work"))
        );
    }

    #[test]
    fn tab_history_discards_forward_entries_after_navigation() {
        let mut tab = TabState::new(PathBuf::from("a"));
        tab.navigate(PathBuf::from("b"));
        tab.navigate(PathBuf::from("c"));
        assert_eq!(tab.go_back(), Some(PathBuf::from("b")));
        tab.navigate(PathBuf::from("d"));
        assert!(!tab.can_go_forward());
        assert_eq!(
            tab.history,
            vec![PathBuf::from("a"), PathBuf::from("b"), PathBuf::from("d")]
        );
    }

    #[test]
    fn shelf_deduplicates_paths() {
        let mut shelf = Shelf::default();
        assert_eq!(
            shelf.add_paths([PathBuf::from("a"), PathBuf::from("a"), PathBuf::from("b")]),
            2
        );
        assert_eq!(shelf.items.len(), 2);
    }
}
