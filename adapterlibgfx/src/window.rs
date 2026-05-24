use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::api::{Adapter, AdapterConfig};
use crate::renderer::{RenderError, WgpuRenderer};

pub trait FrameProducer {
    fn cursor_visible(&self) -> bool {
        true
    }

    fn window_decorations(&self) -> bool {
        true
    }

    fn resize(&mut self, _width: u32, _height: u32) {}

    fn handle_input(&mut self, _event: InputEvent) {}

    fn build_frame(&mut self, adapter: &mut Adapter);
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputEvent {
    CursorMoved { x: f32, y: f32 },
    MouseButton {
        button: InputMouseButton,
        state: InputButtonState,
    },
    MouseWheel { x: f32, y: f32 },
    KeyPressed(InputKey),
    DigitPressed(u8),
    ModifiersChanged { ctrl: bool },
    EscapePressed,
}

pub struct WgpuWindowApp<P> {
    title: String,
    config: AdapterConfig,
    producer: P,
    adapter: Adapter,
    window: Option<Arc<Window>>,
    renderer: Option<WgpuRenderer>,
    last_error: Option<RenderError>,
}

pub struct WgpuTwoWindowApp<P, S> {
    primary_title: String,
    secondary_title: String,
    primary_config: AdapterConfig,
    secondary_config: AdapterConfig,
    primary_producer: P,
    secondary_producer: S,
    primary_adapter: Adapter,
    secondary_adapter: Adapter,
    primary_window: Option<Arc<Window>>,
    secondary_window: Option<Arc<Window>>,
    primary_renderer: Option<WgpuRenderer>,
    secondary_renderer: Option<WgpuRenderer>,
    last_error: Option<RenderError>,
}

pub struct WgpuThreeWindowApp<P, S, T> {
    primary_title: String,
    secondary_title: String,
    tertiary_title: String,
    primary_config: AdapterConfig,
    secondary_config: AdapterConfig,
    tertiary_config: AdapterConfig,
    primary_producer: P,
    secondary_producer: S,
    tertiary_producer: T,
    primary_adapter: Adapter,
    secondary_adapter: Adapter,
    tertiary_adapter: Adapter,
    primary_window: Option<Arc<Window>>,
    secondary_window: Option<Arc<Window>>,
    tertiary_window: Option<Arc<Window>>,
    primary_renderer: Option<WgpuRenderer>,
    secondary_renderer: Option<WgpuRenderer>,
    tertiary_renderer: Option<WgpuRenderer>,
    last_error: Option<RenderError>,
}

pub struct WgpuFourWindowApp<P, S, T, U> {
    primary_title: String,
    secondary_title: String,
    tertiary_title: String,
    quaternary_title: String,
    primary_config: AdapterConfig,
    secondary_config: AdapterConfig,
    tertiary_config: AdapterConfig,
    quaternary_config: AdapterConfig,
    primary_producer: P,
    secondary_producer: S,
    tertiary_producer: T,
    quaternary_producer: U,
    primary_adapter: Adapter,
    secondary_adapter: Adapter,
    tertiary_adapter: Adapter,
    quaternary_adapter: Adapter,
    primary_window: Option<Arc<Window>>,
    secondary_window: Option<Arc<Window>>,
    tertiary_window: Option<Arc<Window>>,
    quaternary_window: Option<Arc<Window>>,
    primary_renderer: Option<WgpuRenderer>,
    secondary_renderer: Option<WgpuRenderer>,
    tertiary_renderer: Option<WgpuRenderer>,
    quaternary_renderer: Option<WgpuRenderer>,
    last_error: Option<RenderError>,
}

impl<P> WgpuWindowApp<P> {
    pub fn new(title: impl Into<String>, config: AdapterConfig, producer: P) -> Self {
        Self {
            title: title.into(),
            config,
            producer,
            adapter: Adapter::new(config),
            window: None,
            renderer: None,
            last_error: None,
        }
    }

