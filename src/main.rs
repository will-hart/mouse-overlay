//! Ensure https://github.com/bevyengine/bevy/issues/7544#issuecomment-2840720210

use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
    time::Duration,
};

use bevy::{
    prelude::*,
    window::{CursorOptions, PrimaryWindow, WindowLevel, WindowMode},
    winit::WinitSettings,
};

#[cfg(target_os = "macos")]
use bevy::window::CompositeAlphaMode;
use mki::{Action, InhibitEvent, Mouse};

fn main() {
    let window = Window {
        transparent: true,
        decorations: false,
        window_level: WindowLevel::AlwaysOnTop,
        #[cfg(target_os = "macos")]
        composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
        cursor_options: CursorOptions {
            hit_test: false,
            ..default()
        },
        ..default()
    };

    App::new()
        // Make it render background as transparent
        .insert_resource(ClearColor(Color::srgba(0.0, 0.0, 0.0, 0.02)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .init_resource::<IndicatorAssets>()
        .init_resource::<GlobalMouseEventQueue>()
        .insert_resource(WinitSettings {
            focused_mode: bevy::winit::UpdateMode::Reactive {
                wait: Duration::from_millis(100),
                react_to_device_events: true,
                react_to_user_events: true,
                react_to_window_events: true,
            },
            unfocused_mode: bevy::winit::UpdateMode::Continuous,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            monitor_event_queue.run_if(resource_exists::<GlobalMouseEventQueue>),
        )
        .run();
}

fn setup(mut commands: Commands, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2d);

    for mut window in &mut windows {
        window.mode =
            WindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current);
    }
}

fn monitor_event_queue(
    mut commands: Commands,
    assets: Res<IndicatorAssets>,
    queue: ResMut<GlobalMouseEventQueue>,
    indicators: Query<(Entity, &MouseIndicator)>,
) {
    if let Ok(mut queue) = queue.0.write() {
        while let Some(event) = queue.pop_front() {
            match event {
                MouseEvent::LeftDown => {
                    info!("Handling spawn left");
                    commands.spawn((MouseIndicator(true), assets.left()));
                }
                MouseEvent::LeftUp => {
                    info!("Handling despawn left");
                    for (entity, indicator) in &indicators {
                        if indicator.0 {
                            commands.entity(entity).despawn();
                        }
                    }
                }
                MouseEvent::RightDown => {
                    trace!("Handling spawn right");
                    commands.spawn((MouseIndicator(false), assets.right()));
                }
                MouseEvent::RightUp => {
                    trace!("Handling despawn right");
                    for (entity, indicator) in &indicators {
                        if !indicator.0 {
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Resource)]
struct IndicatorAssets {
    sheet: Handle<Image>,
}

impl FromWorld for IndicatorAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            sheet: assets.load("indicators.png"),
        }
    }
}

impl IndicatorAssets {
    fn left(&self) -> Sprite {
        Sprite {
            image: self.sheet.clone(),
            rect: Some(Rect::from_corners(Vec2::ZERO, Vec2::new(64.0, 64.0))),
            ..default()
        }
    }
    fn right(&self) -> Sprite {
        Sprite {
            image: self.sheet.clone(),
            rect: Some(Rect::from_corners(
                Vec2::new(64.0, 0.0),
                Vec2::new(128.0, 64.0),
            )),
            ..default()
        }
    }
}

/// true for left
#[derive(Component)]
struct MouseIndicator(bool);

#[derive(Debug)]
enum MouseEvent {
    LeftDown,
    LeftUp,
    RightDown,
    RightUp,
}

#[derive(Resource, Debug)]
struct GlobalMouseEventQueue(Arc<RwLock<VecDeque<MouseEvent>>>);

impl Default for GlobalMouseEventQueue {
    fn default() -> Self {
        let registry = Arc::new(RwLock::new(VecDeque::new()));

        let left_registry = registry.clone();
        Mouse::Left.act_on(Action {
            callback: Box::new(move |_e, s| {
                if s == mki::State::Released {
                    trace!("Queueing left up");
                    left_registry.write().unwrap().push_back(MouseEvent::LeftUp);
                } else if s == mki::State::Pressed {
                    trace!("Queueing left down");
                    left_registry
                        .write()
                        .unwrap()
                        .push_back(MouseEvent::LeftDown);
                }
            }),
            inhibit: InhibitEvent::No,
            defer: true,
            sequencer: false,
        });

        let right_registry = registry.clone();
        Mouse::Right.act_on(Action {
            callback: Box::new(move |_e, s| {
                if s == mki::State::Released {
                    trace!("Queueing right up");
                    right_registry
                        .write()
                        .unwrap()
                        .push_back(MouseEvent::RightUp);
                } else if s == mki::State::Pressed {
                    trace!("Queueing right down");
                    right_registry
                        .write()
                        .unwrap()
                        .push_back(MouseEvent::RightDown);
                }
            }),
            inhibit: InhibitEvent::No,
            defer: true,
            sequencer: false,
        });

        Self(registry.clone())
    }
}
