//! The small, in-process preferences window launched by `dfiles --settings`.

use crate::config::Config;
use creamui_core::{
    layout::{AlignItems, Dimension, FlexDirection, JustifyContent, LengthPercentage, Style},
    BoxedWidget, Size, StateStyle, Styled,
};
use creamui_reactive::Signal;
use creamui_theme::{use_theme, Color};
use creamui_widgets::{
    CUIWindowDragArea, RawButton, RawScrollView, RawView, ScrollController, Switch, Text,
};

const CLEAR: Color = Color::rgba(0, 0, 0, 0);

#[derive(Clone)]
pub(crate) struct State {
    config: Signal<Config>,
    message: Signal<Option<String>>,
    scroll: ScrollController,
}

impl State {
    pub(crate) fn load() -> Self {
        Self {
            config: Signal::new(Config::load()),
            message: Signal::new(None),
            scroll: ScrollController::new(0.0),
        }
    }

    fn persist(&self) {
        match self.config.peek().save() {
            Ok(()) => self
                .message
                .set(Some(format!("Saved to {}", Config::path().display()))),
            Err(error) => self
                .message
                .set(Some(format!("Could not save preferences: {error}"))),
        }
    }
}

pub(crate) fn build(state: &State, size: Size) -> BoxedWidget {
    let theme = use_theme();
    let config = state.config.get();
    let header = header();
    let hidden = preference_row(
        "Show hidden files",
        "Display dot-prefixed files and folders in directory listings.",
        config.explorer.show_hidden,
        {
            let state = state.clone();
            move || {
                state
                    .config
                    .update(|config| config.explorer.show_hidden = !config.explorer.show_hidden);
                state.persist();
            }
        },
    );
    let grid = preference_row(
        "Use grid view by default",
        "Open new folders using icon tiles instead of a detailed list.",
        config.explorer.default_grid_view,
        {
            let state = state.clone();
            move || {
                state.config.update(|config| {
                    config.explorer.default_grid_view = !config.explorer.default_grid_view
                });
                state.persist();
            }
        },
    );
    let reset = {
        let state = state.clone();
        button("Restore defaults", false, move || {
            state.config.set(Config::default());
            state.persist();
        })
    };
    let config_path = Config::path().display().to_string();
    let content = Box::new(
        RawScrollView::controlled(
            Style {
                flex_grow: 1.0,
                flex_basis: Dimension::Length(0.0),
                ..column(0.0)
            },
            state.scroll.clone(),
        )
        .child(Box::new(
            RawView::new(pad(column(22.0), 36.0, 28.0)).with_children(vec![
                section_title("Browsing", "Choose how folders are displayed."),
                card(vec![hidden, grid]),
                section_title(
                    "Configuration",
                    "Preferences apply to newly opened File Explorer windows.",
                ),
                card(vec![
                    Box::new(Text::new("Configuration file").font_size(13.0).bold(true)),
                    Box::new(Text::secondary(config_path).font_size(12.0)),
                    reset,
                ]),
            ]),
        )),
    ) as BoxedWidget;
    let mut children = vec![header, content];
    if let Some(message) = state.message.get() {
        children.push(Box::new(
            RawView::new(pad(row(0.0), 36.0, 12.0))
                .background(theme.colors.selection_background)
                .with_children(vec![Box::new(Text::new(message).font_size(12.0))]),
        ));
    }
    Box::new(
        RawView::new(Style {
            size: creamui_core::layout::Size {
                width: Dimension::Length(size.width),
                height: Dimension::Length(size.height),
            },
            ..column(0.0)
        })
        .background(theme.colors.surface)
        .with_children(children),
    )
}

