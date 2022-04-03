// from https://github.com/emilk/egui/blob/0.17.0/egui_extras/src/image.rs
// slightly modified for fltk image and fltk svg image

use egui::ColorImage;
use fltk::{
    enums,
    image::SvgImage,
    prelude::{FltkError, ImageExt},
};
use std::sync::Mutex;

/// An image to be shown in egui.
///
/// Load once, and save somewhere in your app state.
///
/// Use the `svg` and `image` features to enable more constructors.
pub struct RetainedEguiImage {
    debug_name: String,
    size: [usize; 2],
    /// Cleared once [`Self::texture`] has been loaded.
    image: Mutex<egui::ColorImage>,
    /// Lazily loaded when we have an egui context.
    texture: Mutex<Option<egui::TextureHandle>>,
}

impl RetainedEguiImage {
    pub fn from_color_image(debug_name: impl Into<String>, image: ColorImage) -> Self {
        Self {
            debug_name: debug_name.into(),
            size: image.size,
            image: Mutex::new(image),
            texture: Default::default(),
        }
    }

    pub fn from_fltk_image<I: ImageExt>(
        debug_name: impl Into<String>,
        image: I,
    ) -> Result<RetainedEguiImage, FltkError> {
        let size = [image.data_w() as usize, image.data_h() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            &image
                .to_rgb()?
                .convert(enums::ColorDepth::Rgba8)?
                .to_rgb_data(),
        );

        Ok(RetainedEguiImage::from_color_image(debug_name, color_image))
    }

    pub fn from_fltk_image_as_ref<I: ImageExt>(
        debug_name: impl Into<String>,
        image: &I,
    ) -> Result<RetainedEguiImage, FltkError> {
        let size = [image.data_w() as usize, image.data_h() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            &image
                .to_rgb()?
                .convert(enums::ColorDepth::Rgba8)?
                .to_rgb_data(),
        );

        Ok(RetainedEguiImage::from_color_image(debug_name, color_image))
    }

    pub fn from_fltk_svg_image_as_ref(
        debug_name: impl Into<String>,
        svg_image: &mut SvgImage,
    ) -> Result<RetainedEguiImage, FltkError> {
        svg_image.normalize();
        let size = [svg_image.data_w() as usize, svg_image.data_h() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            &svg_image
                .to_rgb()?
                .convert(enums::ColorDepth::Rgba8)?
                .to_rgb_data(),
        );

        Ok(RetainedEguiImage::from_color_image(debug_name, color_image))
    }

    pub fn from_fltk_svg_image(
        debug_name: impl Into<String>,
        mut svg_image: SvgImage,
    ) -> Result<RetainedEguiImage, FltkError> {
        svg_image.normalize();
        let size = [svg_image.data_w() as usize, svg_image.data_h() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            &svg_image
                .to_rgb()?
                .convert(enums::ColorDepth::Rgba8)?
                .to_rgb_data(),
        );

        Ok(RetainedEguiImage::from_color_image(debug_name, color_image))
    }

    /// The size of the image data (number of pixels wide/high).
    pub fn size(&self) -> [usize; 2] {
        self.size
    }

    /// The size of the image data (number of pixels wide/high).
    pub fn size_vec2(&self) -> egui::Vec2 {
        let [w, h] = self.size();
        egui::vec2(w as f32, h as f32)
    }

    /// The debug name of the image, e.g. the file name.
    pub fn debug_name(&self) -> &str {
        &self.debug_name
    }

    /// The texture if for this image.
    pub fn texture_id(&self, ctx: &egui::Context) -> egui::TextureId {
        self.texture
            .lock()
            .unwrap()
            .get_or_insert_with(|| {
                let image: &mut ColorImage = &mut self.image.lock().unwrap();
                let image = std::mem::take(image);
                ctx.load_texture(&self.debug_name, image)
            })
            .id()
    }

    /// Show the image with the given maximum size.
    pub fn show_max_size(&self, ui: &mut egui::Ui, max_size: egui::Vec2) -> egui::Response {
        let mut desired_size = self.size_vec2();
        desired_size *= (max_size.x / desired_size.x).min(1.0);
        desired_size *= (max_size.y / desired_size.y).min(1.0);
        self.show_size(ui, desired_size)
    }

    /// Show the image with the original size (one image pixel = one gui point).
    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        self.show_size(ui, self.size_vec2())
    }

    /// Show the image with the given scale factor (1.0 = original size).
    pub fn show_scaled(&self, ui: &mut egui::Ui, scale: f32) -> egui::Response {
        self.show_size(ui, self.size_vec2() * scale)
    }

    /// Show the image with the given size.
    pub fn show_size(&self, ui: &mut egui::Ui, desired_size: egui::Vec2) -> egui::Response {
        // We need to convert the SVG to a texture to display it:
        // Future improvement: tell backend to do mip-mapping of the image to
        // make it look smoother when downsized.
        ui.image(self.texture_id(ui.ctx()), desired_size)
    }
}
