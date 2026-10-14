use self::{directory::directory_view, home::home_view};
use crate::components::*;
use crate::components::{navigation::*, portal_footer::portal_footer, sidebar::sidebar};
pub(crate) mod directory;
mod home;

pub(crate) fn build(b: &BrowserState, size: Size) -> BoxedWidget {
    creamui_reactive::with_context_scope(|| {
        creamui_reactive::provide_context(b.context.clone());
        build_scoped(b, size)
    })
}

fn build_scoped(b: &BrowserState, size: Size) -> BoxedWidget {
    let t = use_theme();
    let tab = b.tab();
    let sidebar_width = if size.width < 850.0 { 192.0 } else { 236.0 };
    let content_width = (size.width - sidebar_width).max(180.0);
    let content = if tab.location() == &Location::Home {
        home_view(b, &tab, content_width)
    } else {
        directory_view(b, &tab, content_width)
    };
    let scroll = Box::new(
        RawScrollView::controlled(
            Style {
                flex_grow: 1.0,
                flex_basis: Dimension::Length(0.0),
                ..column(0.0)
            },
            tab.scroll.clone(),
        )
        .child(content),
    );
    let mut main = vec![
        tab_bar(b, size.width, sidebar_width),
        toolbar(b, &tab, content_width),
    ];
    if let Some(filters) = crate::components::portal_footer::filter_bar(b) {
        main.push(filters);
    }
    main.push(scroll as BoxedWidget);
    if let Some(message) = b.message.get() {
        main.push(view(
            pad(row(8.0), 24.0, 8.0),
            vec![label(message, 12.0, true)],
        ));
    }
    if use_is_portal().is_some() {
        main.push(portal_footer(b, &tab));
    }
    main.push(statusbar(b, &tab));
    view(
        Style {
            size: creamui_core::layout::Size {
                width: Dimension::Length(size.width),
                height: Dimension::Length(size.height),
            },
            align_items: Some(AlignItems::Stretch),
            ..row(0.0)
        },
        vec![
            Box::new(
                RawView::new(Style {
                    size: creamui_core::layout::Size {
                        width: Dimension::Length(sidebar_width),
                        height: Dimension::Percent(1.0),
                    },
                    flex_shrink: 0.0,
                    ..column(0.0)
                })
                .background(t.colors.surface.mix(t.colors.accent, 0.035))
                .with_children(vec![Box::new(
                    RawScrollView::controlled(
                        Style {
                            flex_grow: 1.0,
                            ..column(0.0)
                        },
                        b.sidebar_scroll.clone(),
                    )
                    .scrollbar(false)
                    .child(sidebar(b, &tab)),
                )]),
            ),
            Box::new(
                RawView::new(Style {
                    flex_grow: 1.0,
                    flex_basis: Dimension::Length(0.0),
                    ..column(0.0)
                })
                .background(t.colors.surface)
                .with_children(main),
            ),
        ],
    )
}
