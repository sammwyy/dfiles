use creamui_image::{ImageData, SvgSize};
use creamui_widgets::{IconImage, IconSource};
use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct Icons(HashMap<&'static str, IconSource>);

impl Icons {
    pub fn load() -> Self {
        let paths = [
            ("home", r#"<path d="m3 10 9-7 9 7v10H4V10m5 10v-7h6v7"/>"#),
            (
                "folder",
                r#"<path d="M3 7V5a2 2 0 0 1 2-2h5l3 3h6a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z"/>"#,
            ),
            (
                "file",
                r#"<path d="M14 3H5v18h14V8l-5-5Zm0 0v6h5M8 13h8m-8 4h6"/>"#,
            ),
            (
                "desktop",
                r#"<rect x="3" y="3" width="18" height="13" rx="2"/><path d="M8 21h8m-4-5v5"/>"#,
            ),
            (
                "download",
                r#"<path d="M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5"/>"#,
            ),
            (
                "image",
                r#"<rect x="3" y="3" width="18" height="18" rx="3"/><circle cx="8" cy="8" r="1"/><path d="m3 17 6-6 4 4 3-3 5 5"/>"#,
            ),
            (
                "video",
                r#"<rect x="3" y="5" width="18" height="14" rx="3"/><path d="m10 9 5 3-5 3Z"/>"#,
            ),
            (
                "music",
                r#"<path d="M10 18V5l10-2v13M10 9l10-2"/><ellipse cx="7" cy="18" rx="3" ry="3"/><ellipse cx="17" cy="16" rx="3" ry="3"/>"#,
            ),
            (
                "drive",
                r#"<path d="m3 14 3-9h12l3 9"/><rect x="3" y="13" width="18" height="7" rx="2"/><path d="M16 17h2M6 17h.01"/>"#,
            ),
            (
                "pin",
                r#"<path d="m15 3 6 6-4 1-4 6-2-2-7 7m1-10 6-4 1-4m-7 8 8 5"/>"#,
            ),
            ("back", r#"<path d="m10 5-7 7 7 7M3 12h18"/>"#),
            ("forward", r#"<path d="m14 5 7 7-7 7M3 12h18"/>"#),
            ("up", r#"<path d="m5 10 7-7 7 7M12 3v18"/>"#),
            (
                "search",
                r#"<circle cx="10" cy="10" r="6.5"/><path d="m15 15 6 6"/>"#,
            ),
            (
                "grid",
                r#"<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>"#,
            ),
            (
                "list",
                r#"<path d="M9 5h12M9 12h12M9 19h12M3 5h1M3 12h1M3 19h1"/>"#,
            ),
            ("plus", r#"<path d="M12 5v14M5 12h14"/>"#),
            ("close", r#"<path d="m6 6 12 12M6 18 18 6"/>"#),
            ("minimize", r#"<path d="M5 12h14"/>"#),
            (
                "maximize",
                r#"<rect x="5" y="5" width="14" height="14" rx="2"/>"#,
            ),
            (
                "clock",
                r#"<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>"#,
            ),
            ("refresh", r#"<path d="M20 10a8 8 0 1 0-1 7M20 3v7h-7"/>"#),
        ];
        let mut icons = HashMap::new();
        for (name, path) in paths {
            let svg = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="#000" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">{path}</svg>"##
            );
            icons.insert(name, decode(&svg, true));
        }
        icons.insert("folder-large", decode(r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="64" viewBox="0 0 80 64"><defs><linearGradient id="f" x2="0.8" y2="1"><stop stop-color="#efcc83"/><stop offset="1" stop-color="#dca751"/></linearGradient></defs><path fill="#c99345" d="M5 14a5 5 0 0 1 5-5h19l7 7h34a5 5 0 0 1 5 5v33H5Z"/><rect x="5" y="20" width="70" height="36" rx="5" fill="url(#f)"/><path d="M10 21h59" stroke="#fff" stroke-opacity=".45"/></svg>"##, false));
        Self(icons)
    }
    pub fn get(&self, name: &str) -> IconSource {
        self.0.get(name).unwrap_or(&self.0["file"]).clone()
    }
}

fn decode(svg: &str, monochrome: bool) -> IconSource {
    let image = ImageData::from_svg(svg.as_bytes(), SvgSize::Max(128)).expect("bundled SVG");
    IconSource::Image(IconImage {
        image: image.image().clone(),
        monochrome,
    })
}
