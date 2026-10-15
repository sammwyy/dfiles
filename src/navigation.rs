use crate::{config::ExplorerConfig, filesystem as files};
use creamui_widgets::{ScrollController, TextController};
use std::{path::PathBuf, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Location {
    Home,
    Directory(PathBuf),
}
impl Location {
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Home => "Home".into(),
            Self::Directory(path) => files::name(path),
        }
    }
    pub(crate) fn address(&self) -> String {
        match self {
            Self::Home => "Home".into(),
            Self::Directory(path) => path.display().to_string(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Tab {
    pub(crate) history: Vec<Location>,
    pub(crate) cursor: usize,
    pub(crate) entries: Rc<Vec<files::Entry>>,
    pub(crate) error: Option<String>,
    pub(crate) selected: Option<PathBuf>,
    pub(crate) grid: bool,
    pub(crate) show_hidden: bool,
    pub(crate) address: TextController,
    pub(crate) search: TextController,
    pub(crate) scroll: ScrollController,
}
impl Tab {
    pub(crate) fn new(location: Location, explorer: &ExplorerConfig) -> Self {
        let mut tab = Self {
            address: TextController::new(location.address()),
            history: vec![location],
            cursor: 0,
            entries: Rc::new(Vec::new()),
            error: None,
            selected: None,
            grid: explorer.default_grid_view,
            show_hidden: explorer.show_hidden,
            search: TextController::new(""),
            scroll: ScrollController::new(0.0),
        };
        tab.reload();
        tab
    }
    pub(crate) fn location(&self) -> &Location {
        &self.history[self.cursor]
    }
    pub(crate) fn reload(&mut self) {
        self.error = None;
        self.entries = Rc::new(match self.location() {
            Location::Home => Vec::new(),
            Location::Directory(path) => match files::entries(path, self.show_hidden) {
                Ok(entries) => entries,
                Err(error) => {
                    self.error = Some(format!("Could not read this folder: {error}"));
                    Vec::new()
                }
            },
        });
        if self
            .selected
            .as_ref()
            .is_some_and(|p| !self.entries.iter().any(|e| &e.path == p))
        {
            self.selected = None;
        }
    }
    pub(crate) fn moved(&mut self) {
        self.selected = None;
        self.search.set_value("");
        self.address.set_value(self.location().address());
        self.scroll.set(0.0);
        self.reload();
    }
    pub(crate) fn navigate(&mut self, location: Location) {
        if self.location() != &location {
            self.history.truncate(self.cursor + 1);
            self.history.push(location);
            self.cursor += 1;
        }
        self.moved();
    }
    pub(crate) fn travel(&mut self, forward: bool) {
        if forward && self.cursor + 1 < self.history.len() {
            self.cursor += 1;
        } else if !forward && self.cursor > 0 {
            self.cursor -= 1;
        } else {
            return;
        }
        self.moved();
    }
}
