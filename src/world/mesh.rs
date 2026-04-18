//! This module implements a builder pattern for creating a mesh that can be
//! used to render terrain with a tileset.

use std::ops::Mul;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::tileset::material::ATTRIBUTE_UV_LAYER;

/// A temporary buffer for storing mesh data capable of rendering terrain.
#[derive(Debug, Default, Clone)]
pub struct TerrainMesh {
    /// The vertex positions of the mesh.
    positions: Vec<[f32; 3]>,

    /// The vertex texture coordinates of the mesh.
    uvs: Vec<[f32; 2]>,

    /// The texture array layers of the mesh.
    layers: Vec<u32>,

    /// The vertex normals of the mesh.
    normals: Vec<[f32; 3]>,

    /// The vertex colors of the mesh.
    colors: Vec<[f32; 4]>,

    /// The indices of the mesh.
    indices: Vec<u32>,
}

impl TerrainMesh {
    /// The initial capacity of the vertices.
    const INIT_CAPACITY_VERTS: usize = 1024;

    /// The initial capacity of the indices.
    const INIT_CAPACITY_INDICES: usize = 2048;

    /// Creates a new terrain mesh.
    pub fn new() -> Self {
        Self {
            positions: Vec::with_capacity(Self::INIT_CAPACITY_VERTS),
            uvs: Vec::with_capacity(Self::INIT_CAPACITY_VERTS),
            layers: Vec::with_capacity(Self::INIT_CAPACITY_VERTS),
            normals: Vec::with_capacity(Self::INIT_CAPACITY_VERTS),
            colors: Vec::with_capacity(Self::INIT_CAPACITY_VERTS),
            indices: Vec::with_capacity(Self::INIT_CAPACITY_INDICES),
        }
    }

    /// Gets a reference to the vertex positions of the mesh.
    pub fn positions(&self) -> &[[f32; 3]] {
        &self.positions
    }

    /// Gets a reference to the indices of the mesh.
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Gets a reference to the vertex texture coordinates of the mesh.
    pub fn tex_coords(&self) -> &[[f32; 2]] {
        &self.uvs
    }

    /// Gets a reference to the texture array layers of the mesh.
    pub fn layers(&self) -> &[u32] {
        &self.layers
    }

    /// Gets a reference to the vertex normals of the mesh.
    pub fn normals(&self) -> &[[f32; 3]] {
        &self.normals
    }

    /// Gets a reference to the vertex colors of the mesh.
    pub fn colors(&self) -> &[[f32; 4]] {
        &self.colors
    }

    /// Gets the number of triangles in the mesh.
    pub fn tri_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Appends the mesh data from another mesh to this mesh.
    pub fn append(&mut self, other: &Self, transform: Transform) {
        let offset = self.positions.len() as u32;
        let matrix = transform.to_matrix();

        self.positions.reserve(other.positions.len());
        for position in &other.positions {
            let position = matrix * Vec4::new(position[0], position[1], position[2], 1.0);
            self.positions.push([position.x, position.y, position.z]);
        }

        self.normals.reserve(other.normals.len());
        for normal in &other.normals {
            let normal = matrix * Vec4::new(normal[0], normal[1], normal[2], 0.0);
            self.normals.push([normal.x, normal.y, normal.z]);
        }

        self.uvs.extend_from_slice(&other.uvs);
        self.colors.extend_from_slice(&other.colors);

        self.indices
            .extend(other.indices.iter().map(|i| i + offset));
    }

    /// Appends a triangle to the mesh.
    pub fn add_triangle(&mut self, triangle: TerrainTriangle) {
        let offset = self.positions.len() as u32;

        for vert in triangle.vertices() {
            let pos = [vert.position.x, vert.position.y, vert.position.z];
            let uv = [vert.uv.x, vert.uv.y];
            let normal = [vert.normal.x, vert.normal.y, vert.normal.z];

            let color = vert.color.to_srgba();
            let color = [color.red, color.green, color.blue, color.alpha];

            self.positions.push(pos);
            self.uvs.push(uv);
            self.layers.push(vert.layer);
            self.normals.push(normal);
            self.colors.push(color);
        }

        self.indices.push(offset);
        self.indices.push(offset + 1);
        self.indices.push(offset + 2);
    }