    pub fn run(mut self) -> Result<(), winit::error::EventLoopError>
    where
        P: FrameProducer + 'static,
    {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)
    }
}

impl<P, S> WgpuTwoWindowApp<P, S> {
    pub fn new(
        primary_title: impl Into<String>,
        primary_config: AdapterConfig,
        primary_producer: P,
        secondary_title: impl Into<String>,
        secondary_config: AdapterConfig,
        secondary_producer: S,
    ) -> Self {
        Self {
            primary_title: primary_title.into(),
            secondary_title: secondary_title.into(),
            primary_config,
            secondary_config,
            primary_producer,
            secondary_producer,
            primary_adapter: Adapter::new(primary_config),
            secondary_adapter: Adapter::new(secondary_config),
            primary_window: None,
            secondary_window: None,
            primary_renderer: None,
            secondary_renderer: None,
            last_error: None,
        }
    }

    pub fn run(mut self) -> Result<(), winit::error::EventLoopError>
    where
        P: FrameProducer + 'static,
        S: FrameProducer + 'static,
    {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)
    }
}

impl<P, S, T> WgpuThreeWindowApp<P, S, T> {
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
    ) -> Self {
        Self {
            primary_title: primary_title.into(),
            secondary_title: secondary_title.into(),
            tertiary_title: tertiary_title.into(),
            primary_config,
            secondary_config,
            tertiary_config,
            primary_producer,
            secondary_producer,
            tertiary_producer,
            primary_adapter: Adapter::new(primary_config),
            secondary_adapter: Adapter::new(secondary_config),
            tertiary_adapter: Adapter::new(tertiary_config),
            primary_window: None,
            secondary_window: None,
            tertiary_window: None,
            primary_renderer: None,
            secondary_renderer: None,
            tertiary_renderer: None,
            last_error: None,
        }
    }

    pub fn run(mut self) -> Result<(), winit::error::EventLoopError>
    where
        P: FrameProducer + 'static,
        S: FrameProducer + 'static,
        T: FrameProducer + 'static,
    {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)
    }
}

impl<P, S, T, U> WgpuFourWindowApp<P, S, T, U> {
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
    ) -> Self {
        Self {
            primary_title: primary_title.into(),
            secondary_title: secondary_title.into(),
            tertiary_title: tertiary_title.into(),
            quaternary_title: quaternary_title.into(),
            primary_config,
            secondary_config,
            tertiary_config,
            quaternary_config,
            primary_producer,
            secondary_producer,
            tertiary_producer,
            quaternary_producer,
            primary_adapter: Adapter::new(primary_config),
            secondary_adapter: Adapter::new(secondary_config),
            tertiary_adapter: Adapter::new(tertiary_config),
            quaternary_adapter: Adapter::new(quaternary_config),
            primary_window: None,
            secondary_window: None,
            tertiary_window: None,
            quaternary_window: None,
            primary_renderer: None,
            secondary_renderer: None,
            tertiary_renderer: None,
            quaternary_renderer: None,
            last_error: None,
        }
    }

    pub fn run(mut self) -> Result<(), winit::error::EventLoopError>
    where
        P: FrameProducer + 'static,
        S: FrameProducer + 'static,
        T: FrameProducer + 'static,
        U: FrameProducer + 'static,
    {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)
    }
}

