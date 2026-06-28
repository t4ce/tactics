#[cfg(not(feature = "trueos-blueprint"))]
pub use adapterlibgfx::*;

#[cfg(feature = "trueos-blueprint")]
mod ui3_frame {
    pub(crate) mod gfx {
        pub(crate) use trueos::ui3::gfx::*;
    }

    pub(crate) use trueos::ui3::frame::{Frame as FrameWindow, FrameBounds as FrameRect, FrameId};
}

#[cfg(feature = "trueos-blueprint")]
pub mod records {
    pub const SOLID_RECT_SIZE: usize = 20;
    pub const SPRITE_QUAD_SIZE: usize = 68;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Rgba8 {
        pub r: u8,
        pub g: u8,
        pub b: u8,
        pub a: u8,
    }

    impl Rgba8 {
        pub const WHITE: Self = Self::new(255, 255, 255, 255);

        pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
            Self { r, g, b, a }
        }

        pub const fn array(self) -> [u8; 4] {
            [self.r, self.g, self.b, self.a]
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct SolidRect {
        pub x: f32,
        pub y: f32,
        pub w: f32,
        pub h: f32,
        pub color: Rgba8,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct SpriteCorner {
        pub x: f32,
        pub y: f32,
        pub u: f32,
        pub v: f32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct SpriteQuad {
        pub c0: SpriteCorner,
        pub c1: SpriteCorner,
        pub c2: SpriteCorner,
        pub c3: SpriteCorner,
        pub color: Rgba8,
    }

    pub fn usable_solid_len(len: usize) -> usize {
        len - (len % SOLID_RECT_SIZE)
    }

    pub fn usable_sprite_len(len: usize) -> usize {
        len - (len % SPRITE_QUAD_SIZE)
    }

    pub fn decode_solid_rects(bytes: &[u8]) -> Vec<SolidRect> {
        let usable = usable_solid_len(bytes.len());
        let mut out = Vec::with_capacity(usable / SOLID_RECT_SIZE);
        let mut off = 0usize;
        while off + SOLID_RECT_SIZE <= usable {
            out.push(SolidRect {
                x: f32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()),
                y: f32::from_le_bytes(bytes[off + 4..off + 8].try_into().unwrap()),
                w: f32::from_le_bytes(bytes[off + 8..off + 12].try_into().unwrap()),
                h: f32::from_le_bytes(bytes[off + 12..off + 16].try_into().unwrap()),
                color: Rgba8::new(
                    bytes[off + 16],
                    bytes[off + 17],
                    bytes[off + 18],
                    bytes[off + 19],
                ),
            });
            off += SOLID_RECT_SIZE;
        }
        out
    }
}

#[cfg(feature = "trueos-blueprint")]
pub mod command {
    use super::records::SolidRect;

    #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
    pub enum TextureEffect {
        #[default]
        Plain,
        World,
        Blur,
    }

    impl TextureEffect {
        pub fn from_raw(value: u32) -> Self {
            match value {
                1 => Self::World,
                2 => Self::Blur,
                _ => Self::Plain,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ScissorRect {
        pub x: u32,
        pub y: u32,
        pub width: u32,
        pub height: u32,
    }

    #[derive(Clone, Debug)]
    pub enum FrameCommand {
        SetBlend,
        SetSampler,
        SetTextureEffect(TextureEffect),
        SetScissor(Option<ScissorRect>),
        SetRenderTarget(u32),
        DrawSolid { rects: Vec<SolidRect> },
        DrawSprite { tex_id: u32, quads: Vec<u8> },
    }

    #[derive(Clone, Debug)]
    pub struct Frame {
        pub seq: u32,
        pub logical_width: u32,
        pub logical_height: u32,
        pub clear_rgb: u32,
        pub allow_present: bool,
        pub preserve_contents: bool,
        pub commands: Vec<FrameCommand>,
    }
}

#[cfg(feature = "trueos-blueprint")]
pub mod api {
    use std::collections::BTreeMap;

    use super::command::{Frame, FrameCommand, ScissorRect, TextureEffect};
    use super::records::{
        decode_solid_rects, usable_solid_len, usable_sprite_len, Rgba8, SPRITE_QUAD_SIZE,
    };
    use super::ui3_frame::{gfx, FrameId};

    const PRESERVE_RENDER_TARGET_CLEAR_RGB: u32 = u32::MAX;
    const SUPPRESS_REPAINT_WINDOW_ID: u32 = u32::MAX;
    const ASYNC_TEX_STATUS_PENDING: i32 = 1;
    const SURFACE_BACKPRESSURE_FRAMES: u8 = 1;
    const SURFACE_BACKBUFFER_TEX_OFFSET: u32 = 20_000;
    const SURFACE_BACKBUFFER_COUNT: u8 = 3;

    #[derive(Clone, Copy, Debug)]
    pub struct AdapterConfig {
        pub width: u32,
        pub height: u32,
    }

    impl Default for AdapterConfig {
        fn default() -> Self {
            Self {
                width: 1280,
                height: 800,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct FrameStats {
        pub frame_seq: u32,
        pub rgb_draws: u32,
        pub tex_draws: u32,
        pub draw_bytes: u32,
        pub command_count: u32,
    }

    #[derive(Debug)]
    pub struct Adapter {
        pub config: AdapterConfig,
        frame_active: bool,
        frame_seq: u32,
        clear_rgb: u32,
        allow_present: bool,
        preserve_contents: bool,
        commands: Vec<FrameCommand>,
        stats: FrameStats,
        last_frame: Option<Frame>,
        target_tex_id: u32,
        surface_backbuffer_tex_id: u32,
        surface_backbuffer_slot: u8,
        repaint_window_id: u32,
        current_render_target: u32,
        surface_backpressure_frames: u8,
        uploaded: BTreeMap<u32, (u32, u32)>,
    }

    impl Default for Adapter {
        fn default() -> Self {
            Self::new(AdapterConfig::default())
        }
    }

    impl Adapter {
        pub fn new(config: AdapterConfig) -> Self {
            Self {
                config,
                frame_active: false,
                frame_seq: 0,
                clear_rgb: 0,
                allow_present: false,
                preserve_contents: false,
                commands: Vec::new(),
                stats: FrameStats::default(),
                last_frame: None,
                target_tex_id: 0,
                surface_backbuffer_tex_id: 0,
                surface_backbuffer_slot: 0,
                repaint_window_id: 0,
                current_render_target: 0,
                surface_backpressure_frames: 0,
                uploaded: BTreeMap::new(),
            }
        }

        pub fn bind_surface(&mut self, target_tex_id: u32, repaint_window_id: u32) {
            self.target_tex_id = target_tex_id;
            self.surface_backbuffer_tex_id = 0;
            self.surface_backbuffer_slot = 0;
            self.repaint_window_id = repaint_window_id;
            self.uploaded.insert(
                target_tex_id,
                (self.config.width.max(1), self.config.height.max(1)),
            );
        }

        pub fn resize(&mut self, width: u32, height: u32) {
            self.config.width = width.max(1);
            self.config.height = height.max(1);
            if self.target_tex_id != 0 {
                self.uploaded
                    .insert(self.target_tex_id, (self.config.width, self.config.height));
            }
        }

        pub fn begin_frame(&mut self, clear_rgb: u32) -> i32 {
            self.begin_frame_inner(clear_rgb, false, true)
        }

        pub fn begin_frame_no_present(&mut self, clear_rgb: u32) -> i32 {
            self.begin_frame_inner(clear_rgb, false, false)
        }

        pub fn begin_frame_preserve(&mut self, clear_rgb: u32) -> i32 {
            self.begin_frame_inner(clear_rgb, true, true)
        }

        fn begin_frame_inner(
            &mut self,
            clear_rgb: u32,
            preserve_contents: bool,
            allow_present: bool,
        ) -> i32 {
            if self.frame_active {
                return -1;
            }
            if self.surface_backpressure_frames > 0 {
                self.surface_backpressure_frames -= 1;
                return -2;
            }
            if self.target_tex_id != 0
                && self.texture_status(self.target_tex_id) == ASYNC_TEX_STATUS_PENDING
            {
                return -2;
            }
            if self.target_tex_id != 0 && !self.select_surface_backbuffer() {
                return -3;
            }
            self.frame_active = true;
            self.frame_seq = self.frame_seq.wrapping_add(1).max(1);
            self.clear_rgb = clear_rgb & 0x00FF_FFFF;
            self.allow_present = allow_present;
            self.preserve_contents = preserve_contents;
            self.current_render_target = 0;
            self.commands.clear();
            self.stats = FrameStats {
                frame_seq: self.frame_seq,
                ..FrameStats::default()
            };
            0
        }

        pub fn end_frame(&mut self) -> Result<Frame, i32> {
            if !self.frame_active {
                return Err(-1);
            }
            self.frame_active = false;
            let frame = Frame {
                seq: self.frame_seq,
                logical_width: self.config.width,
                logical_height: self.config.height,
                clear_rgb: self.clear_rgb,
                allow_present: self.allow_present,
                preserve_contents: self.preserve_contents,
                commands: core::mem::take(&mut self.commands),
            };
            self.stats.command_count = frame.commands.len().min(u32::MAX as usize) as u32;
            if frame.allow_present {
                self.present_frame(&frame);
            }
            self.last_frame = Some(frame.clone());
            Ok(frame)
        }

        pub fn upload_texture_rgba_image(
            &mut self,
            tex_id: u32,
            width: u32,
            height: u32,
            rgba: &[u8],
        ) -> i32 {
            if gfx::upload_texture_rgba_image_now(tex_id, width, height, rgba) {
                self.uploaded.insert(tex_id, (width.max(1), height.max(1)));
                0
            } else {
                -1
            }
        }

        pub fn texture_status(&self, tex_id: u32) -> i32 {
            gfx::texture_status(tex_id)
        }

        pub fn texture_dimensions(&self, tex_id: u32) -> Option<(u32, u32)> {
            gfx::texture_dimensions(tex_id).or_else(|| self.uploaded.get(&tex_id).copied())
        }

        pub fn set_blend_raw(&mut self, _enabled: u32, _src_rgb: u32, _dst_rgb: u32) -> i32 {
            if self.frame_active {
                self.commands.push(FrameCommand::SetBlend);
            }
            0
        }

        pub fn set_sampler_raw(
            &mut self,
            _wrap_s: u32,
            _wrap_t: u32,
            _min_filter: u32,
            _mag_filter: u32,
        ) -> i32 {
            if self.frame_active {
                self.commands.push(FrameCommand::SetSampler);
            }
            0
        }

        pub fn set_texture_effect(&mut self, effect: TextureEffect) -> i32 {
            if self.frame_active {
                self.commands.push(FrameCommand::SetTextureEffect(effect));
            }
            0
        }

        pub fn set_texture_effect_raw(&mut self, effect: u32) -> i32 {
            self.set_texture_effect(TextureEffect::from_raw(effect))
        }

        pub fn set_scissor(&mut self, rect: Option<ScissorRect>) -> i32 {
            if self.frame_active {
                self.commands.push(FrameCommand::SetScissor(rect));
            }
            0
        }

        pub fn set_render_target(&mut self, tex_id: u32) -> i32 {
            if tex_id != 0 && self.texture_dimensions(tex_id).is_none() {
                return -1;
            }
            if self.frame_active {
                self.commands.push(FrameCommand::SetRenderTarget(tex_id));
            }
            0
        }

        pub fn draw_solid_batch_no_present(&mut self, bytes: &[u8]) -> i32 {
            if bytes.is_empty() {
                return 0;
            }
            let usable = usable_solid_len(bytes.len());
            if usable == 0 {
                return -2;
            }
            if !self.frame_active {
                return -3;
            }
            self.stats.rgb_draws = self.stats.rgb_draws.saturating_add(1);
            self.stats.draw_bytes = self
                .stats
                .draw_bytes
                .saturating_add(usable.min(u32::MAX as usize) as u32);
            self.commands.push(FrameCommand::DrawSolid {
                rects: decode_solid_rects(&bytes[..usable]),
            });
            0
        }

        pub fn draw_sprite_batch_no_present(&mut self, tex_id: u32, bytes: &[u8]) -> i32 {
            if tex_id == 0 {
                return -1;
            }
            if bytes.is_empty() {
                return 0;
            }
            let usable = usable_sprite_len(bytes.len());
            if usable == 0 {
                return -3;
            }
            if !self.frame_active {
                return -4;
            }
            if self.texture_dimensions(tex_id).is_none() {
                return -5;
            }
            self.stats.tex_draws = self.stats.tex_draws.saturating_add(1);
            self.stats.draw_bytes = self
                .stats
                .draw_bytes
                .saturating_add(usable.min(u32::MAX as usize) as u32);
            self.commands.push(FrameCommand::DrawSprite {
                tex_id,
                quads: bytes[..usable].to_vec(),
            });
            0
        }

        pub fn last_stats(&self) -> FrameStats {
            self.stats
        }

        pub fn take_last_frame(&mut self) -> Option<Frame> {
            self.last_frame.take()
        }

        fn present_frame(&mut self, frame: &Frame) {
            let surface_tex_id = self.target_tex_id;
            let backbuffer_tex_id = self.surface_backbuffer_tex_id;
            let default_target_tex_id = if backbuffer_tex_id != 0 {
                backbuffer_tex_id
            } else {
                surface_tex_id
            };
            let mut target_tex_id = default_target_tex_id;
            let mut target_width = frame.logical_width.max(1);
            let mut target_height = frame.logical_height.max(1);
            let mut texture_effect = TextureEffect::Plain;
            let mut failed = false;

            let rc = if frame.preserve_contents {
                gfx::begin_frame_preserve(frame.clear_rgb)
            } else {
                gfx::begin_frame_no_present(frame.clear_rgb)
            };
            if rc != 0 {
                self.mark_surface_backpressure();
                return;
            }

            if target_tex_id != 0 && gfx::set_render_target(target_tex_id) != 0 {
                self.mark_surface_backpressure();
                let _ = gfx::end_frame();
                return;
            }

            for command in frame.commands.iter() {
                match command {
                    FrameCommand::SetRenderTarget(tex_id) => {
                        target_tex_id = if *tex_id == 0 {
                            default_target_tex_id
                        } else {
                            *tex_id
                        };
                        if let Some((w, h)) = self.texture_dimensions(target_tex_id) {
                            target_width = w.max(1);
                            target_height = h.max(1);
                        }
                        if target_tex_id != 0 && gfx::set_render_target(target_tex_id) != 0 {
                            self.mark_surface_backpressure();
                            failed = true;
                            break;
                        }
                    }
                    FrameCommand::DrawSolid { rects } => {
                        if target_tex_id == 0 || rects.is_empty() {
                            continue;
                        }
                        let _ = texture_effect;
                        let rects = rects
                            .iter()
                            .map(|r| gfx::SolidRect::new(r.x, r.y, r.w, r.h, r.color.array()))
                            .collect::<Vec<_>>();
                        let rc = gfx::draw_solid_batch_no_present(&rects);
                        if rc != 0 {
                            self.mark_surface_backpressure();
                            failed = true;
                            break;
                        }
                    }
                    FrameCommand::DrawSprite { tex_id, quads } => {
                        if target_tex_id == 0 || quads.is_empty() {
                            continue;
                        }
                        let _ = texture_effect;
                        let rc = gfx::draw_sprite_batch_no_present(*tex_id, quads);
                        if rc != 0 {
                            self.mark_surface_backpressure();
                            failed = true;
                            break;
                        }
                    }
                    FrameCommand::SetTextureEffect(effect) => {
                        texture_effect = *effect;
                    }
                    FrameCommand::SetBlend => {
                        let _ = gfx::set_blend_raw(1, 0x0302, 0x0303, 0x0302, 0x0303, 0, 0);
                    }
                    FrameCommand::SetSampler => {
                        let _ = gfx::set_sampler_raw(0, 0, 0, 0);
                    }
                    FrameCommand::SetScissor(rect) => {
                        let rc = if let Some(rect) = rect {
                            gfx::set_scissor(rect.x, rect.y, rect.width, rect.height)
                        } else {
                            gfx::clear_scissor()
                        };
                        if rc != 0 {
                            self.mark_surface_backpressure();
                            failed = true;
                            break;
                        }
                    }
                }
            }

            if !failed && surface_tex_id != 0 && backbuffer_tex_id != 0 {
                let (surface_w, surface_h) = self
                    .texture_dimensions(surface_tex_id)
                    .unwrap_or((frame.logical_width.max(1), frame.logical_height.max(1)));
                let sprite_quads = fullscreen_sprite_quad_records(surface_w, surface_h);
                if gfx::set_render_target(surface_tex_id) != 0
                    || gfx::draw_sprite_batch_no_present(backbuffer_tex_id, &sprite_quads) != 0
                {
                    self.mark_surface_backpressure();
                    failed = true;
                }
            }
            if gfx::end_frame() != 0 {
                self.mark_surface_backpressure();
                return;
            }
            if !failed && self.repaint_window_id != SUPPRESS_REPAINT_WINDOW_ID {
                if let Some(frame_id) = FrameId::new(self.repaint_window_id) {
                    let _ = frame_id.request_repaint();
                }
            }
        }

        fn mark_surface_backpressure(&mut self) {
            self.surface_backpressure_frames = self
                .surface_backpressure_frames
                .max(SURFACE_BACKPRESSURE_FRAMES);
        }

        fn select_surface_backbuffer(&mut self) -> bool {
            if self.target_tex_id == 0 {
                self.surface_backbuffer_tex_id = 0;
                return true;
            }
            for offset in 0..SURFACE_BACKBUFFER_COUNT {
                let slot = (self.surface_backbuffer_slot + offset) % SURFACE_BACKBUFFER_COUNT;
                let tex_id = self.surface_backbuffer_tex_id_for_slot(slot);
                if self.texture_status(tex_id) == ASYNC_TEX_STATUS_PENDING {
                    continue;
                }
                if self.ensure_surface_backbuffer(tex_id) {
                    self.surface_backbuffer_tex_id = tex_id;
                    self.surface_backbuffer_slot = (slot + 1) % SURFACE_BACKBUFFER_COUNT;
                    return true;
                }
            }
            false
        }

        fn surface_backbuffer_tex_id_for_slot(&self, slot: u8) -> u32 {
            self.target_tex_id
                .saturating_add(SURFACE_BACKBUFFER_TEX_OFFSET)
                .saturating_add(slot as u32)
        }

        fn ensure_surface_backbuffer(&mut self, tex_id: u32) -> bool {
            let width = self.config.width.max(1);
            let height = self.config.height.max(1);
            if self.texture_dimensions(tex_id) == Some((width, height)) {
                return true;
            }
            let Some(len) = (width as usize)
                .checked_mul(height as usize)
                .and_then(|pixels| pixels.checked_mul(4))
            else {
                return false;
            };
            let pixels = vec![0; len];
            if gfx::upload_texture_rgba_image_now(tex_id, width, height, &pixels) {
                self.uploaded.insert(tex_id, (width, height));
                true
            } else {
                false
            }
        }
    }

    fn fullscreen_sprite_quad_records(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(SPRITE_QUAD_SIZE);
        let w = width.max(1) as f32;
        let h = height.max(1) as f32;
        for (x, y, u, v) in [
            (0.0, 0.0, 0.0, 0.0),
            (w, 0.0, 1.0, 0.0),
            (w, h, 1.0, 1.0),
            (0.0, h, 0.0, 1.0),
        ] {
            push_sprite_corner(&mut bytes, x, y, u, v);
        }
        bytes.extend_from_slice(&Rgba8::WHITE.array());
        bytes
    }

    fn push_sprite_corner(bytes: &mut Vec<u8>, x: f32, y: f32, u: f32, v: f32) {
        bytes.extend_from_slice(&x.to_le_bytes());
        bytes.extend_from_slice(&y.to_le_bytes());
        bytes.extend_from_slice(&u.to_le_bytes());
        bytes.extend_from_slice(&v.to_le_bytes());
    }
}

#[cfg(feature = "trueos-blueprint")]
pub mod window {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use super::api::{Adapter, AdapterConfig};
    use super::ui3_frame::{FrameRect, FrameWindow};
    use trueos::platform;

    const PRIMARY_BUTTON_MASK: u32 = 1;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum InputButtonState {
        Pressed,
        Released,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum InputMouseButton {
        Left,
        Right,
        Middle,
        Other(u16),
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum InputKey {
        U,
        J,
        H,
        K,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum InputEvent {
        CursorMoved {
            x: f32,
            y: f32,
        },
        CursorLeft,
        MouseButton {
            button: InputMouseButton,
            state: InputButtonState,
        },
        MouseWheel {
            x: f32,
            y: f32,
        },
        TextInput(String),
        BackspacePressed,
        EnterPressed,
        KeyPressed(InputKey),
        DigitPressed(u8),
        ModifiersChanged {
            ctrl: bool,
        },
        EscapePressed,
    }

    pub trait FrameProducer {
        fn cursor_visible(&self) -> bool {
            true
        }

        fn window_resizable(&self) -> bool {
            true
        }

        fn window_drag_region(&self) -> bool {
            false
        }

        fn resize(&mut self, _width: u32, _height: u32) {}

        fn handle_input(&mut self, _event: InputEvent) {}

        fn prepare_window_assets(&mut self, _adapter: &mut Adapter) {}

        fn window_assets_ready(&self, _adapter: &Adapter) -> bool {
            true
        }

        fn build_frame(&mut self, adapter: &mut Adapter);
    }

    pub struct WgpuSevenWindowApp<P, S, T, U, V, W, X> {
        primary_title: String,
        secondary_title: String,
        tertiary_title: String,
        quaternary_title: String,
        quinary_title: String,
        senary_title: String,
        septenary_title: String,
        primary_config: AdapterConfig,
        secondary_config: AdapterConfig,
        tertiary_config: AdapterConfig,
        quaternary_config: AdapterConfig,
        quinary_config: AdapterConfig,
        senary_config: AdapterConfig,
        septenary_config: AdapterConfig,
        primary_producer: P,
        secondary_producer: S,
        tertiary_producer: T,
        quaternary_producer: U,
        quinary_producer: V,
        senary_producer: W,
        septenary_producer: X,
        primary_adapter: Adapter,
        secondary_adapter: Adapter,
        tertiary_adapter: Adapter,
        quaternary_adapter: Adapter,
        quinary_adapter: Adapter,
        senary_adapter: Adapter,
        septenary_adapter: Adapter,
        primary_create_request: Option<Arc<AtomicBool>>,
        secondary_create_request: Option<Arc<AtomicBool>>,
        tertiary_create_request: Option<Arc<AtomicBool>>,
        quinary_create_request: Option<Arc<AtomicBool>>,
        senary_create_request: Option<Arc<AtomicBool>>,
        septenary_create_request: Option<Arc<AtomicBool>>,
        exit_request: Option<Arc<AtomicBool>>,
    }

    impl<P, S, T, U, V, W, X> WgpuSevenWindowApp<P, S, T, U, V, W, X> {
        pub fn new(
            primary_title: impl Into<String>,
            primary_config: AdapterConfig,
            primary_producer: P,
            secondary_title: impl Into<String>,
            secondary_config: AdapterConfig,
            secondary_producer: S,
            tertiary_title: impl Into<String>,
            tertiary_config: AdapterConfig,
            tertiary_producer: T,
            quaternary_title: impl Into<String>,
            quaternary_config: AdapterConfig,
            quaternary_producer: U,
            quinary_title: impl Into<String>,
            quinary_config: AdapterConfig,
            quinary_producer: V,
            senary_title: impl Into<String>,
            senary_config: AdapterConfig,
            senary_producer: W,
            septenary_title: impl Into<String>,
            septenary_config: AdapterConfig,
            septenary_producer: X,
        ) -> Self {
            Self {
                primary_title: primary_title.into(),
                secondary_title: secondary_title.into(),
                tertiary_title: tertiary_title.into(),
                quaternary_title: quaternary_title.into(),
                quinary_title: quinary_title.into(),
                senary_title: senary_title.into(),
                septenary_title: septenary_title.into(),
                primary_config,
                secondary_config,
                tertiary_config,
                quaternary_config,
                quinary_config,
                senary_config,
                septenary_config,
                primary_producer,
                secondary_producer,
                tertiary_producer,
                quaternary_producer,
                quinary_producer,
                senary_producer,
                septenary_producer,
                primary_adapter: Adapter::new(primary_config),
                secondary_adapter: Adapter::new(secondary_config),
                tertiary_adapter: Adapter::new(tertiary_config),
                quaternary_adapter: Adapter::new(quaternary_config),
                quinary_adapter: Adapter::new(quinary_config),
                senary_adapter: Adapter::new(senary_config),
                septenary_adapter: Adapter::new(septenary_config),
                primary_create_request: None,
                secondary_create_request: None,
                tertiary_create_request: None,
                quinary_create_request: None,
                senary_create_request: None,
                septenary_create_request: None,
                exit_request: None,
            }
        }

        pub fn defer_primary_until(mut self, request: Arc<AtomicBool>) -> Self {
            self.primary_create_request = Some(request);
            self
        }

        pub fn defer_secondary_until(mut self, request: Arc<AtomicBool>) -> Self {
            self.secondary_create_request = Some(request);
            self
        }

        pub fn defer_tertiary_until(mut self, request: Arc<AtomicBool>) -> Self {
            self.tertiary_create_request = Some(request);
            self
        }

        pub fn defer_quinary_until(mut self, request: Arc<AtomicBool>) -> Self {
            self.quinary_create_request = Some(request);
            self
        }

        pub fn defer_senary_until(mut self, request: Arc<AtomicBool>) -> Self {
            self.senary_create_request = Some(request);
            self
        }

        pub fn defer_septenary_until(mut self, request: Arc<AtomicBool>) -> Self {
            self.septenary_create_request = Some(request);
            self
        }

        pub fn exit_on(mut self, request: Arc<AtomicBool>) -> Self {
            self.exit_request = Some(request);
            self
        }

        pub fn run(mut self) -> Result<(), &'static str>
        where
            P: FrameProducer + 'static,
            S: FrameProducer + 'static,
            T: FrameProducer + 'static,
            U: FrameProducer + 'static,
            V: FrameProducer + 'static,
            W: FrameProducer + 'static,
            X: FrameProducer + 'static,
        {
            let mut primary = if should_build(&self.primary_create_request) {
                self.primary_producer
                    .prepare_window_assets(&mut self.primary_adapter);
                if self
                    .primary_producer
                    .window_assets_ready(&self.primary_adapter)
                {
                    Some(bind_frame(
                        &self.primary_title,
                        self.primary_config,
                        &self.primary_producer,
                        &mut self.primary_adapter,
                        31_000,
                        0,
                    )?)
                } else {
                    None
                }
            } else {
                None
            };
            let mut secondary = if should_build(&self.secondary_create_request) {
                self.secondary_producer
                    .prepare_window_assets(&mut self.secondary_adapter);
                if self
                    .secondary_producer
                    .window_assets_ready(&self.secondary_adapter)
                {
                    Some(bind_frame(
                        &self.secondary_title,
                        self.secondary_config,
                        &self.secondary_producer,
                        &mut self.secondary_adapter,
                        31_001,
                        40,
                    )?)
                } else {
                    None
                }
            } else {
                None
            };
            let mut tertiary = if should_build(&self.tertiary_create_request) {
                self.tertiary_producer
                    .prepare_window_assets(&mut self.tertiary_adapter);
                if self
                    .tertiary_producer
                    .window_assets_ready(&self.tertiary_adapter)
                {
                    Some(bind_frame(
                        &self.tertiary_title,
                        self.tertiary_config,
                        &self.tertiary_producer,
                        &mut self.tertiary_adapter,
                        31_002,
                        80,
                    )?)
                } else {
                    None
                }
            } else {
                None
            };
            self.quaternary_producer
                .prepare_window_assets(&mut self.quaternary_adapter);
            let quaternary = bind_frame(
                &self.quaternary_title,
                self.quaternary_config,
                &self.quaternary_producer,
                &mut self.quaternary_adapter,
                31_003,
                120,
            )?;
            let mut quinary = if should_build(&self.quinary_create_request) {
                self.quinary_producer
                    .prepare_window_assets(&mut self.quinary_adapter);
                if self
                    .quinary_producer
                    .window_assets_ready(&self.quinary_adapter)
                {
                    Some(bind_frame(
                        &self.quinary_title,
                        self.quinary_config,
                        &self.quinary_producer,
                        &mut self.quinary_adapter,
                        31_004,
                        160,
                    )?)
                } else {
                    None
                }
            } else {
                None
            };
            let mut senary = if should_build(&self.senary_create_request) {
                self.senary_producer
                    .prepare_window_assets(&mut self.senary_adapter);
                if self
                    .senary_producer
                    .window_assets_ready(&self.senary_adapter)
                {
                    Some(bind_frame(
                        &self.senary_title,
                        self.senary_config,
                        &self.senary_producer,
                        &mut self.senary_adapter,
                        31_005,
                        200,
                    )?)
                } else {
                    None
                }
            } else {
                None
            };
            let mut septenary = if should_build(&self.septenary_create_request) {
                self.septenary_producer
                    .prepare_window_assets(&mut self.septenary_adapter);
                if self
                    .septenary_producer
                    .window_assets_ready(&self.septenary_adapter)
                {
                    Some(bind_frame(
                        &self.septenary_title,
                        self.septenary_config,
                        &self.septenary_producer,
                        &mut self.septenary_adapter,
                        31_006,
                        240,
                    )?)
                } else {
                    None
                }
            } else {
                None
            };
            let mut input_pump = CursorInputPump::new();

            loop {
                if self
                    .exit_request
                    .as_ref()
                    .is_some_and(|request| request.load(Ordering::Relaxed))
                {
                    close_optional_frame_window(&mut primary);
                    close_optional_frame_window(&mut secondary);
                    close_optional_frame_window(&mut tertiary);
                    close_frame_window(&quaternary);
                    close_optional_frame_window(&mut quinary);
                    close_optional_frame_window(&mut senary);
                    close_optional_frame_window(&mut septenary);
                    return Ok(());
                }

                platform::poll_once();

                hide_unrequested_frame_window(&mut primary, &self.primary_create_request);
                hide_unrequested_frame_window(&mut secondary, &self.secondary_create_request);
                hide_unrequested_frame_window(&mut tertiary, &self.tertiary_create_request);
                hide_unrequested_frame_window(&mut quinary, &self.quinary_create_request);
                hide_unrequested_frame_window(&mut senary, &self.senary_create_request);
                hide_unrequested_frame_window(&mut septenary, &self.septenary_create_request);

                if let Some(window) = primary.as_ref() {
                    dispatch_frame_cursor_events(
                        window,
                        &mut self.primary_producer,
                        &mut input_pump,
                    );
                }
                if let Some(window) = secondary.as_ref() {
                    dispatch_frame_cursor_events(
                        window,
                        &mut self.secondary_producer,
                        &mut input_pump,
                    );
                }
                if let Some(window) = tertiary.as_ref() {
                    dispatch_frame_cursor_events(
                        window,
                        &mut self.tertiary_producer,
                        &mut input_pump,
                    );
                }
                dispatch_frame_cursor_events(
                    &quaternary,
                    &mut self.quaternary_producer,
                    &mut input_pump,
                );
                if let Some(window) = quinary.as_ref() {
                    dispatch_frame_cursor_events(
                        window,
                        &mut self.quinary_producer,
                        &mut input_pump,
                    );
                }
                if let Some(window) = senary.as_ref() {
                    dispatch_frame_cursor_events(
                        window,
                        &mut self.senary_producer,
                        &mut input_pump,
                    );
                }
                if let Some(window) = septenary.as_ref() {
                    dispatch_frame_cursor_events(
                        window,
                        &mut self.septenary_producer,
                        &mut input_pump,
                    );
                }

                if should_build(&self.primary_create_request) {
                    if primary.is_none() {
                        self.primary_producer
                            .prepare_window_assets(&mut self.primary_adapter);
                        if self
                            .primary_producer
                            .window_assets_ready(&self.primary_adapter)
                        {
                            primary = Some(bind_frame(
                                &self.primary_title,
                                self.primary_config,
                                &self.primary_producer,
                                &mut self.primary_adapter,
                                31_000,
                                0,
                            )?);
                        }
                    }
                    if primary.is_some() {
                        self.primary_producer.build_frame(&mut self.primary_adapter);
                    }
                }
                if should_build(&self.secondary_create_request) {
                    if secondary.is_none() {
                        self.secondary_producer
                            .prepare_window_assets(&mut self.secondary_adapter);
                        if self
                            .secondary_producer
                            .window_assets_ready(&self.secondary_adapter)
                        {
                            secondary = Some(bind_frame(
                                &self.secondary_title,
                                self.secondary_config,
                                &self.secondary_producer,
                                &mut self.secondary_adapter,
                                31_001,
                                40,
                            )?);
                        }
                    }
                    if secondary.is_some() {
                        self.secondary_producer
                            .build_frame(&mut self.secondary_adapter);
                    }
                }
                if should_build(&self.tertiary_create_request) {
                    if tertiary.is_none() {
                        self.tertiary_producer
                            .prepare_window_assets(&mut self.tertiary_adapter);
                        if self
                            .tertiary_producer
                            .window_assets_ready(&self.tertiary_adapter)
                        {
                            tertiary = Some(bind_frame(
                                &self.tertiary_title,
                                self.tertiary_config,
                                &self.tertiary_producer,
                                &mut self.tertiary_adapter,
                                31_002,
                                80,
                            )?);
                        }
                    }
                    if tertiary.is_some() {
                        self.tertiary_producer
                            .build_frame(&mut self.tertiary_adapter);
                    }
                }
                self.quaternary_producer
                    .build_frame(&mut self.quaternary_adapter);
                if should_build(&self.quinary_create_request) {
                    if quinary.is_none() {
                        self.quinary_producer
                            .prepare_window_assets(&mut self.quinary_adapter);
                        if self
                            .quinary_producer
                            .window_assets_ready(&self.quinary_adapter)
                        {
                            quinary = Some(bind_frame(
                                &self.quinary_title,
                                self.quinary_config,
                                &self.quinary_producer,
                                &mut self.quinary_adapter,
                                31_004,
                                160,
                            )?);
                        }
                    }
                    if quinary.is_some() {
                        self.quinary_producer.build_frame(&mut self.quinary_adapter);
                    }
                }
                if should_build(&self.senary_create_request) {
                    if senary.is_none() {
                        self.senary_producer
                            .prepare_window_assets(&mut self.senary_adapter);
                        if self
                            .senary_producer
                            .window_assets_ready(&self.senary_adapter)
                        {
                            senary = Some(bind_frame(
                                &self.senary_title,
                                self.senary_config,
                                &self.senary_producer,
                                &mut self.senary_adapter,
                                31_005,
                                200,
                            )?);
                        }
                    }
                    if senary.is_some() {
                        self.senary_producer.build_frame(&mut self.senary_adapter);
                    }
                }
                if should_build(&self.septenary_create_request) {
                    if septenary.is_none() {
                        self.septenary_producer
                            .prepare_window_assets(&mut self.septenary_adapter);
                        if self
                            .septenary_producer
                            .window_assets_ready(&self.septenary_adapter)
                        {
                            septenary = Some(bind_frame(
                                &self.septenary_title,
                                self.septenary_config,
                                &self.septenary_producer,
                                &mut self.septenary_adapter,
                                31_006,
                                240,
                            )?);
                        }
                    }
                    if septenary.is_some() {
                        self.septenary_producer
                            .build_frame(&mut self.septenary_adapter);
                    }
                }

                platform::sleep_ms(16);
            }
        }
    }

    fn should_build(request: &Option<Arc<AtomicBool>>) -> bool {
        request
            .as_ref()
            .is_none_or(|request| request.load(Ordering::Relaxed))
    }

    fn close_frame_window(frame: &FrameWindow) {
        let _ = frame.id().close();
    }

    fn close_optional_frame_window(frame: &mut Option<FrameWindow>) {
        if let Some(frame) = frame.take() {
            close_frame_window(&frame);
        }
    }

    fn hide_optional_frame_window(frame: &mut Option<FrameWindow>) {
        if let Some(frame) = frame.take() {
            let id = frame.id();
            let _ = id.set_position(i32::MIN / 2, i32::MIN / 2);
            let _ = id.set_size(1, 1);
            let _ = frame.leak();
        }
    }

    fn hide_unrequested_frame_window(
        frame: &mut Option<FrameWindow>,
        request: &Option<Arc<AtomicBool>>,
    ) {
        if request
            .as_ref()
            .is_some_and(|request| !request.load(Ordering::Relaxed))
        {
            hide_optional_frame_window(frame);
        }
    }

    #[derive(Clone, Copy, Debug)]
    struct CursorInputEvent {
        x: f32,
        y: f32,
        buttons_down: u32,
        pressed: u32,
        released: u32,
        wheel: i16,
    }

    impl CursorInputEvent {
        fn left_pressed(self) -> bool {
            self.pressed & PRIMARY_BUTTON_MASK != 0
        }
    }

    #[derive(Default)]
    struct CursorInputPump {
        buttons_down_by_slot: Vec<(u32, u32)>,
    }

    impl CursorInputPump {
        fn new() -> Self {
            Self {
                buttons_down_by_slot: Vec::new(),
            }
        }

        fn poll_frame(&mut self, frame: &FrameWindow) -> Vec<CursorInputEvent> {
            let events = frame.id().take_cursor_events(64);
            let mut out = Vec::with_capacity(events.len());
            for event in events {
                let slot_id = event.slot_id.max(1);
                let previous_buttons = self.buttons_down_for_slot(slot_id);
                let pressed = event.buttons_down & !previous_buttons;
                let released = previous_buttons & !event.buttons_down;
                self.set_buttons_down_for_slot(slot_id, event.buttons_down);
                out.push(CursorInputEvent {
                    x: event.x,
                    y: event.y,
                    buttons_down: event.buttons_down,
                    pressed,
                    released,
                    wheel: event.wheel,
                });
            }
            out
        }

        fn buttons_down_for_slot(&self, slot_id: u32) -> u32 {
            self.buttons_down_by_slot
                .iter()
                .find_map(|(id, buttons)| (*id == slot_id).then_some(*buttons))
                .unwrap_or(0)
        }

        fn set_buttons_down_for_slot(&mut self, slot_id: u32, buttons_down: u32) {
            if let Some((_, buttons)) = self
                .buttons_down_by_slot
                .iter_mut()
                .find(|(id, _)| *id == slot_id)
            {
                *buttons = buttons_down;
                return;
            }
            self.buttons_down_by_slot.push((slot_id, buttons_down));
        }
    }

    fn dispatch_cursor_to_producer<P: FrameProducer>(
        frame: &FrameWindow,
        producer: &mut P,
        event: CursorInputEvent,
    ) -> bool {
        producer.handle_input(InputEvent::CursorMoved {
            x: event.x,
            y: event.y,
        });

        if event.left_pressed() && producer.window_drag_region() && frame.id().begin_move() {
            return true;
        }

        dispatch_mouse_button_transitions(producer, event.pressed, InputButtonState::Pressed);
        dispatch_mouse_button_transitions(producer, event.released, InputButtonState::Released);
        if event.wheel != 0 {
            producer.handle_input(InputEvent::MouseWheel {
                x: 0.0,
                y: event.wheel as f32,
            });
        }
        true
    }

    fn dispatch_frame_cursor_events<P: FrameProducer>(
        frame: &FrameWindow,
        producer: &mut P,
        input_pump: &mut CursorInputPump,
    ) {
        for event in input_pump.poll_frame(frame) {
            let _ = dispatch_cursor_to_producer(frame, producer, event);
        }
    }

    fn dispatch_mouse_button_transitions<P: FrameProducer>(
        producer: &mut P,
        mask: u32,
        state: InputButtonState,
    ) {
        if mask & PRIMARY_BUTTON_MASK != 0 {
            producer.handle_input(InputEvent::MouseButton {
                button: InputMouseButton::Left,
                state,
            });
        }
        if mask & (1 << 1) != 0 {
            producer.handle_input(InputEvent::MouseButton {
                button: InputMouseButton::Right,
                state,
            });
        }
        if mask & (1 << 2) != 0 {
            producer.handle_input(InputEvent::MouseButton {
                button: InputMouseButton::Middle,
                state,
            });
        }
    }

    fn bind_frame<P: FrameProducer>(
        title: &str,
        config: AdapterConfig,
        _producer: &P,
        adapter: &mut Adapter,
        tex_id: u32,
        offset: i32,
    ) -> Result<FrameWindow, &'static str> {
        let frame = FrameWindow::create(
            title,
            FrameRect {
                x: offset,
                y: offset,
                width: config.width.max(1),
                height: config.height.max(1),
            },
            tex_id,
        )
        .ok_or("failed to create ui3 frame window")?;
        adapter.bind_surface(frame.tex_id(), frame.id().raw());
        Ok(frame)
    }
}
