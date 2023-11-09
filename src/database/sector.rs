use crate::database::texture::*;
use crate::database::structs::AppState;
use crate::database::voxel::*;
use crate::database::example_chunk::STONE_CHUNK;
use crate::database::chunk::*;
use crate::database::settings::*;
use bevy_flycam::FlyCam;
use bevy::{
    core_pipeline::{
        experimental::taa::{
            TemporalAntiAliasPlugin,
        }
    },
    asset::LoadState,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use std::collections::HashMap;
use std::sync::Arc;
use bevy::math::vec3;

#[derive(Default,Resource)]
pub struct Universe
{
    sectors: HashMap<[i32; 3], Sector>,
}

#[derive(Default,Resource, Clone)]
pub struct CurrentlyLoaded
{
    pub sectors: HashMap<[i32; 3], SectorsRendering>
}

#[derive(Default,Resource, Clone)]
pub struct SectorsRendering
{
    pub chunks: HashMap<[i32; 3], Arc<Option<RenderData>>>
}


#[derive(Default,Resource)]
pub struct Sector
{
    chunks: HashMap<[i32; 3], Chunk>
}

pub fn load_chunks
(
    mut commands: Commands,
    player: Query<(&Camera,&Transform), With<FlyCam>>,
    universe: Res<Universe>,
    mut loaded_sectors: ResMut<CurrentlyLoaded>,
    mut chunk_entitys: ResMut<ChunkEntitys>,
)
{
    let thread_pool = AsyncComputeTaskPool::get();
    for (i,b) in player.iter()
    {
        //i is camera
        //b is position
        //repeat through all sectors
        for sector in &universe.sectors
        {
            //find camera to sector offset
            let offset = b.translation - Vec3::new(sector.0[0] as f32, sector.0[1] as f32, sector.0[2] as f32);
            let chunk_pos = (offset / CHUNK_SIZE as f32).round();
            let chunk_center = [chunk_pos.x as i32, chunk_pos.y as i32, chunk_pos.z as i32];
            for x in chunk_center[0]-RENDER_DISTANCE..chunk_center[0]+RENDER_DISTANCE
            {
                for y in chunk_center[1]-RENDER_DISTANCE..chunk_center[1]+RENDER_DISTANCE
                {
                    for z in chunk_center[2]-RENDER_DISTANCE..chunk_center[2]+RENDER_DISTANCE
                    {
                        let chunk_id = [x,y,z];
                        //check if sector is loaded already
                        if let Some(mut loaded_sector) = loaded_sectors.sectors.get_mut(sector.0)
                        {
                            //sector already loaded
                            //check if chunk is loaded before rendering new chunk
                            if let Some(chunk) = loaded_sector.chunks.get(&chunk_id)
                            {
                                //chunk is loaded currently
                                //no need to reload chunk because its already loaded so do nothing
                            }else
                            {
                                //chunk is not loaded so load up chunk
                                loaded_sector.chunks.insert(chunk_id, Arc::new(None));
                                //build chunk data
                                let chunk_task:Task<Chunk> = thread_pool.spawn(async move
                                    {
                                        Chunk
                                        {
                                            block_data: generate_chunk(chunk_id),
                                            pos: (chunk_id[0]*CHUNK_SIZE, chunk_id[1]*CHUNK_SIZE, chunk_id[2]*CHUNK_SIZE),
                                        }
                                    });
                                //println!("Chunk {:?} in sector {:?} Loaded!", chunk_id, sector.0);
                                let mut name: String = chunk_id
                                    .iter()
                                    .map(|&n| n.to_string())  // Convert each integer to a String
                                    .collect::<Vec<_>>()       // Collect into a vector of strings
                                    .join(",");
                                name = format!("[{}]", name);
                                let generate_chunk = commands.spawn((GenerateChunk(chunk_task),Name::new(name))).id();
                                chunk_entitys.entitys.insert(chunk_id,generate_chunk);
                            }
                        }else
                        {
                            //load up sector
                            loaded_sectors.sectors.insert(*sector.0, SectorsRendering::default());
                            //chunk has not been loaded because sector was not loaded so a new chunk can be rendered safely
                            if let Some(loaded_sector) = loaded_sectors.sectors.get_mut(sector.0)
                            {
                                loaded_sector.chunks.insert(chunk_id, Arc::new(None));
                                let chunk_task:Task<Chunk> = thread_pool.spawn(async move
                                    {
                                        Chunk
                                        {
                                            block_data: generate_chunk(chunk_id),
                                            pos: (chunk_id[0]*CHUNK_SIZE, chunk_id[1]*CHUNK_SIZE, chunk_id[2]*CHUNK_SIZE),
                                        }
                                    });
                                //println!("Chunk {:?} in sector {:?} Loaded!", chunk_id, sector.0);
                                let generate_chunk = commands.spawn(GenerateChunk(chunk_task)).id();
                                chunk_entitys.entitys.insert(chunk_id,generate_chunk);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn unload_chunks
(
    mut commands: Commands,
    mut loaded_sectors: ResMut<CurrentlyLoaded>,
    mut chunk_entitys: ResMut<ChunkEntitys>,
    player: Query<(&Camera,&Transform), With<FlyCam>>,
)
{
    for (i, b) in &player
    {
        //i is camera
        //b is position
        //b.translation
        for sector in loaded_sectors.sectors.clone()
        {
            let offset = b.translation - Vec3::new(sector.0[0] as f32, sector.0[1] as f32, sector.0[2] as f32);
            let chunk_pos = (offset / CHUNK_SIZE as f32).round();
            let chunk_center = [chunk_pos.x as i32, chunk_pos.y as i32, chunk_pos.z as i32];
            for (position, chunk_entity) in chunk_entitys.entitys.clone()
            {
                let mut outside_range = true;
                for x in chunk_center[0]-RENDER_DISTANCE..chunk_center[0]+RENDER_DISTANCE
                {
                    for y in chunk_center[1]-RENDER_DISTANCE..chunk_center[1]+RENDER_DISTANCE
                    {
                        for z in chunk_center[2]-RENDER_DISTANCE..chunk_center[2]+RENDER_DISTANCE
                        {
                            if [x,y,z] == position
                            {

                                outside_range = false;
                            }
                        }
                    }
                }
                if outside_range
                {
                    loaded_sectors.sectors.get_mut(&sector.0).unwrap().chunks.remove(&position);
                    chunk_entitys.entitys.remove(&position);
                    commands.entity(chunk_entity).despawn();
                    //println!("Chunk {:?} in sector {:?} Unloaded!", position, sector.0);
                }
            }
        }
    }
}

pub fn generate_planet
(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlas_data: Res<TextureAtlasBuilt>,
    mut universe: ResMut<Universe>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
)
{
    let mut chunk_array:HashMap<[i32; 3], Chunk> = HashMap::new();
    let sector = Sector
    {
        chunks: chunk_array,
    };
    universe.sectors.insert([0,0,0],sector);


    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
}