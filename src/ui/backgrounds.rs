//! Painted menu backgrounds that are independent of gameplay state.

use macroquad::prelude::*;
use std::cell::RefCell;

thread_local! {
    static TITLE_BACKGROUND: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
}

/// Draw the original title key art as a full-bleed, aspect-cropped backdrop.
/// The texture is embedded so it is available before the asset archive loads.
pub fn draw_title_background() {
    let texture = TITLE_BACKGROUND.with(|cached| {
        let mut cached = cached.borrow_mut();
        if cached.is_none() {
            let texture = Texture2D::from_file_with_format(
                include_bytes!("../../assets/ui/title_background.png"),
                Some(ImageFormat::Png),
            );
            texture.set_filter(FilterMode::Linear);
            *cached = Some(texture);
        }
        cached.as_ref().cloned()
    });

    let Some(texture) = texture else {
        clear_background(BLACK);
        return;
    };
    let w = screen_width();
    let h = screen_height();
    let tex_w = texture.width();
    let tex_h = texture.height();
    let dest_aspect = w / h.max(1.0);
    let tex_aspect = tex_w / tex_h.max(1.0);
    let source = if dest_aspect > tex_aspect {
        let src_h = tex_w / dest_aspect;
        Rect::new(0.0, (tex_h - src_h) / 2.0, tex_w, src_h)
    } else {
        let src_w = tex_h * dest_aspect;
        Rect::new((tex_w - src_w) / 2.0, 0.0, src_w, tex_h)
    };
    draw_texture_ex(
        &texture,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            source: Some(source),
            ..Default::default()
        },
    );
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.24));
    draw_rectangle(0.0, 0.0, w * 0.58, h, Color::new(0.0, 0.0, 0.0, 0.20));
}
