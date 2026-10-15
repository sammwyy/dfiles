use super::*;
use crate::views::directory::visible_entries;

pub(crate) const CHROME_HEIGHT: f32 = 54.0;
pub(crate) const COMPACT_STATUS_WIDTH: f32 = 210.0;

#[derive(Default)]
struct DecorationInsets {
    start: f32,
    end: f32,
}

/// Returns the space in this row that belongs to the window manager.
///
/// Hybrid decorations are expressed in window coordinates, while the browser
/// chrome begins after the sidebar.  Reserving the intersecting side here
/// keeps controls out from underneath compositor buttons regardless of which
/// side the compositor puts them on.
fn decoration_insets(
    content_x: f32,
    width: f32,
    row_y: f32,
    row_height: f32,
    client_controls: bool,
) -> DecorationInsets {
    let decorations = creamui_render::use_window_decorations();
    if decorations.mode == WindowDecorationMode::Client {
        return if client_controls {
            DecorationInsets {
                end: 108.0,
                ..Default::default()
            }
        } else {
            DecorationInsets::default()
        };
    }
    let controls = decorations.controls;
    let controls_top = controls.y as f32;
    let controls_bottom = controls_top + controls.height as f32;
    let overlaps_row = controls.width > 0
        && controls.height > 0
        && controls_top < row_y + row_height
        && controls_bottom > row_y;
    if !overlaps_row {
        return DecorationInsets::default();
    }
    let right = content_x + width;
    let center = controls.x as f32 + controls.width as f32 / 2.0;
    if center <= content_x + width / 2.0 {
        DecorationInsets {
            start: (controls.x as f32 + controls.width as f32 + 8.0 - content_x).clamp(0.0, width),
            ..Default::default()
        }
    } else {
        DecorationInsets {
            end: (right - controls.x as f32 + 8.0).clamp(0.0, width),
            ..Default::default()
        }
    }
}

pub(crate) fn tab_bar(
    b: &BrowserState,
    width: f32,
    content_x: f32,
    row_y: f32,
    show_client_controls: bool,
) -> Option<BoxedWidget> {
    // A lone tab does not need its own title strip: its location is already
    // visible in the address bar. Show the strip only when it is needed to
    // switch between tabs, reclaiming that vertical space for the file list.
    if b.tabs.get().len() == 1 {
        return None;
    }
    let t = use_theme();
    let insets = decoration_insets(content_x, width, row_y, CHROME_HEIGHT, show_client_controls);
    let usable_width = (width - insets.start - insets.end).max(0.0);
    let tabs = b.tabs.get();
    // Page the strip when space is tight, keeping the active tab and native
    // window controls visible even after shrinking a window with many tabs.
    let capacity = ((usable_width - 136.0) / 116.0).floor().max(1.0) as usize;
    let paged = tabs.len() > capacity;
    let start = if paged {
        b.active.get() / capacity * capacity
    } else {
        0
    };
    let end = (start + capacity).min(tabs.len());
    // Fill the usable strip instead of leaving a large, ambiguous blank area.
    // The decoration inset is supplied by the compositor/window decorations,
    // so tabs never extend under KWin's (or another compositor's) controls.
    let tab_width =
        ((usable_width - if paged { 136.0 } else { 66.0 }) / (end - start) as f32).max(58.0);
    let mut children = Vec::new();
    if insets.start > 0.0 {
        children.push(view(fixed(insets.start, 1.0), vec![]));
    }
    if paged {
        let previous = b.clone();
        children.push(icon_button(b, "back", start > 0, false, move || {
            previous.switch(start.saturating_sub(1))
        }));
    }
    for (index, tab) in tabs.iter().enumerate().take(end).skip(start) {
        let state = b.clone();
        let close = b.clone();
        let active = index == b.active.get();
        let title_width = (tab_width - 80.0).max(1.0);
        let tab_children = vec![
            Box::new(button(
                pad(
                    Style {
                        flex_grow: 1.0,
                        flex_basis: Dimension::Length(0.0),
                        ..row(7.0)
                    },
                    9.0,
                    0.0,
                ),
                false,
                move || state.switch(index),
                vec![
                    glyph(
                        b,
                        if tab.location() == &Location::Home {
                            "home"
                        } else {
                            "folder"
                        },
                        14.0,
                        if active {
                            t.colors.accent
                        } else {
                            t.colors.text_secondary
                        },
                    ),
                    label(
                        short(&tab.location().label(), (title_width / 8.0) as usize),
                        12.0,
                        !active,
                    ),
                ],
            )) as BoxedWidget,
            icon_button(b, "close", tabs.len() > 1, false, move || {
                close.close_tab(index)
            }),
        ];
        children.push(Box::new(
            RawView::new(fixed(tab_width, 36.0))
                .background(if active { t.colors.surface } else { CLEAR })
                .corner_radius(9.0)
                .with_children(tab_children),
        ) as BoxedWidget);
    }
    if paged {
        let next = b.clone();
        children.push(icon_button(
            b,
            "forward",
            end < tabs.len(),
            false,
            move || next.switch(end),
        ));
    }
    let add = b.clone();
    children.push(icon_button(b, "plus", true, false, move || add.add_tab()));
    children.push(Box::new(CUIWindowDragArea::new().layout(Style {
        flex_grow: 1.0,
        size: creamui_core::layout::Size {
            width: Dimension::Auto,
            height: Dimension::Length(CHROME_HEIGHT),
        },
        ..row(0.0)
    })));
    if show_client_controls
        && creamui_render::use_window_decorations().mode == WindowDecorationMode::Client
    {
        for (index, icon) in ["minimize", "maximize", "close"].iter().enumerate() {
            let state = b.clone();
            children.push(icon_button(b, icon, true, false, move || {
                if let Some(handle) = state.window.borrow().as_ref() {
                    match index {
                        0 => handle.minimize(),
                        1 => {
                            let next = !state.maximized.peek();
                            handle.set_maximized(next);
                            state.maximized.set(next);
                        }
                        _ => handle.close(),
                    }
                }
            }));
        }
    } else if insets.end > 0.0 {
        children.push(view(fixed(insets.end, 1.0), vec![]));
    }
    Some(Box::new(
        RawView::new(pad(
            Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Length(width),
                    height: Dimension::Length(CHROME_HEIGHT),
                },
                flex_shrink: 0.0,
                ..row(3.0)
            },
            8.0,
            0.0,
        ))
        .background(t.colors.surface.mix(t.colors.accent, 0.025))
        .with_children(children),
    ))
}

