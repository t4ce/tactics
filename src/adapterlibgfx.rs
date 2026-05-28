#[cfg(not(feature = "trueos-blueprint"))]
pub use ::adapterlibgfx::*;

#[cfg(feature = "trueos-blueprint")]
pub mod vertex {
    pub const RGB_VERTEX_SIZE: usize = 12;
    pub const TEX_VERTEX_SIZE: usize = 20;

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
    pub struct RgbVertex {
        pub x: f32,
        pub y: f32,
        pub color: Rgba8,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct TexVertex {
        pub x: f32,
        pub y: f32,
        pub u: f32,
        pub v: f32,
        pub color: Rgba8,
    }

    pub fn usable_rgb_len(len: usize) -> usize {
        len - (len % RGB_VERTEX_SIZE)
    }

    pub fn usable_tex_len(len: usize) -> usize {
        len - (len % TEX_VERTEX_SIZE)
    }

    pub fn decode_rgb_vertices(bytes: &[u8]) -> Vec<RgbVertex> {
        let usable = usable_rgb_len(bytes.len());
        let mut out = Vec::with_capacity(usable / RGB_VERTEX_SIZE);
        let mut off = 0usize;
        while off + RGB_VERTEX_SIZE <= usable {
            out.push(RgbVertex {
                x: f32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()),
                y: f32::from_le_bytes(bytes[off + 4..off + 8].try_into().unwrap()),
                color: Rgba8::new(
                    bytes[off + 8],
                    bytes[off + 9],
                    bytes[off + 10],
                    bytes[off + 11],
                ),
            });
            off += RGB_VERTEX_SIZE;
        }
        out
    }

    pub fn encode_rgb_vertices(vertices: &[RgbVertex]) -> Vec<trueos::ui2::gfx::RgbVertex> {
        vertices
            .iter()
            .map(|v| trueos::ui2::gfx::RgbVertex::new(v.x, v.y, v.color.array()))
            .collect()
    }
}

#[cfg(feature = "trueos-blueprint")]
pub mod command {
    use super::vertex::RgbVertex;

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
        DrawRgb { vertices: Vec<RgbVertex> },
        DrawTex { tex_id: u32, vertices: Vec<u8> },
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
    use super::vertex::{decode_rgb_vertices, encode_rgb_vertices, usable_rgb_len, usable_tex_len};
    use trueos::ui2::{WindowId, gfx};

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

        pub fn draw_rgb_triangles_no_present(&mut self, bytes: &[u8]) -> i32 {
            if bytes.is_empty() {
                return 0;
            }
            let usable = usable_rgb_len(bytes.len());
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
            self.commands.push(FrameCommand::DrawRgb {
                vertices: decode_rgb_vertices(&bytes[..usable]),
            });
            0
        }

