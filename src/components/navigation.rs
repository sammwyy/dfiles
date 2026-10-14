use super::*;
use crate::views::directory::visible_entries;

pub(crate) fn tab_bar(b: &BrowserState, window_width: f32, sidebar_width: f32) -> BoxedWidget {
    let width = window_width - sidebar_width;
    let t = use_theme();
    let decorations = creamui_render::use_window_decorations();
    let reserved = if decorations.mode == WindowDecorationMode::Client {
        108.0
    } else if decorations.controls.width > 0 && decorations.controls.x as f32 > window_width / 2.0 {
        (window_width - decorations.controls.x as f32 + 8.0).max(0.0)
    } else {
        10.0
    };
    let tabs = b.tabs.get();
    // Page the strip when space is tight, keeping the active tab and native
    // window controls visible even after shrinking a window with many tabs.
    let capacity = ((width - reserved - 136.0) / 116.0).floor().max(1.0) as usize;
    let paged = tabs.len() > capacity;
    let start = if paged {
        b.active.get() / capacity * capacity
    } else {
        0
    };
    let end = (start + capacity).min(tabs.len());
    let tab_width = ((width - reserved - if paged { 136.0 } else { 66.0 }) / (end - start) as f32)
        .clamp(58.0, 188.0);
    let mut children = Vec::new();
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
            height: Dimension::Length(54.0),
        },
        ..row(0.0)
    })));
    if decorations.mode == WindowDecorationMode::Client {
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
    } else {
        children.push(view(fixed(reserved, 1.0), vec![]));
    }
    Box::new(
        RawView::new(pad(
            Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Length(54.0),
                },
                flex_shrink: 0.0,
                ..row(3.0)
            },
            8.0,
            0.0,
        ))
        .background(t.colors.surface.mix(t.colors.accent, 0.025))
        .with_children(children),
    )
}

pub(crate) fn toolbar(b: &BrowserState, tab: &Tab, width: f32) -> BoxedWidget {
    let t = use_theme();
    let back = b.clone();
    let forward = b.clone();
    let up = b.clone();
    let address = b.clone();
    let refresh = b.clone();
    let list = b.clone();
    let grid = b.clone();
    let parent = match tab.location() {
        Location::Directory(p) => p.parent().map(Path::to_path_buf),
        _ => None,
    };
    let mut items = vec![
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
    ];
    items.push(Box::new(
        TextInput::controlled(&tab.address)
            .layout(Style {
                flex_grow: 1.0,
                flex_basis: Dimension::Length(0.0),
                min_size: creamui_core::layout::Size {
                    width: Dimension::Length(40.0),
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
    let search_width = if width < 650.0 { 132.0 } else { 210.0 };
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
        items.push(icon_button(b, "list", true, !tab.grid, move || {
            list.edit(|t| t.grid = false)
        }));
        items.push(icon_button(b, "grid", true, tab.grid, move || {
            grid.edit(|t| t.grid = true)
        }));
    }
    view(
        pad(
            Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Length(54.0),
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

pub(crate) fn statusbar(b: &BrowserState, tab: &Tab) -> BoxedWidget {
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
    if b.context.is_manager() && selection.is_some() {
        let state = b.clone();
        children.push(action("Open", false, true, move || state.accept()));
    }
    Box::new(
        RawView::new(pad(
            Style {
                size: creamui_core::layout::Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Length(38.0),
                },
                flex_shrink: 0.0,
                ..row(12.0)
            },
            16.0,
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
