//! One decoration policy and one set of client controls for every Dfiles window.

use super::*;
use crate::icons::Icons;
use creamui_reactive::Signal;
use creamui_render::{CompositorIntegrationRequest, WindowDecorations, WindowHandle};
use std::cell::RefCell;

pub(crate) const CONTROLS_WIDTH: f32 = 104.0;
const CONTROLS_MARGIN: f32 = 8.0;
const PENDING_CONTROLS_SPACE: f32 = 128.0;

pub(crate) fn request_hybrid(handle: &WindowHandle) {
    handle.set_compositor_integration(Some(CompositorIntegrationRequest::Hybrid));
}

pub(crate) fn decorations() -> WindowDecorations {
    creamui_reactive::try_use_context::<Signal<WindowDecorations>>()
        .map(|decorations| decorations.get())
        .unwrap_or_default()
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Insets {
    pub(crate) start: f32,
    pub(crate) end: f32,
}

/// Reserve only the part of this row owned by controls. Server decorations
/// live outside the client surface and never consume application content.
pub(crate) fn insets(
    content_x: f32,
    width: f32,
    row_y: f32,
    row_height: f32,
    client_controls: bool,
) -> Insets {
    let decorations = decorations();
    match decorations.mode {
        WindowDecorationMode::Client => {
            return Insets {
                end: if client_controls {
                    CONTROLS_WIDTH + CONTROLS_MARGIN
                } else {
                    0.0
                },
                ..Default::default()
            }
        }
        WindowDecorationMode::Pending => {
            return Insets {
                end: if client_controls {
                    PENDING_CONTROLS_SPACE
                } else {
                    0.0
                },
                ..Default::default()
            }
        }
        WindowDecorationMode::Hybrid => {}
        WindowDecorationMode::Server | WindowDecorationMode::None => return Insets::default(),
    }
    let controls = decorations.controls;
    if controls.width <= 0 || controls.height <= 0 {
        return Insets {
            end: if client_controls {
                PENDING_CONTROLS_SPACE
            } else {
                0.0
            },
            ..Default::default()
        };
    }
    let controls_top = controls.y as f32;
    let controls_bottom = controls_top + controls.height as f32;
    let left = controls.x as f32;
    let right = left + controls.width as f32;
    if controls_top >= row_y + row_height
        || controls_bottom <= row_y
        || left >= content_x + width
        || right <= content_x
    {
        return Insets::default();
    }
    if (left + right) / 2.0 <= content_x + width / 2.0 {
        Insets {
            start: (right + CONTROLS_MARGIN - content_x).clamp(0.0, width),
            ..Default::default()
        }
    } else {
        Insets {
            end: (content_x + width - left + CONTROLS_MARGIN).clamp(0.0, width),
            ..Default::default()
        }
    }
}

/// Every client-decorated window uses the same geometry, icons and actions.
/// Hybrid, Server, Pending and None deliberately return no application buttons.
pub(crate) fn client_controls(
    icons: &Icons,
    window: &Rc<RefCell<Option<WindowHandle>>>,
    maximized: &Signal<bool>,
) -> Option<BoxedWidget> {
    if decorations().mode != WindowDecorationMode::Client {
        return None;
    }
    let theme = use_theme();
    let mut children = Vec::new();
    for (index, icon) in ["minimize", "maximize", "close"].into_iter().enumerate() {
        let window = window.clone();
        let maximized = maximized.clone();
        children.push(Box::new(
            RawButton::new(
                Style {
                    justify_content: Some(JustifyContent::Center),
                    ..fixed(32.0, 32.0)
                },
                move || {
                    if let Some(handle) = window.borrow().as_ref() {
                        match index {
                            0 => handle.minimize(),
                            1 => {
                                let next = !maximized.peek();
                                handle.set_maximized(next);
                                maximized.set(next);
                            }
                            _ => handle.close(),
                        }
                    }
                },
            )
            .background(CLEAR)
            .corner_radius(8.0)
            .hover_style(StateStyle::new().background(theme.colors.surface_hover))
            .focus_style(StateStyle::new().outline(theme.colors.accent, 2.0))
            .child(Box::new(
                Icon::new(icons.get(icon), theme.colors.text_secondary).size(17.0),
            )),
        ) as BoxedWidget);
    }
    Some(Box::new(
        RawView::new(Style {
            gap: row(4.0).gap,
            ..fixed(CONTROLS_WIDTH, 32.0)
        })
        .with_children(children),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use creamui_render::CompositorControls;

    #[test]
    fn negotiated_mode_controls_space_and_ignores_stale_server_rectangles() {
        creamui_reactive::with_context_scope(|| {
            let decorations = Signal::new(WindowDecorations::default());
            creamui_reactive::provide_context(decorations.clone());
            let right = CompositorControls {
                x: 632,
                y: 14,
                width: 112,
                height: 28,
            };
            for (mode, expected) in [
                (
                    WindowDecorationMode::Client,
                    CONTROLS_WIDTH + CONTROLS_MARGIN,
                ),
                (WindowDecorationMode::Hybrid, 136.0),
                (WindowDecorationMode::Server, 0.0),
                (WindowDecorationMode::None, 0.0),
                (WindowDecorationMode::Pending, PENDING_CONTROLS_SPACE),
            ] {
                decorations.set(WindowDecorations {
                    mode,
                    controls: right,
                });
                assert_eq!(
                    insets(0.0, 760.0, 0.0, 64.0, true),
                    Insets {
                        start: 0.0,
                        end: expected
                    },
                    "{mode:?}"
                );
            }
            decorations.set(WindowDecorations {
                mode: WindowDecorationMode::Client,
                controls: right,
            });
            assert_eq!(insets(0.0, 760.0, 54.0, 54.0, false), Insets::default());
            decorations.set(WindowDecorations {
                mode: WindowDecorationMode::Hybrid,
                controls: right,
            });
            assert_eq!(
                insets(236.0, 524.0, 0.0, 54.0, true),
                Insets {
                    start: 0.0,
                    end: 136.0
                }
            );
            assert_eq!(insets(236.0, 524.0, 54.0, 54.0, false), Insets::default());
            decorations.set(WindowDecorations {
                mode: WindowDecorationMode::Hybrid,
                controls: CompositorControls { x: 8, ..right },
            });
            assert_eq!(
                insets(0.0, 760.0, 0.0, 64.0, true),
                Insets {
                    start: 128.0,
                    end: 0.0
                }
            );
            assert_eq!(insets(236.0, 524.0, 0.0, 54.0, true), Insets::default());
            decorations.set(WindowDecorations {
                mode: WindowDecorationMode::Hybrid,
                ..Default::default()
            });
            assert_eq!(
                insets(0.0, 760.0, 0.0, 64.0, true),
                Insets {
                    start: 0.0,
                    end: PENDING_CONTROLS_SPACE
                }
            );
        });
    }
}
