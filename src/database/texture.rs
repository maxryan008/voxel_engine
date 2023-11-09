use bevy::{prelude::*,pbr::wireframe::WireframeConfig,asset::LoadState};
use crate::database::structs::AppState;
use crate::database::settings::WIREFRAME;
use std::fs;
use bevy::{asset::LoadedFolder, prelude::*};
use bevy::app::DynEq;

#[derive(Resource, Default)]
pub struct TextureHandles
{
    pub handles: Vec<UntypedHandle>,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct TextureAtlasBuilt
{
    pub tex: Handle<Image>,
    pub texture_rects: Vec<Rect>,
    pub texture_map: Vec<usize>,
    pub texture_size: Vec2,
}

#[derive(Default, Debug)]
pub struct TextureInfo
{
    pub map: Vec<usize>,
    pub rects: Vec<Rect>,
    pub size: Vec2,
}

#[derive(Resource, Default)]
pub struct TextureFolder(Handle<LoadedFolder>);

pub fn load_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a folder
    commands.insert_resource(TextureFolder(asset_server.load_folder("textures/blocks")));
}

pub fn build_texture_atlas(
    mut wireframe_config: ResMut<WireframeConfig>,
    texture_folder: Res<TextureFolder>,
    mut texture_atlas_data : ResMut<TextureAtlasBuilt>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    texture_handles: Res<TextureHandles>,
    mut textures: ResMut<Assets<Image>>,
)
{
    //enable wireframe?
    wireframe_config.global = WIREFRAME;

    //new texture atlas builder

    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    let loaded_folder = loaded_folders.get(&texture_folder.0).unwrap();
    let mut texture_map: Vec<usize> = vec![0; loaded_folder.handles.len()+1];
    for (handle_iter,handle) in loaded_folder.handles.iter().enumerate() {
        let id = handle.id().typed_unchecked::<Image>();
        let Some(texture) = textures.get(id.clone()) else {
            warn!(
                "{:?} did not resolve to an `Image` asset.",
                handle.path().unwrap()
            );
            continue;
        };
        texture_atlas_builder.add_texture(id, texture);
        //load texs into map
        let handle_clone = handle.clone();
        for (texture_iter, line) in fs::read_to_string("Assets/TexMem").unwrap().lines().enumerate() {
            let expected_texture_asset_path = handle_clone.path();
            let expected_texture_path = expected_texture_asset_path.unwrap().path();
            let expected_texture_file_name = expected_texture_path.file_name().unwrap();
            let expected_texture_str = expected_texture_file_name.to_string_lossy();
            let expected_texture_name = expected_texture_str.split(".png").next().unwrap();
            let current_texture_name = line;
            if current_texture_name == expected_texture_name
            {
                texture_map[texture_iter] = handle_iter;
            }
        }
    }

    let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();
    let texture_atlas_texture = texture_atlas.texture.clone();
    let atlas_handle = texture_atlases.add(texture_atlas.clone());
    texture_atlas_data.tex = texture_atlas.texture.clone();
    texture_atlas_data.texture_rects = texture_atlas.textures.clone();
    texture_atlas_data.texture_map = texture_map;
    texture_atlas_data.texture_size = texture_atlas.size;




    //assign texture atlas
    //texture

    next_state.set(AppState::Generating);
}



pub fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    textures_folder: ResMut<TextureFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&textures_folder.0) {
            next_state.set(AppState::Finished);
        }
    }
}