use adapterlibgfx::api::{Adapter, AdapterConfig};
use adapterlibgfx::command::FrameCommand;
use adapterlibgfx::texture::ASYNC_TEX_STATUS_READY;
use adapterlibgfx::vertex::{Rgba8, TexVertex, decode_rgb_vertices, decode_tex_vertices};

fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_rgb(out: &mut Vec<u8>, x: f32, y: f32, color: Rgba8) {
    push_f32(out, x);
    push_f32(out, y);
    out.extend_from_slice(&[color.r, color.g, color.b, color.a]);
}

fn push_tex(out: &mut Vec<u8>, vertex: TexVertex) {
    push_f32(out, vertex.x);
    push_f32(out, vertex.y);
    push_f32(out, vertex.u);
    push_f32(out, vertex.v);
    out.extend_from_slice(&[
        vertex.color.r,
        vertex.color.g,
        vertex.color.b,
        vertex.color.a,
    ]);
}

#[test]
fn decodes_trueos_vertex_bytes() {
    let mut rgb = Vec::new();
    push_rgb(&mut rgb, -1.0, 0.5, Rgba8::new(1, 2, 3, 4));
    rgb.extend_from_slice(&[0xAA, 0xBB]);
    let decoded = decode_rgb_vertices(&rgb);
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].color, Rgba8::new(1, 2, 3, 4));

    let tex_vertex = TexVertex {
        x: -0.25,
        y: 0.25,
        u: 0.125,
        v: 0.875,
        color: Rgba8::new(255, 128, 64, 32),
    };
    let mut tex = Vec::new();
    push_tex(&mut tex, tex_vertex);
    assert_eq!(decode_tex_vertices(&tex), vec![tex_vertex]);
}

#[test]
fn records_command_flow_for_renderer() {
    let mut adapter = Adapter::new(AdapterConfig {
        width: 320,
        height: 200,
    });
    let pixels = vec![0xFF; 4 * 4 * 4];
    assert_eq!(adapter.upload_texture_rgba_image(42, 4, 4, &pixels), 0);
    assert_eq!(adapter.texture_status(42), ASYNC_TEX_STATUS_READY);

    assert_eq!(adapter.begin_frame(0xE9EEF2), 0);
    assert_eq!(adapter.set_blend_raw(1, 0x0302, 0x0303), 0);

    let mut rgb = Vec::new();
    push_rgb(&mut rgb, -1.0, -1.0, Rgba8::WHITE);
    push_rgb(&mut rgb, 1.0, -1.0, Rgba8::WHITE);
    push_rgb(&mut rgb, 0.0, 1.0, Rgba8::WHITE);
    assert_eq!(adapter.draw_rgb_triangles_no_present(&rgb), 0);

    let frame = adapter.end_frame().unwrap();
    assert_eq!(frame.clear_rgb, 0xE9EEF2);
    assert!(matches!(frame.commands[0], FrameCommand::SetBlend(_)));
    assert!(matches!(frame.commands[1], FrameCommand::DrawRgb { .. }));
    assert_eq!(adapter.last_stats().rgb_draws, 1);
}
