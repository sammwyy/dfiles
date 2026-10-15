use crate::components::*;
use crate::{config::ItemSize, navigation::SortColumn};
use std::{cmp::Ordering, time::SystemTime};

pub(crate) fn visible_entries(b: &BrowserState, tab: &Tab) -> Vec<files::Entry> {
    let query = tab.search.value().to_lowercase();
    let mut entries: Vec<_> = tab
        .entries
        .iter()
        .filter(|e| {
            b.context.accepts(&e.path, e.directory) && e.name().to_lowercase().contains(&query)
        })
        .cloned()
        .collect();
    entries.sort_by(|a, b| {
        compare(a, b, tab.sort).then_with(|| a.name().to_lowercase().cmp(&b.name().to_lowercase()))
    });
    if !tab.sort_ascending {
        entries.reverse();
    }
    entries
}

fn compare(a: &files::Entry, b: &files::Entry, column: SortColumn) -> Ordering {
    match column {
        SortColumn::Name => a.name().to_lowercase().cmp(&b.name().to_lowercase()),
        SortColumn::Size => a.bytes.cmp(&b.bytes),
        SortColumn::Kind => a.kind().cmp(&b.kind()),
        SortColumn::Modified => a.modified.cmp(&b.modified),
        SortColumn::Created => a.created.cmp(&b.created),
        SortColumn::Permissions => a.permissions.cmp(&b.permissions),
    }
}

pub(crate) fn directory_header(b: &BrowserState, tab: &Tab) -> BoxedWidget {
    if tab.grid {
        return view(fixed(1.0, 0.0), vec![]);
    }
    let c = &b.explorer.columns;
    // The first file cell starts after its icon and the row gap.
    let mut children = vec![view(fixed(36.0, 28.0), vec![])];
    if c.name {
        children.push(sort_header(b, "Name", SortColumn::Name, flex_column()));
    }
    if c.size {
        children.push(sort_header(b, "Size", SortColumn::Size, fixed(82.0, 28.0)));
    }
    if c.kind {
        children.push(sort_header(b, "Type", SortColumn::Kind, fixed(118.0, 28.0)));
    }
    if c.modified {
        children.push(sort_header(
            b,
            "Modified",
            SortColumn::Modified,
            fixed(126.0, 28.0),
        ));
    }
    if c.created {
        children.push(sort_header(
            b,
            "Created",
            SortColumn::Created,
            fixed(126.0, 28.0),
        ));
    }
    if c.permissions {
        children.push(sort_header(
            b,
            "Permissions",
            SortColumn::Permissions,
            fixed(96.0, 28.0),
        ));
    }
    view(pad(row(12.0), 24.0, 2.0), children)
}

fn flex_column() -> Style {
    Style {
        flex_grow: 1.0,
        flex_basis: Dimension::Length(0.0),
        ..fixed(0.0, 28.0)
    }
}

fn sort_header(b: &BrowserState, title: &str, column: SortColumn, style: Style) -> BoxedWidget {
    let active = tab_sort(b) == column;
    let ascending = b.tab().sort_ascending;
    let state = b.clone();
    let t = use_theme();
    Box::new(button(
        style,
        // Headers are controls, not selected rows: only their label and
        // chevron indicate the active sort.
        false,
        move || {
            state.edit(|tab| {
                if tab.sort == column {
                    tab.sort_ascending = !tab.sort_ascending;
                } else {
                    tab.sort = column;
                    tab.sort_ascending = true;
                }
            })
        },
        vec![
            Box::new(Text::new(title).font_size(11.0).color(if active {
                t.colors.accent
            } else {
                t.colors.text_secondary
            })),
            if active {
                glyph(
                    b,
                    if ascending {
                        "chevron-up"
                    } else {
                        "chevron-down"
                    },
                    13.0,
                    t.colors.accent,
                )
            } else {
                view(fixed(0.0, 0.0), vec![])
            },
        ],
    ))
}
fn tab_sort(b: &BrowserState) -> SortColumn {
    b.tab().sort
}

