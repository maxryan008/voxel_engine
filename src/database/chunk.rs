use std::any::Any;
use crate::database::texture::*;
use crate::database::structs::AppState;
use crate::database::voxel::*;
use crate::database::example_chunk::STONE_CHUNK;
use crate::database::settings::{CHUNK_SIZE, DENSITY_MOD, SEA_LEVEL};
use rand::prelude::*;
use std::sync::Arc;
use std::thread::current;
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
use splines::{Interpolation, Key, Spline};
use futures_lite::{future, StreamExt};
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::mesh;
use bevy::utils::HashMap;
use bevy::utils::label::DynEq;
use noise::{NoiseFn, Perlin, Seedable, Fbm, MultiFractal};
use crate::database::sector::*;



#[derive(Default, Debug, Clone)]
pub struct Chunk
{
    pub block_data: Vec<Voxel>,
    pub pos: (i32,i32,i32)
}

#[derive(Default, Debug, Clone, Resource)]
pub struct ChunkEntitys
{
    pub entitys: HashMap<[i32; 3], Entity>
}


#[derive(Component)]
pub struct GenerateChunk(pub Task<Chunk>);

#[derive(Default, Debug, Clone)]
pub struct RenderData
{
    vertices: Vec<[f32; 3]>,
    indices: Vec<u32>,
    uvs: Vec<[f32; 2]>,
    chunk_position: [i32; 3],
    chunk_data: Chunk,
    neighbours: HashMap<[i8; 3], Arc<Option<RenderData>>>,
}

#[derive(Component)]
pub struct ComputeChunk(Task<RenderData>);

#[derive(Component)]
pub struct UpdateChunk(Task<RenderData>);

#[derive(Component)]
pub struct SpawnChunk(Task<RenderData>);


pub fn chunk_to_render
(
    chunk: Chunk,
    loaded_chunks: CurrentlyLoaded,
) -> RenderData
{
    let mut render_data = RenderData::default();
    //check all 6 neighbouring chunks to see if there are any chunks currently loaded that can be added.
    //get sector
    if let Some(sector) = loaded_chunks.sectors.get(&[0,0,0])
    {
        //get chunk [x+1,y,z]
        let mut key = &[chunk.pos.0/CHUNK_SIZE+1,chunk.pos.1/CHUNK_SIZE,chunk.pos.2/CHUNK_SIZE];
        if !render_data.neighbours.contains_key(&[key.clone()[0] as i8,key.clone()[1] as i8,key.clone()[2] as i8])
        {
            if let Some(neighbour) = sector.chunks.get(&key.clone())
            {
                render_data.neighbours.insert([key[0] as i8,key[1] as i8,key[2] as i8], neighbour.clone());
            }
        }
        //get chunk [x-1,y,z]
        let mut key = &[chunk.pos.0/CHUNK_SIZE-1,chunk.pos.1/CHUNK_SIZE,chunk.pos.2/CHUNK_SIZE];
        if !render_data.neighbours.contains_key(&[key.clone()[0] as i8,key.clone()[1] as i8,key.clone()[2] as i8])
        {
            if let Some(neighbour) = sector.chunks.get(&key.clone())
            {
                render_data.neighbours.insert([key[0] as i8,key[1] as i8,key[2] as i8], neighbour.clone());
            }
        }
        //get chunk [x,y+1,z]
        let mut key = &[chunk.pos.0/CHUNK_SIZE,chunk.pos.1/CHUNK_SIZE+1,chunk.pos.2/CHUNK_SIZE];
        if !render_data.neighbours.contains_key(&[key.clone()[0] as i8,key.clone()[1] as i8,key.clone()[2] as i8])
        {
            if let Some(neighbour) = sector.chunks.get(&key.clone())
            {
                render_data.neighbours.insert([key[0] as i8,key[1] as i8,key[2] as i8], neighbour.clone());
            }
        }
        //get chunk [x,y-1,z]
        let mut key = &[chunk.pos.0/CHUNK_SIZE,chunk.pos.1/CHUNK_SIZE-1,chunk.pos.2/CHUNK_SIZE];
        if !render_data.neighbours.contains_key(&[key.clone()[0] as i8,key.clone()[1] as i8,key.clone()[2] as i8])
        {
            if let Some(neighbour) = sector.chunks.get(&key.clone())
            {
                render_data.neighbours.insert([key[0] as i8,key[1] as i8,key[2] as i8], neighbour.clone());
            }
        }
        //get chunk [x,y,z+1]
        let mut key = &[chunk.pos.0/CHUNK_SIZE,chunk.pos.1/CHUNK_SIZE,chunk.pos.2/CHUNK_SIZE+1];
        if !render_data.neighbours.contains_key(&[key.clone()[0] as i8,key.clone()[1] as i8,key.clone()[2] as i8])
        {
            if let Some(neighbour) = sector.chunks.get(&key.clone())
            {
                render_data.neighbours.insert([key[0] as i8,key[1] as i8,key[2] as i8], neighbour.clone());
            }
        }
        //get chunk [x,y,z-1]
        let mut key = &[chunk.pos.0/CHUNK_SIZE,chunk.pos.1/CHUNK_SIZE,chunk.pos.2/CHUNK_SIZE-1];
        if !render_data.neighbours.contains_key(&[key.clone()[0] as i8,key.clone()[1] as i8,key.clone()[2] as i8])
        {
            if let Some(neighbour) = sector.chunks.get(&key.clone())
            {
                render_data.neighbours.insert([key[0] as i8,key[1] as i8,key[2] as i8], neighbour.clone());
            }
        }
    }
    //render data stuff... moved to another thread for the purpose of locking threads until they have all neighbours available to check
    return render_data;
}

