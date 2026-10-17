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

pub(crate) fn directory_header(b: &BrowserState, tab: &Tab, width: f32) -> BoxedWidget {
    if tab.grid {
        return view(fixed(1.0, 0.0), vec![]);
    }
    let c = &b.explorer.columns;
    let layout = detail_layout(c, width);
    // The first file cell starts after its icon and the row gap.
    // Keep the header's inset identical to a row: the directory content has
    // 24px of padding and each entry adds 12px around its cells.
    let mut children = vec![view(fixed(24.0, 28.0), vec![])];
    if c.name {
        children.push(sort_header(
            b,
            "Name",
            SortColumn::Name,
            name_column(layout.name, 28.0),
            false,
        ));
    }
    if c.size {
        children.push(sort_header(
            b,
            "Size",
            SortColumn::Size,
            fixed(layout.size, 28.0),
            true,
        ));
    }
    if c.kind {
        children.push(sort_header(
            b,
            "Type",
            SortColumn::Kind,
            fixed(layout.kind, 28.0),
            false,
        ));
    }
    if c.modified {
        children.push(sort_header(
            b,
            "Modified",
            SortColumn::Modified,
            fixed(layout.modified, 28.0),
            true,
        ));
    }
    if c.created {
        children.push(sort_header(
            b,
            "Created",
            SortColumn::Created,
            fixed(layout.created, 28.0),
            true,
        ));
    }
    if c.permissions {
        children.push(sort_header(
            b,
            "Permissions",
            SortColumn::Permissions,
            fixed(layout.permissions, 28.0),
            true,
        ));
    }
    view(pad(row(layout.gap), 36.0, 2.0), children)
}

fn name_column(width: f32, height: f32) -> Style {
    Style {
        flex_grow: 1.0,
        flex_basis: Dimension::Length(width),
        ..fixed(0.0, height)
    }
}

fn sort_header(
    b: &BrowserState,
    title: &str,
    column: SortColumn,
    mut style: Style,
    align_end: bool,
) -> BoxedWidget {
    let active = tab_sort(b) == column;
    let ascending = b.tab().sort_ascending;
    let state = b.clone();
    let t = use_theme();
    if align_end {
        style.justify_content = Some(JustifyContent::End);
    }
    // Sort headers are clickable labels, not toolbar buttons. Keeping their
    // surface transparent avoids a full-column hover pill when the pointer
    // happens to cross the header.
    Box::new(
        RawButton::new(style, move || {
            state.edit(|tab| {
                if tab.sort == column {
                    tab.sort_ascending = !tab.sort_ascending;
                } else {
                    tab.sort = column;
                    tab.sort_ascending = true;
                }
            })
        })
        .background(CLEAR)
        .with_children(vec![
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
        ]),
    )
}

/// Semantic column widths for a directory row.  File names receive all of
/// the surplus width; metadata keeps a useful, content-based width and is
/// compacted only when the window cannot accommodate every optional column.
#[derive(Clone, Copy)]
struct DetailLayout {
    gap: f32,
    name: f32,
    size: f32,
    kind: f32,
    modified: f32,
    created: f32,
    permissions: f32,
}