pub(crate) fn toolbar(
    b: &BrowserState,
    tab: &Tab,
    content_x: f32,
    width: f32,
    row_y: f32,
    show_client_controls: bool,
) -> BoxedWidget {
    let t = use_theme();
    let back = b.clone();
    let forward = b.clone();
    let up = b.clone();
    let address = b.clone();
    let refresh = b.clone();
    let add = b.clone();
    let list = b.clone();
    let grid = b.clone();
    let parent = match tab.location() {
        Location::Directory(p) => p.parent().map(Path::to_path_buf),
        _ => None,
    };
    let insets = decoration_insets(content_x, width, row_y, CHROME_HEIGHT, false);
    let usable_width = (width - insets.start - insets.end).max(0.0);
    let compact_view_mode = usable_width < 610.0;
    let search_width = if usable_width < 540.0 {
        92.0
    } else if usable_width < 700.0 {
        132.0
    } else {
        210.0
    };
    let mut items = Vec::new();
    if insets.start > 0.0 {
        items.push(view(fixed(insets.start, 1.0), vec![]));
    }
    items.extend([
        icon_button(b, "back", tab.cursor > 0, false, move || back.travel(false)),
        icon_button(
            b,
            "forward",
            tab.cursor + 1 < tab.history.len(),
            false,
            move || forward.travel(true),
        ),
        icon_button(b, "up", parent.is_some(), false, move || {
            if let Some(p) = &parent {
                up.navigate(Location::Directory(p.clone()));
            }
        }),
    ]);
    if b.tabs.get().len() == 1 {
        // Keep a compact, dedicated drag handle when the single-tab strip is
        // hidden. It replaces the otherwise empty title-strip real estate.
        items.push(Box::new(CUIWindowDragArea::new().layout(fixed(24.0, 32.0))));
    }
    items.push(Box::new(
        TextInput::controlled(&tab.address)
            .layout(Style {
                flex_grow: 1.0,
                flex_basis: Dimension::Length(0.0),
                min_size: creamui_core::layout::Size {
                    width: Dimension::Length(if compact_view_mode { 72.0 } else { 96.0 }),
                    height: Dimension::Length(34.0),
                },
                ..TextInput::default_style()
            })
            .font_size(12.0)
            .corner_radius(7.0)
            .background(t.colors.surface_elevated)
            .border(t.colors.border, 0.5)
            .on_submit(move || address.submit_address()),
    ));
    items.push(icon_button(b, "refresh", true, false, move || {
        refresh.edit(Tab::reload);
        refresh.drives.set(files::drives());
    }));
    if b.tabs.get().len() == 1 {
        items.push(icon_button(b, "plus", true, false, move || add.add_tab()));
    }
    items.push(Box::new(
        RawView::new(pad(fixed(search_width, 34.0), 10.0, 0.0))
            .background(t.colors.surface_elevated)
            .corner_radius(7.0)
            .with_children(vec![
                glyph(b, "search", 15.0, t.colors.text_secondary),
                Box::new(
                    TextInput::controlled(&tab.search)
                        .layout(Style {
                            flex_grow: 1.0,
                            flex_basis: Dimension::Length(0.0),
                            ..TextInput::default_style()
                        })
                        .height(32.0)
                        .font_size(12.0)
                        .placeholder("Search")
                        .background(CLEAR)
                        .border(CLEAR, 0.0),
                ),
            ]),
    ));
    if tab.location() != &Location::Home {
        items.push(view_mode_switcher(
            b,
            tab.grid,
            compact_view_mode,
            list,
            grid,
        ));
    }
    if show_client_controls
        && creamui_render::use_window_decorations().mode == WindowDecorationMode::Client
    {
        for (index, icon) in ["minimize", "maximize", "close"].iter().enumerate() {
            let state = b.clone();
            items.push(icon_button(b, icon, true, false, move || {
                if let Some(handle) = state.window.borrow().as_ref() {
                    match index {
                        0 => handle.minimize(),
                        1 => {
                            let next = !state.maximized.peek();
                            handle.set_maximized(next);
                            state.maximized.set(next);
                        }
                        _ => handle.close(),
                    }
                }
            }));
        }
    } else if insets.end > 0.0 {
        items.push(view(fixed(insets.end, 1.0), vec![]));
    }
    view(
        pad(
            Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Length(CHROME_HEIGHT),
                },
                flex_shrink: 0.0,
                ..row(4.0)
            },
            12.0,
            0.0,
        ),
        items,
    )
}