    /// Appends a quad to the mesh.
    pub fn add_quad(&mut self, quad: TerrainQuad) {
        let offset = self.positions.len() as u32;

        for vert in quad.vertices() {
            let pos = [vert.position.x, vert.position.y, vert.position.z];
            let uv = [vert.uv.x, vert.uv.y];
            let normal = [vert.normal.x, vert.normal.y, vert.normal.z];

            let color = vert.color.to_srgba();
            let color = [color.red, color.green, color.blue, color.alpha];

            self.positions.push(pos);
            self.uvs.push(uv);
            self.layers.push(vert.layer);
            self.normals.push(normal);
            self.colors.push(color);
        }

        self.indices.push(offset);
        self.indices.push(offset + 1);
        self.indices.push(offset + 2);

        self.indices.push(offset);
        self.indices.push(offset + 2);
        self.indices.push(offset + 3);
    }
}

impl From<TerrainMesh> for Mesh {
    fn from(value: TerrainMesh) -> Self {
        let indices = if value.indices.len() > u16::MAX as usize {
            Indices::U32(value.indices)
        } else {
            Indices::U16(value.indices.iter().map(|&i| i as u16).collect())
        };

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, value.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, value.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, value.uvs)
        .with_inserted_attribute(ATTRIBUTE_UV_LAYER, value.layers)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, value.colors)
        .with_inserted_indices(indices)
    }
}

/// A vertex that stores the position, normal, texture coordinates, layer, and
/// color of a terrain vertex.
#[derive(Debug, Clone, Copy)]
pub struct TerrainVertex {
    /// The position of the vertex.
    pub position: Vec3,

    /// The normal of the vertex.
    pub normal: Vec3,

    /// The texture coordinates of the vertex.
    pub uv: Vec2,

    /// The texture array layer of the vertex.
    pub layer: u32,

    /// The color of the vertex.
    pub color: Color,
}

impl Mul<TerrainVertex> for Mat4 {
    type Output = TerrainVertex;

    fn mul(self, rhs: TerrainVertex) -> Self::Output {
        let pos4 = self * Vec4::new(rhs.position.x, rhs.position.y, rhs.position.z, 1.0);
        let norm4 = self * Vec4::new(rhs.normal.x, rhs.normal.y, rhs.normal.z, 0.0);

        TerrainVertex {
            position: pos4.xyz(),
            normal: norm4.xyz(),
            uv: rhs.uv,
            layer: rhs.layer,
            color: rhs.color,
        }
    }
}

impl Mul<Transform> for TerrainVertex {
    type Output = TerrainVertex;

    fn mul(self, rhs: Transform) -> Self::Output {
        rhs.to_matrix() * self
    }
}

/// A triangle that stores the vertices for a [`TerrainMesh`].
#[derive(Debug, Clone, Copy)]
pub struct TerrainTriangle(pub TerrainVertex, pub TerrainVertex, pub TerrainVertex);

impl TerrainTriangle {
    /// Returns an array of the vertices of the triangle.
    fn vertices(&self) -> [TerrainVertex; 3] {
        [self.0, self.1, self.2]
    }
}

/// A quad that stores the vertices for a [`TerrainMesh`].
#[derive(Debug, Clone, Copy)]
pub struct TerrainQuad(
    pub TerrainVertex,
    pub TerrainVertex,
    pub TerrainVertex,
    pub TerrainVertex,
);

impl TerrainQuad {
    /// Returns an array of the vertices of the quad.
    fn vertices(&self) -> [TerrainVertex; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

impl Mul<Transform> for TerrainQuad {
    type Output = Self;

    fn mul(self, rhs: Transform) -> Self::Output {
        let matrix = rhs.to_matrix();
        TerrainQuad(
            matrix * self.0,
            matrix * self.1,
            matrix * self.2,
            matrix * self.3,
        )
    }
}
