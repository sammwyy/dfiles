use crate::components::*;

pub(crate) fn visible_entries(b: &BrowserState, tab: &Tab) -> Vec<files::Entry> {
    let query = tab.search.value().to_lowercase();
    tab.entries
        .iter()
        .filter(|e| {
            b.context.accepts(&e.path, e.directory) && e.name().to_lowercase().contains(&query)
        })
        .cloned()
        .collect()
}

pub(crate) fn directory_view(b: &BrowserState, tab: &Tab, width: f32) -> BoxedWidget {
    let t = use_theme();
    let entries = visible_entries(b, tab);
    // The address bar and tab already identify the current folder. Starting
    // with the content keeps the directory view from repeating that name.
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
        if !tab.grid {
            content.push(view(
                pad(row(16.0), 12.0, 4.0),
                vec![
                    view(
                        Style {
                            flex_grow: 1.0,
                            ..row(0.0)
                        },
                        vec![label("Name", 11.0, true)],
                    ),
                    view(fixed(120.0, 20.0), vec![label("Kind", 11.0, true)]),
                    view(fixed(74.0, 20.0), vec![label("Size", 11.0, true)]),
                ],
            ));
        }
        for entry in entries {
            let active = tab.selected.as_ref() == Some(&entry.path);
            let icon = if entry.directory {
                "folder-large"
            } else {
                file_icon(&entry.path)
            };
            let name = entry.name();
            let style;
            let children;
            if tab.grid {
                style = pad(
                    Style {
                        flex_direction: FlexDirection::Column,
                        gap: creamui_core::layout::Size {
                            width: LengthPercentage::Length(8.0),
                            height: LengthPercentage::Length(8.0),
                        },
                        ..fixed(116.0, 118.0)
                    },
                    8.0,
                    8.0,
                );
                children = vec![
                    glyph(b, icon, 64.0, t.colors.accent),
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
                ];
            } else {
                style = pad(
                    Style {
                        size: creamui_core::layout::Size {
                            width: Dimension::Percent(1.0),
                            height: Dimension::Length(42.0),
                        },
                        ..row(16.0)
                    },
                    12.0,
                    0.0,
                );
                children = vec![
                    glyph(b, icon, 24.0, t.colors.accent),
                    view(
                        Style {
                            flex_grow: 1.0,
                            flex_basis: Dimension::Length(0.0),
                            ..row(0.0)
                        },
                        vec![label(
                            short(&name, ((width - 330.0) / 7.0).max(12.0) as usize),
                            13.0,
                            false,
                        )],
                    ),
                    view(
                        fixed(120.0, 20.0),
                        vec![label(short(&entry.kind(), 17), 11.0, true)],
                    ),
                    view(
                        fixed(74.0, 20.0),
                        vec![label(
                            if entry.directory {
                                "—".into()
                            } else {
                                files::size(entry.bytes)
                            },
                            11.0,
                            true,
                        )],
                    ),
                ];
            }
            items.push(Box::new(entry_button(b, style, active, entry, children)) as BoxedWidget);
        }
        content.push(if tab.grid {
            wrap(items, 12.0)
        } else {
            view(column(2.0), items)
        });
    }
    view(pad(column(20.0), 24.0, 24.0), content)
}