fn header() -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        RawView::new(pad(
            Style {
                flex_shrink: 0.0,
                size: creamui_core::layout::Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Length(88.0),
                },
                align_items: Some(AlignItems::Center),
                ..row(14.0)
            },
            36.0,
            0.0,
        ))
        .background(theme.colors.surface.mix(theme.colors.accent, 0.035))
        .with_children(vec![
            Box::new(CUIWindowDragArea::new().layout(Style {
                flex_grow: 1.0,
                flex_basis: Dimension::Length(0.0),
                ..row(0.0)
            })),
            Box::new(
                RawView::new(Style {
                    flex_grow: 1.0,
                    flex_basis: Dimension::Length(0.0),
                    ..column(3.0)
                })
                .with_children(vec![
                    Box::new(Text::new("File Explorer").font_size(22.0).bold(true)),
                    Box::new(Text::secondary("Settings").font_size(13.0)),
                ]),
            ),
        ]),
    )
}

fn preference_row(
    title: &str,
    description: &str,
    enabled: bool,
    on_click: impl Fn() + 'static,
) -> BoxedWidget {
    Box::new(
        RawView::new(Style {
            align_items: Some(AlignItems::Center),
            ..row(18.0)
        })
        .with_children(vec![
            Box::new(
                RawView::new(Style {
                    flex_grow: 1.0,
                    flex_basis: Dimension::Length(0.0),
                    ..column(4.0)
                })
                .with_children(vec![
                    Box::new(Text::new(title).font_size(14.0).bold(true)),
                    Box::new(Text::secondary(description).font_size(12.0)),
                ]),
            ),
            Box::new(Switch::new(enabled, on_click)),
        ]),
    )
}

fn section_title(title: &str, description: &str) -> BoxedWidget {
    Box::new(RawView::new(column(4.0)).with_children(vec![
        Box::new(Text::new(title).font_size(16.0).bold(true)),
        Box::new(Text::secondary(description).font_size(12.0)),
    ]))
}

fn card(children: Vec<BoxedWidget>) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        RawView::new(pad(
            Style {
                gap: creamui_core::layout::Size {
                    width: LengthPercentage::Length(14.0),
                    height: LengthPercentage::Length(14.0),
                },
                ..column(14.0)
            },
            18.0,
            18.0,
        ))
        .background(theme.colors.surface_elevated)
        .corner_radius(12.0)
        .with_children(children),
    )
}

fn button(label: &str, primary: bool, click: impl Fn() + 'static) -> BoxedWidget {
    let theme = use_theme();
    Box::new(
        RawButton::new(
            pad(
                Style {
                    align_self: Some(creamui_core::layout::AlignSelf::Start),
                    justify_content: Some(JustifyContent::Center),
                    ..fixed(154.0, 34.0)
                },
                12.0,
                0.0,
            ),
            click,
        )
        .background(if primary { theme.colors.accent } else { CLEAR })
        .border(theme.colors.border, 1.0)
        .corner_radius(8.0)
        .hover_style(StateStyle::new().background(theme.colors.surface_hover))
        .with_children(vec![Box::new(Text::new(label).font_size(12.0))]),
    )
}

fn row(gap: f32) -> Style {
    Style {
        gap: creamui_core::layout::Size {
            width: LengthPercentage::Length(gap),
            height: LengthPercentage::Length(gap),
        },
        ..Default::default()
    }
}

fn column(gap: f32) -> Style {
    Style {
        flex_direction: FlexDirection::Column,
        ..row(gap)
    }
}

fn fixed(width: f32, height: f32) -> Style {
    Style {
        size: creamui_core::layout::Size {
            width: Dimension::Length(width),
            height: Dimension::Length(height),
        },
        flex_shrink: 0.0,
        ..row(0.0)
    }
}

fn pad(mut style: Style, horizontal: f32, vertical: f32) -> Style {
    style.padding = creamui_core::layout::Rect {
        left: LengthPercentage::Length(horizontal),
        right: LengthPercentage::Length(horizontal),
        top: LengthPercentage::Length(vertical),
        bottom: LengthPercentage::Length(vertical),
    };
    style
}
