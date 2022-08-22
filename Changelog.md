## Unreleased
* Update egui 0.19
* Update examples
* Replace tex_handle_from_* with egui::TextureHandle::from_* (use trait TextureHandleExt required)
* Add TextureHandleExt and ColorImageExt
* Remove gl in favor of painter.gl()

## 0.7.1 - 2022-05-10
* Update egui 0.18.1

## 0.6.0 - 2022-03-31
* Replace GL backend with egui_glow crate.
* Update egui (v0.17).
* Scaling can be set using EguiState::set_visual_scale() instead of the previous DpiScaling.