pub fn render_update
(
    mut render_data: RenderData,
    texture_info: TextureInfo,
    loaded_chunks: CurrentlyLoaded,
) -> RenderData
{
    let chunk_size_squared = CHUNK_SIZE * CHUNK_SIZE;

    for(index, voxel) in render_data.chunk_data.block_data.iter().enumerate()
    {
        let rotation = VOXEL_ROTATIONS[voxel.voxel_rotation as usize].clone();
        let z = index as f32 % CHUNK_SIZE as f32;
        let y = ((index as f32 - z) / CHUNK_SIZE as f32) % CHUNK_SIZE as f32;
        let x = (index as f32 - z - CHUNK_SIZE as f32 * y) / chunk_size_squared as f32;
        //sides
        //voxel 0 is air so we dont want to draw it
        if voxel.voxel_type != VoxelType::Air
        {
            //p for side iter
            let mut faces = FACES.clone();
            for i in 0..faces.len()
            {
                if rotation.switch
                {
                    faces[i] = [faces[i][2] * rotation.values[0], faces[i][1], faces[i][0] * rotation.values[1]];
                }else
                {
                    faces[i] = [faces[i][0] * rotation.values[0], faces[i][1], faces[i][2] * rotation.values[1]];
                }
            }
            for p in 0 .. 6
            {
                let neighbour_index = ((x + faces[p][0]) * chunk_size_squared as f32) + ((y + faces[p][1]) * CHUNK_SIZE as f32) + (z + faces[p][2]);
                //makes sure that neighbouring voxels are inside of the bounds of the chunk as to avoid errors
                if (x + faces[p][0]) >= 0. && (x + faces[p][0]) < CHUNK_SIZE as f32 && (y + faces[p][1]) >= 0. && (y + faces[p][1]) < CHUNK_SIZE as f32 && (z + faces[p][2]) >= 0. && (z + faces[p][2]) < CHUNK_SIZE as f32 {
                    //if the p side does not have a voxel then draw the face.
                    if render_data.chunk_data.block_data[neighbour_index as usize].solid == false {
                        if voxel.voxel_variant == VoxelVariant::Block
                        {
                            //i represents the corners of the triangle. 3 per triangle. 2 triangles per face
                            for i in 0 .. 6 {
                                if rotation.switch
                                {
                                    render_data.vertices.push([((BLOCK_VERTS[RECTANGLE_TRIS[p][i]][2]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(BLOCK_VERTS[RECTANGLE_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((BLOCK_VERTS[RECTANGLE_TRIS[p][i]][0]-0.5)*rotation.values[1] + z as f32) + 0.5 + render_data.chunk_data.pos.2 as f32]);
                                }else
                                {
                                    render_data.vertices.push([((BLOCK_VERTS[RECTANGLE_TRIS[p][i]][0]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(BLOCK_VERTS[RECTANGLE_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((BLOCK_VERTS[RECTANGLE_TRIS[p][i]][2]-0.5)*rotation.values[1] + z as f32) + 0.5 + render_data.chunk_data.pos.2 as f32]);
                                }
                                render_data.indices.push((render_data.vertices.len()-1) as u32);
                            }
                            let uv_rect = block_to_tex(voxel.voxel_type, texture_info.map.clone(), texture_info.rects.clone(),texture_info.size);
                            render_data.uvs.push([uv_rect.max.x,uv_rect.max.y]);
                            render_data.uvs.push([uv_rect.max.x,uv_rect.min.y]);
                            render_data.uvs.push([uv_rect.min.x,uv_rect.max.y]);
                            render_data.uvs.push([uv_rect.min.x,uv_rect.max.y]);
                            render_data.uvs.push([uv_rect.max.x,uv_rect.min.y]);
                            render_data.uvs.push([uv_rect.min.x,uv_rect.min.y]);
                        }
                        if voxel.voxel_variant == VoxelVariant::Slab
                        {
                            let uv_rect = block_to_tex(voxel.voxel_type, texture_info.map.clone(), texture_info.rects.clone(),texture_info.size);
                            for i in 0 .. 6 {
                                if rotation.switch
                                {
                                    render_data.vertices.push([((SLAB_VERTS[RECTANGLE_TRIS[p][i]][2]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(SLAB_VERTS[RECTANGLE_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((SLAB_VERTS[RECTANGLE_TRIS[p][i]][0]-0.5)*rotation.values[1] + z as f32) + 0.5 + render_data.chunk_data.pos.2 as f32]);
                                }else
                                {
                                    render_data.vertices.push([((SLAB_VERTS[RECTANGLE_TRIS[p][i]][0]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(SLAB_VERTS[RECTANGLE_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((SLAB_VERTS[RECTANGLE_TRIS[p][i]][2]-0.5)*rotation.values[1] + z as f32) + 0.5 + render_data.chunk_data.pos.2 as f32]);
                                }
                                render_data.indices.push((render_data.vertices.len()-1) as u32);
                                render_data.uvs.push([uv_rect.min.x+(uv_rect.max.x-uv_rect.min.x)*SLAB_UVS[p][i].x,uv_rect.min.y+(uv_rect.max.y-uv_rect.min.y)*SLAB_UVS[p][i].y]);
                            }
                        }
                        if voxel.voxel_variant == VoxelVariant::Stair
                        {
                            let uv_rect = block_to_tex(voxel.voxel_type, texture_info.map.clone(), texture_info.rects.clone(),texture_info.size);
                            for i in 0 .. STAIR_TRIS[p].len() {
                                if rotation.switch
                                {
                                    render_data.vertices.push([((STAIR_VERTS[STAIR_TRIS[p][i]][2]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(STAIR_VERTS[STAIR_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((STAIR_VERTS[STAIR_TRIS[p][i]][0]-0.5)*rotation.values[1] + 0.5 + z as f32) + render_data.chunk_data.pos.2 as f32]);
                                }else
                                {
                                    render_data.vertices.push([((STAIR_VERTS[STAIR_TRIS[p][i]][0]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(STAIR_VERTS[STAIR_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((STAIR_VERTS[STAIR_TRIS[p][i]][2]-0.5)*rotation.values[1] + 0.5 + z as f32) + render_data.chunk_data.pos.2 as f32]);
                                }
                                render_data.indices.push((render_data.vertices.len()-1) as u32);
                                render_data.uvs.push([uv_rect.min.x+(uv_rect.max.x-uv_rect.min.x)*STAIR_UVS[p][i].x,uv_rect.min.y+(uv_rect.max.y-uv_rect.min.y)*STAIR_UVS[p][i].y]);
                            }
                        }
                    }
                }else{
                    //it is along the border. depending on the face check direction check the chunk neighbour if exists for each face direction
                    let mut display_face = true;
                    //get sector
                    if let Some(sector) = loaded_chunks.sectors.get(&[0,0,0])
                    {
                        if (x + faces[p][0]) < 0.
                        {
                            //chunk [x-1,y,z]
                            //voxel [CHUNK_SIZE-1,y,z]
                            let key = &[render_data.chunk_data.pos.0/CHUNK_SIZE-1,render_data.chunk_data.pos.1/CHUNK_SIZE,render_data.chunk_data.pos.2/CHUNK_SIZE];
                            if let Some(neighbouring_chunk_arc) = sector.chunks.get(key)
                            {
                                if neighbouring_chunk_arc.is_some()
                                {
                                    if let Some(neighbouring_chunk) = &**neighbouring_chunk_arc
                                    {
                                        if neighbouring_chunk.chunk_data.block_data[(((CHUNK_SIZE-1)*CHUNK_SIZE*CHUNK_SIZE) as f32+y*CHUNK_SIZE as f32+z) as usize].solid == false
                                        {
                                            display_face = false;
                                        }
                                    }
                                }
                            }
                        }
                        if (x + faces[p][0]) >= CHUNK_SIZE as f32
                        {
                            //chunk [x+1,y,z]
                            //voxel [0,y,z]
                            let key = &[render_data.chunk_data.pos.0/CHUNK_SIZE+1,render_data.chunk_data.pos.1/CHUNK_SIZE,render_data.chunk_data.pos.2/CHUNK_SIZE];
                            if let Some(neighbouring_chunk_arc) = sector.chunks.get(key)
                            {
                                if neighbouring_chunk_arc.is_some()
                                {
                                    if let Some(neighbouring_chunk) = &**neighbouring_chunk_arc
                                    {
                                        if neighbouring_chunk.chunk_data.block_data[(y*CHUNK_SIZE as f32+z) as usize].solid == false
                                        {
                                            display_face = false;
                                        }
                                    }
                                }
                            }
                        }
                        if (y + faces[p][1]) < 0.
                        {
                            //chunk [x,y-1,z]
                            //voxel [x,CHUNK_SIZE-1,z]
                            let key = &[render_data.chunk_data.pos.0/CHUNK_SIZE,render_data.chunk_data.pos.1/CHUNK_SIZE-1,render_data.chunk_data.pos.2/CHUNK_SIZE];
                            if let Some(neighbouring_chunk_arc) = sector.chunks.get(key)
                            {
                                if neighbouring_chunk_arc.is_some()
                                {
                                    if let Some(neighbouring_chunk) = &**neighbouring_chunk_arc
                                    {
                                        if neighbouring_chunk.chunk_data.block_data[(x*chunk_size_squared as f32+((CHUNK_SIZE-1)*CHUNK_SIZE) as f32+z) as usize].solid == false
                                        {
                                            display_face = false;
                                        }
                                    }
                                }
                            }
                        }
                        if (y + faces[p][1]) >= CHUNK_SIZE as f32
                        {
                            //chunk [x,y+1,z]
                            //voxel [x,0,z]
                            let key = &[render_data.chunk_data.pos.0/CHUNK_SIZE,render_data.chunk_data.pos.1/CHUNK_SIZE+1,render_data.chunk_data.pos.2/CHUNK_SIZE];
                            if let Some(neighbouring_chunk_arc) = sector.chunks.get(key)
                            {
                                if neighbouring_chunk_arc.is_some()
                                {
                                    if let Some(neighbouring_chunk) = &**neighbouring_chunk_arc
                                    {
                                        if neighbouring_chunk.chunk_data.block_data[(x*chunk_size_squared as f32+z) as usize].solid == false
                                        {
                                            display_face = false;
                                        }
                                    }
                                }
                            }
                        }
                        if (z + faces[p][2]) < 0.
                        {
                            //chunk [x,y,z-1]
                            //voxel [x,y,CHUNK_SIZE-1]
                            let key = &[render_data.chunk_data.pos.0/CHUNK_SIZE,render_data.chunk_data.pos.1/CHUNK_SIZE,render_data.chunk_data.pos.2/CHUNK_SIZE-1];
                            if let Some(neighbouring_chunk_arc) = sector.chunks.get(key)
                            {
                                if neighbouring_chunk_arc.is_some()
                                {
                                    if let Some(neighbouring_chunk) = &**neighbouring_chunk_arc
                                    {
                                        if neighbouring_chunk.chunk_data.block_data[(x*chunk_size_squared as f32+y*CHUNK_SIZE as f32+(CHUNK_SIZE-1) as f32) as usize].solid == false
                                        {
                                            display_face = false;
                                        }
                                    }
                                }
                            }
                        }
                        if (z + faces[p][2]) >= CHUNK_SIZE as f32
                        {
                            //chunk [x,y,z+1]
                            //voxel [x,y,0]
                            let key = &[render_data.chunk_data.pos.0/CHUNK_SIZE,render_data.chunk_data.pos.1/CHUNK_SIZE,render_data.chunk_data.pos.2/CHUNK_SIZE+1];
                            if let Some(neighbouring_chunk_arc) = sector.chunks.get(key)
                            {
                                if neighbouring_chunk_arc.is_some()
                                {
                                    if let Some(neighbouring_chunk) = &**neighbouring_chunk_arc
                                    {
                                        if neighbouring_chunk.chunk_data.block_data[(x*chunk_size_squared as f32+y*CHUNK_SIZE as f32) as usize].solid == false
                                        {
                                            display_face = false;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !display_face
                    {
                        if voxel.voxel_variant == VoxelVariant::Block
                        {
                            for i in 0 .. 6 {
                                if rotation.switch
                                {
                                    render_data.vertices.push([((BLOCK_VERTS[RECTANGLE_TRIS[p][i]][2]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(BLOCK_VERTS[RECTANGLE_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((BLOCK_VERTS[RECTANGLE_TRIS[p][i]][0]-0.5)*rotation.values[1] + z as f32) + 0.5 + render_data.chunk_data.pos.2 as f32]);
                                }else
                                {
                                    render_data.vertices.push([((BLOCK_VERTS[RECTANGLE_TRIS[p][i]][0]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(BLOCK_VERTS[RECTANGLE_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((BLOCK_VERTS[RECTANGLE_TRIS[p][i]][2]-0.5)*rotation.values[1] + z as f32) + 0.5 + render_data.chunk_data.pos.2 as f32]);
                                }
                                render_data.indices.push((render_data.vertices.len()-1) as u32);
                            }
                            let uv_rect = block_to_tex(voxel.voxel_type, texture_info.map.clone(), texture_info.rects.clone(),texture_info.size);
                            render_data.uvs.push([uv_rect.max.x,uv_rect.max.y]);
                            render_data.uvs.push([uv_rect.max.x,uv_rect.min.y]);
                            render_data.uvs.push([uv_rect.min.x,uv_rect.max.y]);
                            render_data.uvs.push([uv_rect.min.x,uv_rect.max.y]);
                            render_data.uvs.push([uv_rect.max.x,uv_rect.min.y]);
                            render_data.uvs.push([uv_rect.min.x,uv_rect.min.y]);
                        }
                        if voxel.voxel_variant == VoxelVariant::Slab
                        {
                            let uv_rect = block_to_tex(voxel.voxel_type, texture_info.map.clone(), texture_info.rects.clone(),texture_info.size);
                            for i in 0 .. 6 {
                                if rotation.switch
                                {
                                    render_data.vertices.push([((SLAB_VERTS[RECTANGLE_TRIS[p][i]][2]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(SLAB_VERTS[RECTANGLE_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((SLAB_VERTS[RECTANGLE_TRIS[p][i]][0]-0.5)*rotation.values[1] + z as f32) + 0.5 + render_data.chunk_data.pos.2 as f32]);
                                }else
                                {
                                    render_data.vertices.push([((SLAB_VERTS[RECTANGLE_TRIS[p][i]][0]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(SLAB_VERTS[RECTANGLE_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((SLAB_VERTS[RECTANGLE_TRIS[p][i]][2]-0.5)*rotation.values[1] + z as f32) + 0.5 + render_data.chunk_data.pos.2 as f32]);
                                }
                                render_data.indices.push((render_data.vertices.len()-1) as u32);
                                render_data.uvs.push([uv_rect.min.x+(uv_rect.max.x-uv_rect.min.x)*SLAB_UVS[p][i].x,uv_rect.min.y+(uv_rect.max.y-uv_rect.min.y)*SLAB_UVS[p][i].y]);
                            }
                        }
                        if voxel.voxel_variant == VoxelVariant::Stair
                        {
                            let uv_rect = block_to_tex(voxel.voxel_type, texture_info.map.clone(), texture_info.rects.clone(),texture_info.size);
                            for i in 0 .. STAIR_TRIS[p].len() {
                                if rotation.switch
                                {
                                    render_data.vertices.push([((STAIR_VERTS[STAIR_TRIS[p][i]][2]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(STAIR_VERTS[STAIR_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((STAIR_VERTS[STAIR_TRIS[p][i]][0]-0.5)*rotation.values[1] + 0.5 + z as f32) + render_data.chunk_data.pos.2 as f32]);
                                }else
                                {
                                    render_data.vertices.push([((STAIR_VERTS[STAIR_TRIS[p][i]][0]-0.5)*rotation.values[0] + 0.5 + x as f32) + render_data.chunk_data.pos.0 as f32,(STAIR_VERTS[STAIR_TRIS[p][i]][1] + y as f32) + render_data.chunk_data.pos.1 as f32,((STAIR_VERTS[STAIR_TRIS[p][i]][2]-0.5)*rotation.values[1] + 0.5 + z as f32) + render_data.chunk_data.pos.2 as f32]);
                                }
                                render_data.indices.push((render_data.vertices.len()-1) as u32);
                                render_data.uvs.push([uv_rect.min.x+(uv_rect.max.x-uv_rect.min.x)*STAIR_UVS[p][i].x,uv_rect.min.y+(uv_rect.max.y-uv_rect.min.y)*STAIR_UVS[p][i].y]);
                            }
                        }
                    }
                }
            }
        }
    }
    return render_data;
}

pub fn generate_chunk(chunk_position: [i32; 3]) -> Vec<Voxel>
{
    let sp1 = Key::new(-1.0, 50.0, Interpolation::Linear);
    let sp2 = Key::new(0.3, 100.0, Interpolation::default());
    let sp3 = Key::new(0.4, 150.0, Interpolation::default());
    let sp4 = Key::new(0.55, 154.0, Interpolation::default());
    let sp5 = Key::new(0.8, 158.0, Interpolation::default());
    let sp6 = Key::new(1.0, 153.0, Interpolation::default());
    let spline = Spline::from_vec(vec![sp1, sp2, sp3, sp4, sp5]);

    let fbm = Fbm::<Perlin>::default().set_seed(1).set_octaves(4);
    let mut data = Vec::with_capacity(CHUNK_SIZE.pow(3) as usize);

    let x_offset = chunk_position[0] * CHUNK_SIZE;
    let y_offset = chunk_position[1] * CHUNK_SIZE;
    let z_offset = chunk_position[2] * CHUNK_SIZE;

    let mut vals = Vec::with_capacity(CHUNK_SIZE as usize);
    let mut rng = rand::thread_rng();
    for x in 0..CHUNK_SIZE {
        vals.clear();
        for z in 0..CHUNK_SIZE {
            vals.push(fbm.get([(x + x_offset) as f64 * 0.002, (z + z_offset) as f64 * 0.002]));
        }
        for y in 0..CHUNK_SIZE {
            let y_val = y + y_offset;
            for z in 0..CHUNK_SIZE {
                let mut voxel = Voxel::default();
                let height:f64 = (spline.clamped_sample(vals[z as usize]).unwrap()*10.0).round()/10.0;
                if (y_val as f64 - height).abs() < 0.2 {
                    let x1 = spline.clamped_sample(fbm.get([(x + x_offset + 1) as f64 * 0.002, (z + z_offset) as f64 * 0.002])).unwrap().round();
                    let x2 = spline.clamped_sample(fbm.get([(x + x_offset - 1) as f64 * 0.002, (z + z_offset) as f64 * 0.002])).unwrap().round();
                    let z1 = spline.clamped_sample(fbm.get([(x + x_offset) as f64 * 0.002, (z + z_offset + 1) as f64 * 0.002])).unwrap().round();
                    let z2 = spline.clamped_sample(fbm.get([(x + x_offset) as f64 * 0.002, (z + z_offset - 1) as f64 * 0.002])).unwrap().round();
                    let max = x1.max(x2).max(z1).max(z2);
                    let x1max:bool = max == x1;
                    let x2max:bool = max == x2;
                    let z1max:bool = max == z1;
                    let z2max:bool = max == z2;
                    if z1max
                    {
                        voxel.voxel_variant = VoxelVariant::Stair;
                        voxel.voxel_rotation = VoxelRotation::Forward;
                        voxel.voxel_type = VoxelType::Grass;
                        voxel.solid = false;
                    }else if z2max
                    {
                        voxel.voxel_variant = VoxelVariant::Stair;
                        voxel.voxel_rotation = VoxelRotation::Backward;
                        voxel.voxel_type = VoxelType::Grass;
                        voxel.solid = false;
                    }else if x1max
                    {
                        voxel.voxel_variant = VoxelVariant::Stair;
                        voxel.voxel_rotation = VoxelRotation::Left;
                        voxel.voxel_type = VoxelType::Grass;
                        voxel.solid = false;
                    }else if x2max
                    {
                        voxel.voxel_variant = VoxelVariant::Stair;
                        voxel.voxel_rotation = VoxelRotation::Right;
                        voxel.voxel_type = VoxelType::Grass;
                        voxel.solid = false;
                    }
                    if (x1max&&x2max)||(x1max&&z1max)||(x1max&&z2max)||(x2max&&z1max)||(x2max&&z2max)||(z1max&&z2max)
                    {
                        voxel.voxel_variant = VoxelVariant::Slab;
                        voxel.voxel_type = VoxelType::Grass;
                        voxel.solid = false;
                    }
                    if (max - height).abs() < 0.2  {
                        voxel.voxel_variant = VoxelVariant::Slab;
                        voxel.voxel_type = VoxelType::Grass;
                        voxel.solid = false;
                    }
                }else if (y_val as f64) < height
                {
                    voxel.voxel_type = VoxelType::Dirt;
                    voxel.solid = true;
                    if (y_val as f64) < height - rng.gen_range(2..=4) as f64
                    {
                        voxel.voxel_type = VoxelType::Stone;
                    }
                    if (y_val as f64) < height && (y_val as f64) > height-1.0
                        {
                        voxel.voxel_type = VoxelType::Grass;
                    }
                }
                if y_val <= SEA_LEVEL
                {
                    voxel.voxel_variant = VoxelVariant::Block;
                    if voxel.solid == false
                    {
                        voxel.solid = true;
                        voxel.voxel_type = VoxelType::Water;
                    }else
                    {
                        voxel.voxel_type = VoxelType::Sand;
                    }

                }
                if voxel.voxel_type == VoxelType::Air
                {
                    if rng.gen_range(0..1000) < 1
                    {
                        voxel.voxel_type = VoxelType::Glass;
                        voxel.solid = false;
                        voxel.voxel_variant = VoxelVariant::Block;
                    }
                }
                data.push(voxel);
            }
        }
    }

    data
}


pub fn chunk_handler
(
    mut commands: Commands,
    mut compute_chunks: Query<(Entity, &mut ComputeChunk)>,
    mut update_chunks: Query<(Entity, &mut UpdateChunk)>,
    mut spawn_chunks: Query<(Entity, &mut SpawnChunk)>,
    mut generate_chunks: Query<(Entity, &mut GenerateChunk)>,
    mut loaded_sectors: ResMut<CurrentlyLoaded>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    texture_atlas_data: Res<TextureAtlasBuilt>,
    mut chunk_entitys: ResMut<ChunkEntitys>,
    query: Query<Entity>
)
{
    println!("Total number of entities: {}", query.iter().count());
    let thread_pool = AsyncComputeTaskPool::get();

    //generate chunks
    //accepts chunk data and turns into render data via thread
    for (mut entity, mut task) in &mut generate_chunks
    {
        if let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) {
            let texture_atlas_data = texture_atlas_data.clone();
            let loaded_sectors_copy = loaded_sectors.clone();
            let position = [chunk_data.pos.0.clone()/CHUNK_SIZE,chunk_data.pos.1.clone()/CHUNK_SIZE,chunk_data.pos.2.clone()/CHUNK_SIZE];
            let chunk_task:Task<RenderData> =  thread_pool.spawn(async move
                {
                    let texture_info = TextureInfo
                    {
                        map: texture_atlas_data.texture_map.to_vec(),
                        rects: texture_atlas_data.texture_rects.to_vec(),
                        size: texture_atlas_data.texture_size,
                    };
                    let mut render_data = RenderData::default();
                    render_data = chunk_to_render(chunk_data.clone(), loaded_sectors_copy);
                    RenderData
                    {
                        vertices: render_data.vertices,
                        indices: render_data.indices,
                        uvs: render_data.uvs,
                        chunk_position: position.clone(),
                        chunk_data,
                        neighbours: render_data.neighbours,
                    }
                });
            let mut name: String = position
                .iter()
                .map(|&n| n.to_string())  // Convert each integer to a String
                .collect::<Vec<_>>()       // Collect into a vector of strings
                .join(",");
            name = format!("[{}]", name);
            commands.entity(entity).remove::<GenerateChunk>();
            commands.entity(entity).remove::<Name>();
            commands.entity(entity).insert((ComputeChunk(chunk_task),Name::new(name))).id();
        }
    }



    //compute chunks
    //accepts render data and re-renders all neighbour chunks
    for (mut entity, mut task) in &mut compute_chunks
    {
        if let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) {
            //update all neighbour chunks when chunk loads
            //set loaded_sector stuff
            if let Some(mut loaded_sector) = loaded_sectors.sectors.get_mut(&[0,0,0])
            {
                if let Some(mut chunk) = loaded_sector.chunks.get_mut(&chunk_data.chunk_position)
                {
                    *chunk = Arc::new(Some(chunk_data.clone()));
                }
            }
            //get sector [0,0,0]
            if let Some(mut loaded_sector) = loaded_sectors.sectors.get(&[0,0,0])
            {
                //get all neighbouring chunks
                for (position,chunk) in &chunk_data.neighbours
                {
                    //get the render data of the neighbouring chunk in form of arc
                    if let Some(mut neighbouring_chunk_arc) = loaded_sector.chunks.get(&[position[0] as i32,position[1] as i32,position[2] as i32])
                    {
                        //get the render data out of the arc
                        if let Some(neighbouring_chunk) = &**neighbouring_chunk_arc
                        {
                            let texture_atlas_data_copy = texture_atlas_data.clone();
                            let loaded_sectors_copy = loaded_sectors.clone();
                            let neighbouring_chunk_copy = neighbouring_chunk.clone();
                            let position = [neighbouring_chunk_copy.chunk_data.pos.0.clone()/CHUNK_SIZE,neighbouring_chunk_copy.chunk_data.pos.1.clone()/CHUNK_SIZE,neighbouring_chunk_copy.chunk_data.pos.2.clone()/CHUNK_SIZE];
                            let chunk_task:Task<RenderData> =  thread_pool.spawn(async move
                                {
                                    let texture_info = TextureInfo
                                    {
                                        map: texture_atlas_data_copy.texture_map.to_vec(),
                                        rects: texture_atlas_data_copy.texture_rects.to_vec(),
                                        size: texture_atlas_data_copy.texture_size,
                                    };
                                    let mut render_data = RenderData::default();
                                    render_data = render_update(neighbouring_chunk_copy.clone(), texture_info,loaded_sectors_copy);
                                    RenderData
                                    {
                                        vertices: render_data.vertices,
                                        indices: render_data.indices,
                                        uvs: render_data.uvs,
                                        chunk_position: neighbouring_chunk_copy.chunk_position,
                                        chunk_data: neighbouring_chunk_copy.chunk_data,
                                        neighbours: render_data.neighbours,
                                    }

                                });
                            let mut name: String = position
                                .iter()
                                .map(|&n| n.to_string())  // Convert each integer to a String
                                .collect::<Vec<_>>()       // Collect into a vector of strings
                                .join(",");
                            name = format!("[{}]", name);
                            if let Some(entity) = chunk_entitys.entitys.get(&position)
                            {
                                commands.entity(*entity).insert((UpdateChunk(chunk_task)));
                            }
                        }
                    }
                }
            }

            let position = [chunk_data.chunk_data.pos.0.clone()/CHUNK_SIZE,chunk_data.chunk_data.pos.1.clone()/CHUNK_SIZE,chunk_data.chunk_data.pos.2.clone()/CHUNK_SIZE];
            let texture_atlas_data_copy = texture_atlas_data.clone();
            let loaded_sectors_copy = loaded_sectors.clone();
            let chunk_task:Task<RenderData> =  thread_pool.spawn(async move
                {
                    let texture_info = TextureInfo
                    {
                        map: texture_atlas_data_copy.texture_map.to_vec(),
                        rects: texture_atlas_data_copy.texture_rects.to_vec(),
                        size: texture_atlas_data_copy.texture_size,
                    };
                    let mut render_data = RenderData::default();
                    render_data = render_update(chunk_data.clone(), texture_info,loaded_sectors_copy);
                    RenderData
                    {
                        vertices: render_data.vertices,
                        indices: render_data.indices,
                        uvs: render_data.uvs,
                        chunk_position: position.clone(),
                        chunk_data: chunk_data.chunk_data,
                        neighbours: render_data.neighbours,
                    }
                });

            let mut name: String = position
                .iter()
                .map(|&n| n.to_string())  // Convert each integer to a String
                .collect::<Vec<_>>()       // Collect into a vector of strings
                .join(",");
            name = format!("[{}]", name);
            commands.entity(entity).remove::<ComputeChunk>();
            commands.entity(entity).remove::<Name>();
            commands.entity(entity).insert((SpawnChunk(chunk_task),Name::new(name))).id();
        }
    }

    //spawn chunks
    //creates the mesh from render data
    for (mut entity, mut task) in &mut spawn_chunks
    {
        if let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) {
            let mut chunk_new_mesh = Mesh::new(PrimitiveTopology::TriangleList);
            chunk_new_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, chunk_data.vertices.clone());
            chunk_new_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0., 1.]; chunk_data.indices.len()]);
            chunk_new_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, chunk_data.uvs.clone());
            chunk_new_mesh.set_indices(Some(mesh::Indices::U32(chunk_data.indices.clone())));
            commands.entity(entity).insert((PbrBundle {
                mesh: meshes.add(chunk_new_mesh),
                material: materials.add(StandardMaterial{
                    emissive: Color::WHITE,
                    emissive_texture: Option::from(texture_atlas_data.clone().tex),
                    double_sided: false,
                    alpha_mode: AlphaMode::Opaque,

                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            }));
            commands.entity(entity).remove::<SpawnChunk>();
        }
    }






    for (mut entity, mut task) in &mut update_chunks.iter_mut()
    {
        if let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) {
            let mut chunk_new_mesh = Mesh::new(PrimitiveTopology::TriangleList);
            chunk_new_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, chunk_data.vertices.clone());
            chunk_new_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0., 1.]; chunk_data.indices.len()]);
            chunk_new_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, chunk_data.uvs.clone());
            chunk_new_mesh.set_indices(Some(mesh::Indices::U32(chunk_data.indices.clone())));
            commands.entity(entity).remove::<Handle<Mesh>>();
            commands.entity(entity).insert(meshes.add(chunk_new_mesh));
            commands.entity(entity).remove::<UpdateChunk>();
        }
    }

}