        pub fn draw_tex_triangles_no_present(&mut self, tex_id: u32, bytes: &[u8]) -> i32 {
            if tex_id == 0 {
                return -1;
            }
            if bytes.is_empty() {
                return 0;
            }
            let usable = usable_tex_len(bytes.len());
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
            self.commands.push(FrameCommand::DrawTex {
                tex_id,
                vertices: bytes[..usable].to_vec(),
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
                    FrameCommand::DrawRgb { vertices } => {
                        if target_tex_id == 0 || vertices.is_empty() {
                            continue;
                        }
                        let flipped_vertices;
                        let vertices = if texture_effect == TextureEffect::World {
                            flipped_vertices = flip_rgb_vertices_y(vertices);
                            &flipped_vertices
                        } else {
                            vertices
                        };
                        let vertices = encode_rgb_vertices(vertices);
                        let rc = gfx::draw_rgb_triangles_no_present(&vertices);
                        if rc != 0 {
                            self.mark_surface_backpressure();
                            failed = true;
                            break;
                        }
                    }
                    FrameCommand::DrawTex { tex_id, vertices } => {
                        if target_tex_id == 0 || vertices.is_empty() {
                            continue;
                        }
                        let flipped_vertices;
                        let vertices = if texture_effect == TextureEffect::World {
                            flipped_vertices = flip_tex_vertices_y(vertices);
                            &flipped_vertices
                        } else {
                            vertices
                        };
                        let rc = gfx::draw_tex_triangles_no_present(*tex_id, vertices);
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
                let vertices = fullscreen_tex_vertices();
                if gfx::set_render_target(surface_tex_id) != 0
                    || gfx::draw_tex_triangles_no_present(backbuffer_tex_id, &vertices) != 0
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
                if let Some(window_id) = WindowId::new(self.repaint_window_id) {
                    let _ = window_id.request_repaint();
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

    fn fullscreen_tex_vertices() -> Vec<u8> {
        let mut bytes = Vec::with_capacity(6 * 20);
        for (x, y, u, v) in [
            (-1.0, 1.0, 0.0, 1.0),
            (1.0, 1.0, 1.0, 1.0),
            (1.0, -1.0, 1.0, 0.0),
            (-1.0, 1.0, 0.0, 1.0),
            (1.0, -1.0, 1.0, 0.0),
            (-1.0, -1.0, 0.0, 0.0),
        ] {
            push_tex_vertex(&mut bytes, x, y, u, v, [255, 255, 255, 255]);
        }
        bytes
    }

    fn flip_tex_vertices_y(vertices: &[u8]) -> Vec<u8> {
        const VTX_SIZE: usize = 20;
        let mut out = vertices.to_vec();
        for vertex in out.chunks_exact_mut(VTX_SIZE) {
            let y = -f32::from_le_bytes(vertex[4..8].try_into().unwrap());
            vertex[4..8].copy_from_slice(&y.to_le_bytes());
        }
        out
    }

    fn flip_rgb_vertices_y(vertices: &[super::vertex::RgbVertex]) -> Vec<super::vertex::RgbVertex> {
        vertices
            .iter()
            .map(|vertex| super::vertex::RgbVertex {
                x: vertex.x,
                y: -vertex.y,
                color: vertex.color,
            })
            .collect()
    }

    fn push_tex_vertex(bytes: &mut Vec<u8>, x: f32, y: f32, u: f32, v: f32, color: [u8; 4]) {
        bytes.extend_from_slice(&x.to_le_bytes());
        bytes.extend_from_slice(&y.to_le_bytes());
        bytes.extend_from_slice(&u.to_le_bytes());
        bytes.extend_from_slice(&v.to_le_bytes());
        bytes.extend_from_slice(&color);
    }
}

#[cfg(feature = "trueos-blueprint")]
pub mod window {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use super::api::{Adapter, AdapterConfig};
    use trueos::{platform, ui2};

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

        fn window_decorations(&self) -> bool {
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
                Some(bind_window(
                    &self.primary_title,
                    self.primary_config,
                    &self.primary_producer,
                    &mut self.primary_adapter,
                    31_000,
                    0,
                )?)
            } else {
                None
            };
            let mut secondary = if should_build(&self.secondary_create_request) {
                Some(bind_window(
                    &self.secondary_title,
                    self.secondary_config,
                    &self.secondary_producer,
                    &mut self.secondary_adapter,
                    31_001,
                    40,
                )?)
            } else {
                None
            };
            let mut tertiary = if should_build(&self.tertiary_create_request) {
                Some(bind_window(
                    &self.tertiary_title,
                    self.tertiary_config,
                    &self.tertiary_producer,
                    &mut self.tertiary_adapter,
                    31_002,
                    80,
                )?)
            } else {
                None
            };
            let quaternary = bind_window(
                &self.quaternary_title,
                self.quaternary_config,
                &self.quaternary_producer,
                &mut self.quaternary_adapter,
                31_003,
                120,
            )?;
            let mut quinary = if should_build(&self.quinary_create_request) {
                Some(bind_window(
                    &self.quinary_title,
                    self.quinary_config,
                    &self.quinary_producer,
                    &mut self.quinary_adapter,
                    31_004,
                    160,
                )?)
            } else {
                None
            };
            let mut senary = if should_build(&self.senary_create_request) {
                Some(bind_window(
                    &self.senary_title,
                    self.senary_config,
                    &self.senary_producer,
                    &mut self.senary_adapter,
                    31_005,
                    200,
                )?)
            } else {
                None
            };
            let mut septenary = if should_build(&self.septenary_create_request) {
                Some(bind_window(
                    &self.septenary_title,
                    self.septenary_config,
                    &self.septenary_producer,
                    &mut self.septenary_adapter,
                    31_006,
                    240,
                )?)
            } else {
                None
            };
            let _quaternary = quaternary;

            loop {
                if self
                    .exit_request
                    .as_ref()
                    .is_some_and(|request| request.load(Ordering::Relaxed))
                {
                    return Ok(());
                }

                if should_build(&self.primary_create_request) {
                    if primary.is_none() {
                        primary = Some(bind_window(
                            &self.primary_title,
                            self.primary_config,
                            &self.primary_producer,
                            &mut self.primary_adapter,
                            31_000,
                            0,
                        )?);
                    }
                    self.primary_producer.build_frame(&mut self.primary_adapter);
                }
                if should_build(&self.secondary_create_request) {
                    if secondary.is_none() {
                        secondary = Some(bind_window(
                            &self.secondary_title,
                            self.secondary_config,
                            &self.secondary_producer,
                            &mut self.secondary_adapter,
                            31_001,
                            40,
                        )?);
                    }
                    self.secondary_producer
                        .build_frame(&mut self.secondary_adapter);
                }
                if should_build(&self.tertiary_create_request) {
                    if tertiary.is_none() {
                        tertiary = Some(bind_window(
                            &self.tertiary_title,
                            self.tertiary_config,
                            &self.tertiary_producer,
                            &mut self.tertiary_adapter,
                            31_002,
                            80,
                        )?);
                    }
                    self.tertiary_producer
                        .build_frame(&mut self.tertiary_adapter);
                }
                self.quaternary_producer
                    .build_frame(&mut self.quaternary_adapter);
                if should_build(&self.quinary_create_request) {
                    if quinary.is_none() {
                        quinary = Some(bind_window(
                            &self.quinary_title,
                            self.quinary_config,
                            &self.quinary_producer,
                            &mut self.quinary_adapter,
                            31_004,
                            160,
                        )?);
                    }
                    self.quinary_producer.build_frame(&mut self.quinary_adapter);
                }
                if should_build(&self.senary_create_request) {
                    if senary.is_none() {
                        senary = Some(bind_window(
                            &self.senary_title,
                            self.senary_config,
                            &self.senary_producer,
                            &mut self.senary_adapter,
                            31_005,
                            200,
                        )?);
                    }
                    self.senary_producer.build_frame(&mut self.senary_adapter);
                }
                if should_build(&self.septenary_create_request) {
                    if septenary.is_none() {
                        septenary = Some(bind_window(
                            &self.septenary_title,
                            self.septenary_config,
                            &self.septenary_producer,
                            &mut self.septenary_adapter,
                            31_006,
                            240,
                        )?);
                    }
                    self.septenary_producer
                        .build_frame(&mut self.septenary_adapter);
                }

                platform::poll_once();
                platform::sleep_ms(16);
            }
        }
    }

    fn should_build(request: &Option<Arc<AtomicBool>>) -> bool {
        request
            .as_ref()
            .is_none_or(|request| request.load(Ordering::Relaxed))
    }

    fn bind_window<P: FrameProducer>(
        title: &str,
        config: AdapterConfig,
        producer: &P,
        adapter: &mut Adapter,
        tex_id: u32,
        offset: i32,
    ) -> Result<ui2::SurfaceWindow, &'static str> {
        let window = ui2::SurfaceWindow::create(
            title,
            ui2::Rect {
                x: offset,
                y: offset,
                width: config.width.max(1),
                height: config.height.max(1),
            },
            tex_id,
        )
        .ok_or("failed to create ui2 surface window")?;
        if !producer.window_decorations() {
            let _ = window.id().set_decorations(ui2::WindowDecorationMode::None);
        }
        adapter.bind_surface(window.tex_id(), window.id().raw());
        Ok(window)
    }
}
