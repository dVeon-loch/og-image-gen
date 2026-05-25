use std::sync::Arc;

use base64::Engine as _;
use liquid::object;

#[derive(Debug, thiserror::Error)]
pub enum OgImageError {
    #[error("template render failed: {0}")]
    Template(#[from] liquid::Error),
    #[error("svg parse failed: {0}")]
    Svg(#[from] usvg::Error),
    #[error("png encode failed: {0}")]
    Encode(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct OgTextParams {
    pub text: String,
    pub font_size: u32,
    pub font_weight: u32,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct OgHeroParams {
    pub tag: OgTextParams,
    pub title: OgTextParams,
    pub bg_stop_0: String,
    pub bg_stop_1: String,
    /// 0.0–1.0
    pub noise: String,
    pub canvas_w: u32,
    pub canvas_h: u32,
    /// Base64-encoded PNG/JPEG to show as a right-panel screenshot preview.
    /// Use [`encode_image`] to produce this from raw bytes.
    pub screenshot_b64: Option<String>,
    /// Display width of the screenshot in the right panel. Defaults to 568 (full panel width).
    pub screenshot_w: u32,
    /// Display height of the screenshot in the right panel. Defaults to 542 (full panel height).
    pub screenshot_h: u32,
}

impl Default for OgHeroParams {
    fn default() -> Self {
        Self {
            tag: OgTextParams {
                text: "placeholder.com".into(),
                font_size: 20,
                font_weight: 400,
                color: "#f9fafb".into(),
            },
            title: OgTextParams {
                text: "Lorem Ipsum".into(),
                font_size: 48,
                font_weight: 700,
                color: "#f9fafb".into(),
            },
            bg_stop_0: "rgb(187, 247, 208)".into(),
            bg_stop_1: "rgb(34, 197, 94)".into(),
            noise: "0.1".into(),
            canvas_w: 1200,
            canvas_h: 630,
            screenshot_b64: None,
            screenshot_w: 568,
            screenshot_h: 542,
        }
    }
}

/// Base64-encode raw image bytes for use in [`OgHeroParams::screenshot_b64`].
pub fn encode_image(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}

static INTER_400: &[u8] = include_bytes!("../assets/inter-400.ttf");
static INTER_700: &[u8] = include_bytes!("../assets/inter-700.ttf");
static OG_HERO_TEMPLATE: &str = include_str!("../templates/og_hero.svg");

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Wrap `text` into at most two lines, splitting at the last word boundary
/// before `max_chars` characters per line.
fn wrap_title(text: &str, max_chars: usize) -> (String, String) {
    if text.len() <= max_chars {
        return (text.to_string(), String::new());
    }
    let split_at = text[..max_chars].rfind(' ').unwrap_or(max_chars);
    let line1 = text[..split_at].trim().to_string();
    let line2 = text[split_at..].trim().to_string();
    (line1, line2)
}

pub fn render_og_hero(params: &OgHeroParams) -> Result<Vec<u8>, OgImageError> {
    let has_screenshot = params.screenshot_b64.is_some();

    // Narrower wrap when screenshot occupies the right half
    let wrap_width = if has_screenshot { 18 } else { 28 };
    let (title_line1, title_line2) = wrap_title(&params.title.text, wrap_width);

    let (title_y1, title_y2) = if title_line2.is_empty() {
        (360u32, 0u32)
    } else {
        (310u32, 380u32)
    };

    let screenshot_b64 = params.screenshot_b64.as_deref().unwrap_or("");

    // Right panel spans x 580-1180 (600px wide), full canvas height 630px.
    // Center the screenshot within that area with a small inset.
    let sc_w = params.screenshot_w.min(568);
    let sc_h = params.screenshot_h.min(590);
    let sc_x = 590u32 + (568u32.saturating_sub(sc_w)) / 2;
    let sc_y = 20u32 + (590u32.saturating_sub(sc_h)) / 2;

    let tag_text = xml_escape(&params.tag.text);
    let title_line1 = xml_escape(&title_line1);
    let title_line2 = xml_escape(&title_line2);

    let globals = object!({
        "bg_stop_0":      params.bg_stop_0.as_str(),
        "bg_stop_1":      params.bg_stop_1.as_str(),
        "noise_opacity":  params.noise.as_str(),
        "tag_text":       tag_text.as_str(),
        "tag_size":       params.tag.font_size,
        "tag_weight":     params.tag.font_weight,
        "tag_color":      params.tag.color.as_str(),
        "title_line1":    title_line1.as_str(),
        "title_line2":    title_line2.as_str(),
        "title_size":     params.title.font_size,
        "title_weight":   params.title.font_weight,
        "title_color":    params.title.color.as_str(),
        "title_y1":       title_y1,
        "title_y2":       title_y2,
        "screenshot_b64": screenshot_b64,
        "sc_x":           sc_x,
        "sc_y":           sc_y,
        "sc_w":           sc_w,
        "sc_h":           sc_h,
    });

    let template = liquid::ParserBuilder::with_stdlib()
        .build()?
        .parse(OG_HERO_TEMPLATE)?;
    let svg_str = template.render(&globals)?;

    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_font_data(INTER_400.to_vec());
    fontdb.load_font_data(INTER_700.to_vec());

    let options = usvg::Options {
        fontdb: Arc::new(fontdb),
        ..Default::default()
    };

    let tree = usvg::Tree::from_str(&svg_str, &options)?;

    let mut pixmap = tiny_skia::Pixmap::new(params.canvas_w, params.canvas_h)
        .ok_or_else(|| OgImageError::Encode("failed to create pixmap".into()))?;

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| OgImageError::Encode(e.to_string()))
}
