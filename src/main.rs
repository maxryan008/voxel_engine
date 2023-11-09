use crate::database::texture::*;
use crate::database::structs::AppState;
use crate::database::voxel::*;
use crate::database::chunk::*;
use crate::database::sector::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
mod database;
use std::fs;
use std::time::Duration;
use bevy::{
    core_pipeline::{
        experimental::taa::{
            TemporalAntiAliasPlugin,
        }
    },
    asset::{LoadState},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy::time::common_conditions::on_timer;
use bevy::window::{WindowMode};
use bevy_flycam::prelude::*;

fn main() {
    App::new()
        .init_resource::<TextureHandles>()
        .init_resource::<TextureAtlasBuilt>()
        .init_resource::<Universe>()
        .init_resource::<CurrentlyLoaded>()
        .init_resource::<ChunkEntitys>()
        .add_state::<AppState>()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()).set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            }),
            WireframePlugin,
            TemporalAntiAliasPlugin,
        ))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(PlayerPlugin)
        .add_systems(OnEnter(AppState::Setup), load_textures)
        .add_systems(Update, check_textures.run_if(in_state(AppState::Setup)))
        .add_systems(OnEnter(AppState::Finished), build_texture_atlas)
        .add_systems(OnEnter(AppState::Generating), generate_planet)
        .add_systems(Update, load_chunks.run_if(in_state(AppState::Generating)))
        .add_systems(Update, unload_chunks.run_if(in_state(AppState::Generating)).run_if(on_timer(Duration::from_secs(1))))
        .add_systems(Update, chunk_handler.run_if(in_state(AppState::Generating)))
        .run();
}