pub(crate) fn directory_view(b: &BrowserState, tab: &Tab, width: f32) -> BoxedWidget {
    let t = use_theme();
    let entries = visible_entries(b, tab);
    let scale = match b.explorer.item_size {
        ItemSize::Compact => 0.82,
        ItemSize::Default => 1.0,
        ItemSize::Large => 1.22,
    };
    let mut content = Vec::new();
    if let Some(error) = &tab.error {
        content.push(empty_view(b, "This folder is unavailable", error));
    } else if entries.is_empty() {
        content.push(empty_view(
            b,
            if tab.search.value().is_empty() {
                "This folder is empty"
            } else {
                "No files found"
            },
            if tab.search.value().is_empty() {
                "Files and folders will appear here."
            } else {
                "Try another name or clear the search."
            },
        ));
    } else {
        let mut items = Vec::new();
        for entry in entries {
            let active = tab.selected.as_ref() == Some(&entry.path);
            let icon = if entry.directory {
                "folder-large"
            } else {
                file_icon(&entry.path)
            };
            let name = entry.name();
            let (style, children) = if tab.grid {
                (
                    pad(
                        Style {
                            flex_direction: FlexDirection::Column,
                            gap: creamui_core::layout::Size {
                                width: LengthPercentage::Length(8.0),
                                height: LengthPercentage::Length(8.0),
                            },
                            ..fixed(116.0 * scale, 118.0 * scale)
                        },
                        8.0,
                        8.0,
                    ),
                    vec![
                        glyph(b, icon, 64.0 * scale, t.colors.accent),
                        label(short(&name, 15), 12.0, false),
                        label(
                            if entry.directory {
                                "Folder".into()
                            } else {
                                files::size(entry.bytes)
                            },
                            10.0,
                            true,
                        ),
                    ],
                )
            } else {
                let c = &b.explorer.columns;
                let mut cells = vec![glyph(b, icon, 24.0 * scale, t.colors.accent)];
                if c.name {
                    cells.push(view(
                        flex_column(),
                        vec![label(
                            short(&name, ((width - 330.0) / 7.0).max(12.0) as usize),
                            13.0,
                            false,
                        )],
                    ));
                }
                if c.size {
                    cells.push(view(
                        fixed(82.0, 20.0),
                        vec![label(
                            if entry.directory {
                                "—".into()
                            } else {
                                files::size(entry.bytes)
                            },
                            11.0,
                            true,
                        )],
                    ));
                }
                if c.kind {
                    cells.push(view(
                        fixed(118.0, 20.0),
                        vec![label(short(&entry.kind(), 17), 11.0, true)],
                    ));
                }
                if c.modified {
                    cells.push(view(
                        fixed(126.0, 20.0),
                        vec![label(date(entry.modified), 11.0, true)],
                    ));
                }
                if c.created {
                    cells.push(view(
                        fixed(126.0, 20.0),
                        vec![label(date(entry.created), 11.0, true)],
                    ));
                }
                if c.permissions {
                    cells.push(view(
                        fixed(96.0, 20.0),
                        vec![label(format!("{:03o}", entry.permissions), 11.0, true)],
                    ));
                }
                (
                    pad(
                        Style {
                            size: creamui_core::layout::Size {
                                width: Dimension::Percent(1.0),
                                height: Dimension::Length(42.0 * scale),
                            },
                            ..row(12.0)
                        },
                        12.0,
                        0.0,
                    ),
                    cells,
                )
            };
            items.push(Box::new(entry_button(b, style, active, entry, children)) as BoxedWidget);
        }
        content.push(if tab.grid {
            wrap(items, 12.0)
        } else {
            view(column(2.0), items)
        });
    }
    view(pad(column(20.0), 24.0, 20.0), content)
}

fn date(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return "—".into();
    };
    let days = SystemTime::now()
        .duration_since(time)
        .unwrap_or_default()
        .as_secs()
        / 86_400;
    match days {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        n => format!("{n} days ago"),
    }
}
