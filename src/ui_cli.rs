use super::*;
use adapterlibgfx::api::AdapterConfig;
use adapterlibgfx::window::WgpuSevenWindowApp;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn tactics_window() {
    let world_editor_request = Arc::new(AtomicBool::new(false));
    let world_viewer_request = Arc::new(AtomicBool::new(false));
    let unit_walk_viewer_request = Arc::new(AtomicBool::new(false));
    let icon_viewer_request = Arc::new(AtomicBool::new(false));
    let event_editor_request = Arc::new(AtomicBool::new(false));
    let idle_viewer_request = Arc::new(AtomicBool::new(false));
    let exit_request = Arc::new(AtomicBool::new(false));

    WgpuSevenWindowApp::new(
        "tactics world editor",
        AdapterConfig {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        },
        Game::with_exit_request(exit_request.clone()),
        "tactics world viewer",
        AdapterConfig {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        },
        worldviewer::WorldViewer::with_exit_request(exit_request.clone()),
        "tactics unit walk viewer",
        AdapterConfig {
            width: UNIT_VIEWER_WIDTH,
            height: UNIT_VIEWER_HEIGHT,
        },
        UnitWalkViewer::with_exit_request(exit_request.clone()),
        "tactics loadscreen",
        AdapterConfig {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        },
        loadscreen::LoadScreen::new(
            world_editor_request.clone(),
            world_viewer_request.clone(),
            unit_walk_viewer_request.clone(),
            icon_viewer_request.clone(),
            event_editor_request.clone(),
            idle_viewer_request.clone(),
            exit_request.clone(),
        ),
        "tactics icon viewer",
        AdapterConfig {
            width: ICON_VIEWER_WIDTH,
            height: ICON_VIEWER_HEIGHT,
        },
        IconViewer::with_exit_request(exit_request.clone()),
        "tactics event editor",
        AdapterConfig {
            width: EVENT_EDITOR_WIDTH,
            height: EVENT_EDITOR_HEIGHT,
        },
        EventEditor::with_exit_request(exit_request.clone()),
        "tactics_window",
        AdapterConfig {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        },
        IdleWorldViewer::with_exit_request(exit_request.clone()),
    )
    .defer_primary_until(world_editor_request)
    .defer_secondary_until(world_viewer_request)
    .defer_tertiary_until(unit_walk_viewer_request)
    .defer_quinary_until(icon_viewer_request)
    .defer_senary_until(event_editor_request)
    .defer_septenary_until(idle_viewer_request)
    .exit_on(exit_request)
    .run()
    .expect("window loop failed");
}