impl<P: FrameProducer> ApplicationHandler for WgpuWindowApp<P> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_resizable(true)
            .with_decorations(self.producer.window_decorations())
            .with_inner_size(winit::dpi::PhysicalSize::new(
                self.config.width,
                self.config.height,
            ));
        let Ok(window) = event_loop.create_window(attrs) else {
            event_loop.exit();
            return;
        };
        let window = Arc::new(window);
        window.set_cursor_visible(self.producer.cursor_visible());
        match WgpuRenderer::new(window.clone()) {
            Ok(renderer) => {
                self.renderer = Some(renderer);
                self.window = Some(window);
            }
            Err(err) => {
                self.last_error = Some(err);
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map(|w| w.id()) != Some(window_id) {
            return;
        }

        if let Some(input) = input_event(&event) {
            self.producer.handle_input(input);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.adapter.resize(size.width, size.height);
                self.producer.resize(size.width, size.height);
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                self.producer.build_frame(&mut self.adapter);
                if let (Some(renderer), Some(frame)) =
                    (self.renderer.as_mut(), self.adapter.take_last_frame())
                {
                    if let Err(err) = renderer.render_frame(self.adapter.textures(), &frame) {
                        self.last_error = Some(err);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl<P: FrameProducer, S: FrameProducer> ApplicationHandler for WgpuTwoWindowApp<P, S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.primary_window.is_none() {
            let Some((window, renderer)) = create_window_renderer(
                event_loop,
                &self.primary_title,
                self.primary_config,
                self.primary_producer.cursor_visible(),
                self.primary_producer.window_decorations(),
            ) else {
                event_loop.exit();
                return;
            };
            self.primary_window = Some(window);
            self.primary_renderer = Some(renderer);
        }

        if self.secondary_window.is_none() {
            let Some((window, renderer)) = create_window_renderer(
                event_loop,
                &self.secondary_title,
                self.secondary_config,
                self.secondary_producer.cursor_visible(),
                self.secondary_producer.window_decorations(),
            ) else {
                event_loop.exit();
                return;
            };
            self.secondary_window = Some(window);
            self.secondary_renderer = Some(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.primary_window.as_ref().map(|w| w.id()) == Some(window_id) {
            handle_window_event(
                event_loop,
                event,
                &mut self.primary_producer,
                &mut self.primary_adapter,
                &mut self.primary_renderer,
                &mut self.last_error,
            );
        } else if self.secondary_window.as_ref().map(|w| w.id()) == Some(window_id) {
            handle_window_event(
                event_loop,
                event,
                &mut self.secondary_producer,
                &mut self.secondary_adapter,
                &mut self.secondary_renderer,
                &mut self.last_error,
            );
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.primary_window {
            window.request_redraw();
        }
        if let Some(window) = &self.secondary_window {
            window.request_redraw();
        }
    }
}

impl<P: FrameProducer, S: FrameProducer, T: FrameProducer> ApplicationHandler
    for WgpuThreeWindowApp<P, S, T>
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.primary_window.is_none() {
            let Some((window, renderer)) = create_window_renderer(
                event_loop,
                &self.primary_title,
                self.primary_config,
                self.primary_producer.cursor_visible(),
                self.primary_producer.window_decorations(),
            ) else {
                event_loop.exit();
                return;
            };
            self.primary_window = Some(window);
            self.primary_renderer = Some(renderer);
        }

        if self.secondary_window.is_none() {
            let Some((window, renderer)) = create_window_renderer(
                event_loop,
                &self.secondary_title,
                self.secondary_config,
                self.secondary_producer.cursor_visible(),
                self.secondary_producer.window_decorations(),
            ) else {
                event_loop.exit();
                return;
            };
            self.secondary_window = Some(window);
            self.secondary_renderer = Some(renderer);
        }

        if self.tertiary_window.is_none() {
            let Some((window, renderer)) = create_window_renderer(
                event_loop,
                &self.tertiary_title,
                self.tertiary_config,
                self.tertiary_producer.cursor_visible(),
                self.tertiary_producer.window_decorations(),
            ) else {
                event_loop.exit();
                return;
            };
            self.tertiary_window = Some(window);
            self.tertiary_renderer = Some(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.primary_window.as_ref().map(|w| w.id()) == Some(window_id) {
            handle_window_event(
                event_loop,
                event,
                &mut self.primary_producer,
                &mut self.primary_adapter,
                &mut self.primary_renderer,
                &mut self.last_error,
            );
        } else if self.secondary_window.as_ref().map(|w| w.id()) == Some(window_id) {
            handle_window_event(
                event_loop,
                event,
                &mut self.secondary_producer,
                &mut self.secondary_adapter,
                &mut self.secondary_renderer,
                &mut self.last_error,
            );
        } else if self.tertiary_window.as_ref().map(|w| w.id()) == Some(window_id) {
            handle_window_event(
                event_loop,
                event,
                &mut self.tertiary_producer,
                &mut self.tertiary_adapter,
                &mut self.tertiary_renderer,
                &mut self.last_error,
            );
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.primary_window {
            window.request_redraw();
        }
        if let Some(window) = &self.secondary_window {
            window.request_redraw();
        }
        if let Some(window) = &self.tertiary_window {
            window.request_redraw();
        }
    }
}

impl<P: FrameProducer, S: FrameProducer, T: FrameProducer, U: FrameProducer> ApplicationHandler
    for WgpuFourWindowApp<P, S, T, U>
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.primary_window.is_none() {
            let Some((window, renderer)) = create_window_renderer(
                event_loop,
                &self.primary_title,
                self.primary_config,
                self.primary_producer.cursor_visible(),
                self.primary_producer.window_decorations(),
            ) else {
                event_loop.exit();
                return;
            };
            self.primary_window = Some(window);
            self.primary_renderer = Some(renderer);
        }

        if self.secondary_window.is_none() {
            let Some((window, renderer)) = create_window_renderer(
                event_loop,
                &self.secondary_title,
                self.secondary_config,
                self.secondary_producer.cursor_visible(),
                self.secondary_producer.window_decorations(),
            ) else {
                event_loop.exit();
                return;
            };
            self.secondary_window = Some(window);
            self.secondary_renderer = Some(renderer);
        }

        if self.tertiary_window.is_none() {
            let Some((window, renderer)) = create_window_renderer(
                event_loop,
                &self.tertiary_title,
                self.tertiary_config,
                self.tertiary_producer.cursor_visible(),
                self.tertiary_producer.window_decorations(),
            ) else {
                event_loop.exit();
                return;
            };
            self.tertiary_window = Some(window);
            self.tertiary_renderer = Some(renderer);
        }

        if self.quaternary_window.is_none() {
            let Some((window, renderer)) = create_window_renderer(
                event_loop,
                &self.quaternary_title,
                self.quaternary_config,
                self.quaternary_producer.cursor_visible(),
                self.quaternary_producer.window_decorations(),
            ) else {
                event_loop.exit();
                return;
            };
            self.quaternary_window = Some(window);
            self.quaternary_renderer = Some(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.primary_window.as_ref().map(|w| w.id()) == Some(window_id) {
            handle_window_event(
                event_loop,
                event,
                &mut self.primary_producer,
                &mut self.primary_adapter,
                &mut self.primary_renderer,
                &mut self.last_error,
            );
        } else if self.secondary_window.as_ref().map(|w| w.id()) == Some(window_id) {
            handle_window_event(
                event_loop,
                event,
                &mut self.secondary_producer,
                &mut self.secondary_adapter,
                &mut self.secondary_renderer,
                &mut self.last_error,
            );
        } else if self.tertiary_window.as_ref().map(|w| w.id()) == Some(window_id) {
            handle_window_event(
                event_loop,
                event,
                &mut self.tertiary_producer,
                &mut self.tertiary_adapter,
                &mut self.tertiary_renderer,
                &mut self.last_error,
            );
        } else if self.quaternary_window.as_ref().map(|w| w.id()) == Some(window_id) {
            handle_window_event(
                event_loop,
                event,
                &mut self.quaternary_producer,
                &mut self.quaternary_adapter,
                &mut self.quaternary_renderer,
                &mut self.last_error,
            );
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.primary_window {
            window.request_redraw();
        }
        if let Some(window) = &self.secondary_window {
            window.request_redraw();
        }
        if let Some(window) = &self.tertiary_window {
            window.request_redraw();
        }
        if let Some(window) = &self.quaternary_window {
            window.request_redraw();
        }
    }
}

fn create_window_renderer(
    event_loop: &ActiveEventLoop,
    title: &str,
    config: AdapterConfig,
    cursor_visible: bool,
    decorations: bool,
) -> Option<(Arc<Window>, WgpuRenderer)> {
    let attrs = WindowAttributes::default()
        .with_title(title)
        .with_resizable(true)
        .with_decorations(decorations)
        .with_inner_size(winit::dpi::PhysicalSize::new(config.width, config.height));
    let window = event_loop.create_window(attrs).ok()?;
    let window = Arc::new(window);
    window.set_cursor_visible(cursor_visible);
    let renderer = WgpuRenderer::new(window.clone()).ok()?;
    Some((window, renderer))
}

fn handle_window_event<P: FrameProducer>(
    event_loop: &ActiveEventLoop,
    event: WindowEvent,
    producer: &mut P,
    adapter: &mut Adapter,
    renderer: &mut Option<WgpuRenderer>,
    last_error: &mut Option<RenderError>,
) {
    if let Some(input) = input_event(&event) {
        producer.handle_input(input);
    }

    match event {
        WindowEvent::CloseRequested => event_loop.exit(),
        WindowEvent::Resized(size) => {
            adapter.resize(size.width, size.height);
            producer.resize(size.width, size.height);
            if let Some(renderer) = renderer.as_mut() {
                renderer.resize(size);
            }
        }
        WindowEvent::RedrawRequested => {
            producer.build_frame(adapter);
            if let (Some(renderer), Some(frame)) = (renderer.as_mut(), adapter.take_last_frame()) {
                if let Err(err) = renderer.render_frame(adapter.textures(), &frame) {
                    *last_error = Some(err);
                }
            }
        }
        _ => {}
    }
}

fn input_event(event: &WindowEvent) -> Option<InputEvent> {
    match event {
        WindowEvent::CursorMoved { position, .. } => Some(InputEvent::CursorMoved {
            x: position.x as f32,
            y: position.y as f32,
        }),
        WindowEvent::MouseInput { state, button, .. } => Some(InputEvent::MouseButton {
            button: mouse_button(*button),
            state: button_state(*state),
        }),
        WindowEvent::MouseWheel { delta, .. } => {
            let (x, y) = match delta {
                MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                MouseScrollDelta::PixelDelta(position) => {
                    (position.x as f32 / 32.0, position.y as f32 / 32.0)
                }
            };
            Some(InputEvent::MouseWheel { x, y })
        }
        WindowEvent::ModifiersChanged(modifiers) => Some(InputEvent::ModifiersChanged {
            ctrl: modifiers.state().control_key(),
        }),
        WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
            match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyU) => Some(InputEvent::KeyPressed(InputKey::U)),
                PhysicalKey::Code(KeyCode::KeyJ) => Some(InputEvent::KeyPressed(InputKey::J)),
                PhysicalKey::Code(KeyCode::KeyH) => Some(InputEvent::KeyPressed(InputKey::H)),
                PhysicalKey::Code(KeyCode::KeyK) => Some(InputEvent::KeyPressed(InputKey::K)),
                _ => match &event.logical_key {
                Key::Named(NamedKey::Escape) => Some(InputEvent::EscapePressed),
                Key::Character(text) => text
                    .as_str()
                    .parse::<u8>()
                    .ok()
                    .filter(|digit| *digit <= 9)
                    .map(InputEvent::DigitPressed),
                _ => None,
                },
            }
        }
        _ => None,
    }
}

fn button_state(state: ElementState) -> InputButtonState {
    match state {
        ElementState::Pressed => InputButtonState::Pressed,
        ElementState::Released => InputButtonState::Released,
    }
}

fn mouse_button(button: MouseButton) -> InputMouseButton {
    match button {
        MouseButton::Left => InputMouseButton::Left,
        MouseButton::Right => InputMouseButton::Right,
        MouseButton::Middle => InputMouseButton::Middle,
        MouseButton::Back => InputMouseButton::Other(4),
        MouseButton::Forward => InputMouseButton::Other(5),
        MouseButton::Other(value) => InputMouseButton::Other(value),
    }
}
