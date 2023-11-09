pub const FACES: [[f32; 3]; 6] = [[0.0,0.0,-1.0],[0.0,0.0,1.0],[0.0,1.0,0.0],[0.0,-1.0,0.0],[-1.0,0.0,0.0],[1.0,0.0,0.0]];
pub const BLOCK_VERTS: [[f32; 3]; 8] = [[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0],[1.0,0.0,1.0],[0.0,1.0,1.0],[1.0,1.0,1.0]];
pub const SLAB_VERTS: [[f32; 3]; 8] = [[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,0.5,0.0],[0.0,0.5,0.0],[0.0,0.0,1.0],[1.0,0.0,1.0],[0.0,0.5,1.0],[1.0,0.5,1.0]];
pub const STAIR_VERTS: [[f32; 3]; 12] = [[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,0.5,0.0],[1.0,0.5,0.0],[0.0,0.5,0.5],[1.0,0.5,0.5],[0.0,1.0,0.5],[1.0,1.0,0.5],[0.0,0.0,1.0],[1.0,0.0,1.0],[0.0,1.0,1.0],[1.0,1.0,1.0]];
pub const RECTANGLE_TRIS: [[usize; 6]; 6] = [[0,3,1,1,3,2],[5,7,4,4,7,6],[3,6,2,2,6,7],[1,5,0,0,5,4],[4,6,0,0,6,3],[1,2,5,5,2,7]];
pub const STAIR_TRIS: [&'static [usize]; 6] = [/*front face*/&[/*bottom*/0,2,1,1,2,3,/*top*/4,6,5,5,6,7],/*back face*/&[9,11,8,8,11,10],/*top face*/&[/*top*/6,10,7,7,10,11,/*bottom*/2,4,3,3,4,5],/*bottom face*/&[8,0,9,9,0,1],/*left face*/&[8,10,0,0,4,2,4,10,6],/*right face*/&[1,11,9,1,3,5,5,7,11]];
pub const STAIR_UVS: [&'static [Vec2]; 6] = [/*front face*/&[/*bottom*/vec2(1.0,1.0),vec2(1.0,0.5),vec2(0.0,1.0),vec2(0.0,1.0),vec2(1.0,0.5),vec2(0.0,0.5),/*top*/vec2(1.0,0.5),vec2(1.0,0.0),vec2(0.0,0.5),vec2(0.0,0.5),vec2(1.0,0.0),vec2(0.0,0.0)],/*back face*/&[vec2(0.0,1.0),vec2(0.0,0.0),vec2(1.0,1.0),vec2(1.0,1.0),vec2(0.0,0.0),vec2(1.0,0.0)],/*top face*/&[/*top*/vec2(1.0,0.5),vec2(1.0,0.0),vec2(0.0,0.5),vec2(0.0,0.5),vec2(1.0,0.0),vec2(0.0,0.0),/*bottom*/vec2(1.0,1.0),vec2(1.0,0.5),vec2(0.0,1.0),vec2(0.0,1.0),vec2(1.0,0.5),vec2(0.0,0.5)],/*bottom face*/&[vec2(1.0,0.0),vec2(1.0,1.0),vec2(0.0,0.0),vec2(0.0,0.0),vec2(1.0,1.0),vec2(0.0,1.0)],/*left face*/&[vec2(0.0,1.0),vec2(0.0,0.0),vec2(1.0,1.0),vec2(1.0,1.0),vec2(0.5,0.5),vec2(1.0,0.5),vec2(0.5,0.5),vec2(0.0,0.0),vec2(0.5,0.0)],/*right face*/&[vec2(1.0,1.0),vec2(0.0,0.0),vec2(0.0,1.0),vec2(1.0,1.0),vec2(1.0,0.5),vec2(0.5,0.5),vec2(0.5,0.5),vec2(0.5,0.0),vec2(0.0,0.0)]];
pub const SLAB_UVS: [[Vec2; 6]; 6] = [/*front face*/[vec2(1.0,1.0),vec2(1.0,0.5),vec2(0.0,1.0),vec2(0.0,1.0),vec2(1.0,0.5),vec2(0.0,0.5)],/*back face*/[vec2(0.0,1.0),vec2(0.0,0.5),vec2(1.0,1.0),vec2(1.0,1.0),vec2(0.0,0.5),vec2(1.0,0.5)],/*top face*/[vec2(1.0,1.0),vec2(1.0,0.0),vec2(0.0,1.0),vec2(0.0,1.0),vec2(1.0,0.0),vec2(0.0,0.0)],/*bottom face*/[vec2(0.0,1.0),vec2(0.0,0.0),vec2(1.0,1.0),vec2(1.0,1.0),vec2(0.0,0.0),vec2(1.0,0.0)],/*left face*/[vec2(0.0,1.0),vec2(0.0,0.5),vec2(1.0,1.0),vec2(1.0,1.0),vec2(0.0,0.5),vec2(1.0,0.5)],/*right face*/[vec2(1.0,1.0),vec2(1.0,0.5),vec2(0.0,1.0),vec2(0.0,1.0),vec2(1.0,0.5),vec2(0.0,0.5)]];
pub const VOXEL_ROTATIONS: [Rotation; 4] = [/*Forward*/Rotation{switch:false,values:[1.0,1.0]},/*Backward*/Rotation{switch:false,values:[-1.0,-1.0]},/*Left*/Rotation{switch:true,values:[1.0,-1.0]},/*Right*/Rotation{switch:true,values:[-1.0,1.0]}];
use bevy::math::vec2;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable, Fbm, MultiFractal};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum VoxelType {
    #[default]
    Air,
    Stone,
    Dirt,
    Sand,
    Brick,
    Grass,
    Lava,
    Water,
    Salt,
    Ash,
    RedSand,
    Coral,
    Sulfur,
    JungleGrass,
    SavannahGrass,
    SwampGrass,
    Ice,
    SnowBlock,
    Snow,
    Pine,
    Forest,
    Glass,
    Sexy,
    Rainbow,
    StoneBrick,
    Arrow,
    Netherack,
    Arsenic,
    Actinium,
    Antimony,
    Aluminum,
    Copper,
    Cat,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum VoxelVariant {
    #[default]
    Block,
    Slab,
    Stair,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum VoxelRotation {
    #[default]
    Forward,
    Backward,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Rotation {
    pub switch: bool,
    pub values: [f32; 2]
}

#[derive(Default, Debug, Clone)]
pub struct Voxel
{
    pub voxel_type: VoxelType,
    pub solid: bool,
    pub voxel_variant: VoxelVariant,
    pub voxel_rotation: VoxelRotation,
}

pub fn block_to_tex
(
    block_type: VoxelType,
    tex_map: Vec<usize>,
    tex_rects: Vec<Rect>,
    tex_size: Vec2,
) -> Rect
{
    return Rect::new(tex_rects[tex_map[block_type as usize]].min.x/tex_size.x,tex_rects[tex_map[block_type as usize]].min.y/tex_size.y,tex_rects[tex_map[block_type as usize]].max.x/tex_size.x,tex_rects[tex_map[block_type as usize]].max.y/tex_size.y);
}