fn detail_layout(columns: &crate::config::DetailColumns, width: f32) -> DetailLayout {
    const OUTER_PADDING: f32 = 24.0;
    const CELL_PADDING: f32 = 12.0;
    const ICON_WIDTH: f32 = 24.0;
    const COMPACT_GAP: f32 = 8.0;
    const REGULAR_GAP: f32 = 12.0;

    // Preferred widths let readable fields retain their natural rhythm on a
    // spacious window. Minimums preserve the important value in every field
    // when users enable all of the optional columns.
    const SIZE: (f32, f32) = (82.0, 48.0);
    const KIND: (f32, f32) = (118.0, 60.0);
    const DATE: (f32, f32) = (126.0, 70.0);
    const PERMISSIONS: (f32, f32) = (96.0, 64.0);

    let gap = if width < 720.0 {
        COMPACT_GAP
    } else {
        REGULAR_GAP
    };
    let metadata = [
        (columns.size, SIZE),
        (columns.kind, KIND),
        (columns.modified, DATE),
        (columns.created, DATE),
        (columns.permissions, PERMISSIONS),
    ];
    let metadata_count = metadata.iter().filter(|(visible, _)| *visible).count();
    let cell_count = 1 + usize::from(columns.name) + metadata_count;
    let cells_width = (width
        - 2.0 * (OUTER_PADDING + CELL_PADDING)
        - ICON_WIDTH
        - gap * cell_count.saturating_sub(1) as f32)
        .max(0.0);
    let minimum_metadata: f32 = metadata
        .iter()
        .filter(|(visible, _)| *visible)
        .map(|(_, (_, minimum))| minimum)
        .sum();
    let preferred_metadata: f32 = metadata
        .iter()
        .filter(|(visible, _)| *visible)
        .map(|(_, (preferred, _))| preferred)
        .sum();

    // Reserve a meaningful part of the row for names, without starving
    // optional metadata. The final value is also used for safe truncation.
    let preferred_name = (width * 0.30).clamp(128.0, 280.0);
    let name = if columns.name {
        preferred_name.min((cells_width - minimum_metadata).max(0.0))
    } else {
        0.0
    };
    let available_metadata = (cells_width - name).max(0.0);
    let interpolation = if preferred_metadata > minimum_metadata {
        ((available_metadata - minimum_metadata) / (preferred_metadata - minimum_metadata))
            .clamp(0.0, 1.0)
    } else {
        1.0
    };
    let column_width = |visible: bool, (preferred, minimum): (f32, f32)| {
        if visible {
            minimum + (preferred - minimum) * interpolation
        } else {
            0.0
        }
    };

    DetailLayout {
        gap,
        name,
        size: column_width(columns.size, SIZE),
        kind: column_width(columns.kind, KIND),
        modified: column_width(columns.modified, DATE),
        created: column_width(columns.created, DATE),
        permissions: column_width(columns.permissions, PERMISSIONS),
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn compact_layout_keeps_room_for_file_names_with_every_column_enabled() {
        let columns = crate::config::DetailColumns {
            name: true,
            size: true,
            kind: true,
            modified: true,
            created: true,
            permissions: true,
        };
        let layout = detail_layout(&columns, 608.0);

        assert_eq!(layout.gap, 8.0);
        assert!(layout.name >= 150.0);
        assert_eq!(layout.size, 48.0);
        assert_eq!(layout.kind, 60.0);
        assert_eq!(layout.modified, 70.0);
        assert_eq!(layout.created, 70.0);
        assert_eq!(layout.permissions, 64.0);
    }

    #[test]
    fn roomy_layout_uses_semantic_preferred_widths() {
        let columns = crate::config::DetailColumns {
            name: true,
            size: true,
            kind: true,
            modified: true,
            created: true,
            permissions: true,
        };
        let layout = detail_layout(&columns, 1200.0);

        assert_eq!(layout.gap, 12.0);
        assert_eq!(layout.size, 82.0);
        assert_eq!(layout.kind, 118.0);
        assert_eq!(layout.modified, 126.0);
        assert_eq!(layout.created, 126.0);
        assert_eq!(layout.permissions, 96.0);
    }

    #[test]
    fn permissions_show_owner_group_and_other_bits() {
        assert_eq!(permissions(0o644), "rw-r--r--");
        assert_eq!(permissions(0o755), "rwxr-xr-x");
        assert_eq!(permissions(0o700), "rwx------");
    }
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
                                width: LengthPercentage::Length(6.0),
                                height: LengthPercentage::Length(6.0),
                            },
                            ..fixed(96.0 * scale, 94.0 * scale)
                        },
                        6.0,
                        6.0,
                    ),
                    vec![
                        glyph(b, icon, 56.0 * scale, t.colors.accent),
                        label(
                            short(&name, text_capacity(96.0 * scale - 12.0, 14.0)),
                            12.0,
                            false,
                        ),
                    ],
                )
            } else {
                let c = &b.explorer.columns;
                let layout = detail_layout(c, width);
                let mut cells = vec![glyph(b, icon, 24.0 * scale, t.colors.accent)];
                if c.name {
                    cells.push(view(
                        name_column(layout.name, 20.0),
                        vec![label(
                            short(&name, text_capacity(layout.name, 13.0)),
                            13.0,
                            false,
                        )],
                    ));
                }
                if c.size {
                    cells.push(end_cell(
                        layout.size,
                        if entry.directory {
                            "—".into()
                        } else {
                            files::size(entry.bytes)
                        },
                    ));
                }
                if c.kind {
                    cells.push(view(
                        fixed(layout.kind, 20.0),
                        vec![label(
                            short(&entry.kind(), text_capacity(layout.kind, 11.0)),
                            11.0,
                            true,
                        )],
                    ));
                }
                if c.modified {
                    cells.push(end_cell(layout.modified, date(entry.modified)));
                }
                if c.created {
                    cells.push(end_cell(layout.created, date(entry.created)));
                }
                if c.permissions {
                    cells.push(end_cell(layout.permissions, permissions(entry.permissions)));
                }
                (
                    pad(
                        Style {
                            size: creamui_core::layout::Size {
                                width: Dimension::Percent(1.0),
                                height: Dimension::Length(42.0 * scale),
                            },
                            ..row(layout.gap)
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
            wrap(items, 10.0)
        } else {
            view(column(2.0), items)
        });
    }
    view(pad(column(20.0), 24.0, 20.0), content)
}

fn end_cell(width: f32, text: impl Into<String>) -> BoxedWidget {
    view(
        Style {
            justify_content: Some(JustifyContent::End),
            ..fixed(width, 20.0)
        },
        vec![label(text, 11.0, true)],
    )
}

fn text_capacity(width: f32, font_size: f32) -> usize {
    // Latin filenames and the metadata produced by this view average roughly
    // half an em per character. Keeping the estimate conservative prevents a
    // narrow cell from wrapping into the next row.
    (width / (font_size * 0.55)).floor().max(1.0) as usize
}

/// Unix permissions grouped as owner, group and everyone else, e.g.
/// `rwxr-xr-x`, rather than an opaque octal value such as `755`.
fn permissions(mode: u32) -> String {
    [6, 3, 0]
        .into_iter()
        .flat_map(|shift| {
            [(0o4, 'r'), (0o2, 'w'), (0o1, 'x')]
                .into_iter()
                .map(move |(bit, character)| {
                    if mode & (bit << shift) != 0 {
                        character
                    } else {
                        '-'
                    }
                })
        })
        .collect()
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