fn view_mode_switcher(
    b: &BrowserState,
    grid_active: bool,
    compact: bool,
    list: BrowserState,
    grid: BrowserState,
) -> BoxedWidget {
    let t = use_theme();
    if compact {
        let toggle = b.clone();
        return icon_button(
            b,
            if grid_active { "list" } else { "grid" },
            true,
            true,
            move || toggle.edit(|tab| tab.grid = !tab.grid),
        );
    }
    Box::new(
        RawView::new(fixed(68.0, 32.0))
            .background(t.colors.surface_elevated)
            .corner_radius(8.0)
            .with_children(vec![
                Box::new(button(
                    fixed(34.0, 32.0),
                    !grid_active,
                    move || list.edit(|tab| tab.grid = false),
                    vec![glyph(
                        b,
                        "list",
                        16.0,
                        if !grid_active {
                            t.colors.accent
                        } else {
                            t.colors.text_secondary
                        },
                    )],
                )),
                Box::new(button(
                    fixed(34.0, 32.0),
                    grid_active,
                    move || grid.edit(|tab| tab.grid = true),
                    vec![glyph(
                        b,
                        "grid",
                        16.0,
                        if grid_active {
                            t.colors.accent
                        } else {
                            t.colors.text_secondary
                        },
                    )],
                )),
            ]),
    )
}

pub(crate) fn statusbar(b: &BrowserState, tab: &Tab, compact: bool) -> BoxedWidget {
    let count = if tab.location() == &Location::Home {
        let query = tab.search.value().to_lowercase();
        b.pins
            .get()
            .iter()
            .filter(|p| files::name(p).to_lowercase().contains(&query))
            .count()
            + b.drives
                .get()
                .iter()
                .filter(|d| d.label.to_lowercase().contains(&query))
                .count()
            + if b.context.directories_only() {
                0
            } else {
                b.recent
                    .get()
                    .iter()
                    .filter(|p| {
                        files::name(p).to_lowercase().contains(&query)
                            && b.context.accepts(p, false)
                    })
                    .count()
            }
    } else {
        visible_entries(b, tab).len()
    };
    let selection = b.visible_selection(tab);
    let selected = selection
        .as_ref()
        .map(|p| format!("{} selected", files::name(p)))
        .unwrap_or_else(|| {
            if tab.location() == &Location::Home {
                "Home".into()
            } else {
                "Double-click to open".into()
            }
        });
    let mut children = vec![
        view(
            Style {
                flex_grow: 1.0,
                ..row(0.0)
            },
            vec![label(short(&selected, 65), 11.0, true)],
        ),
        label(format!("{count} items"), 11.0, true),
    ];
    if !compact && b.context.is_manager() && selection.is_some() {
        let state = b.clone();
        children.push(action("Open", false, true, move || state.accept()));
    }
    Box::new(
        RawView::new(pad(
            Style {
                size: creamui_core::layout::Size {
                    width: if compact {
                        Dimension::Length(COMPACT_STATUS_WIDTH)
                    } else {
                        Dimension::Percent(1.0)
                    },
                    height: Dimension::Length(if compact { CHROME_HEIGHT } else { 38.0 }),
                },
                flex_shrink: 0.0,
                ..row(12.0)
            },
            if compact { 12.0 } else { 16.0 },
            0.0,
        ))
        .background(
            use_theme()
                .colors
                .surface
                .mix(use_theme().colors.accent, 0.025),
        )
        .with_children(children),
    )
}
