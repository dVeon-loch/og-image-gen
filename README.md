# og-image-gen

Server-side OG image generation via SVG templates. Renders a 1200x630 PNG with a gradient background, noise overlay, and text using the Inter font.

## Usage

```rust
use og_image_gen::{OgHeroParams, OgTextParams, render_og_hero};

let params = OgHeroParams::default();
let png_bytes = render_og_hero(&params)?;
```

All fields have defaults. Override what you need:

```rust
let params = OgHeroParams {
    title: OgTextParams {
        text: "My Project".into(),
        ..Default::default()
    },
    bg_stop_0: "rgb(196, 181, 253)".into(),
    bg_stop_1: "rgb(109, 40, 217)".into(),
    screenshot_b64: Some(og_image_gen::encode_image(&png_file_bytes)),
    ..Default::default()
};
```

## Template

The `og:hero` template (`templates/og_hero.svg`) is a Liquid SVG rendered by `resvg`. It supports:

- Gradient background (`bg_stop_0`, `bg_stop_1`)
- Noise overlay (`noise`, 0.0-1.0)
- Tag line and title text with automatic word wrapping
- Optional right-panel screenshot (`screenshot_b64`)

## Assets

Fonts are embedded at compile time (`assets/inter-400.ttf`, `assets/inter-700.ttf`). No runtime font dependencies.
