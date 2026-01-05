use std::sync::Arc;

use anyhow::anyhow;
use anyrender::{PaintScene as _, render_to_buffer};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::{DocumentConfig, util::Color};
use blitz_html::HtmlDocument;
use blitz_net::Provider;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use image::{ImageBuffer, Rgba};
use peniko::Fill;
use peniko::kurbo::Rect;

pub fn render_html_to_image(
    html: &str,
    width: u32,
    height: u32,
) -> anyhow::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let scale = 1.0;
    let net = Arc::new(Provider::new(None));

    let mut document = HtmlDocument::from_html(
        html,
        DocumentConfig {
            net_provider: Some(Arc::clone(&net) as _),
            viewport: Some(Viewport::new(
                width,
                height,
                scale as f32,
                ColorScheme::Light,
            )),
            ..Default::default()
        },
    );

    loop {
        document.resolve(0.0);
        if net.is_empty() {
            break;
        }
    }

    document.as_mut().resolve(0.0);

    let render_width = (width as f64 * scale) as u32;
    let computed_height = document.as_ref().root_element().final_layout.size.height;
    let render_height = ((computed_height as f64).max(height as f64).min(4000.0) * scale) as u32;

    let buffer = render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| {
            scene.fill(
                Fill::NonZero,
                Default::default(),
                Color::WHITE,
                Default::default(),
                &Rect::new(0.0, 0.0, render_width as f64, render_height as f64),
            );

            paint_scene(scene, document.as_ref(), scale, render_width, render_height);
        },
        render_width,
        render_height,
    );

    ImageBuffer::from_vec(render_width, render_height, buffer)
        .ok_or_else(|| anyhow!("failed to build image from Blitz renderer output"))
}
