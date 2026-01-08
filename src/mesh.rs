//! Convertion between a [`BinaryCurveCollection`] into a Bevy [`Mesh`]

use bevy_asset::RenderAssetUsages;
use bevy_mesh::{Mesh, MeshBuilder, Meshable, PrimitiveTopology, VertexAttributeValues};

use crate::BinaryCurveCollection;

/// Mesh builder for a [`BinaryCurveCollection`]
pub struct BinaryCurveCollectionMeshBuilder<'a> {
    /// The [`BinaryCurveCollection`] from which the mesh will be built from
    bcc: &'a BinaryCurveCollection,
    /// Render asset usage. Assets with [`RenderAssetUsages::RENDER_WORLD`] will have
    /// their data moved to the GPU and will become inaccessable from the Cpu.
    render_asset_usages: RenderAssetUsages,
}

impl BinaryCurveCollectionMeshBuilder<'_> {
    /// Prepares a chunk into a value to be inserted into [`VertexAttributeValues`]
    /// on a Y-up coordinate system
    fn y_up(chunk: &[f32]) -> [f32; 3] {
        let Ok(vertices): Result<[f32; 3], _> = chunk.try_into() else {
            unreachable!("Chunk must contain 3 components.");
        };
        vertices
    }

    /// Prepares a chunk into a value to be inserted into [`VertexAttributeValues`]
    /// on a Z-up coordinate system. This requires converting to Bevy's Y-up coordinate
    /// system.
    fn z_up(chunk: &[f32]) -> [f32; 3] {
        let Ok(vertices): Result<[f32; 3], _> = chunk.try_into() else {
            unreachable!("Chunk must contain 3 components.");
        };
        [vertices[0], -vertices[2], vertices[1]]
    }
}

impl MeshBuilder for BinaryCurveCollectionMeshBuilder<'_> {
    fn build(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip, self.render_asset_usages);

        let Ok(number_of_control_points) =
            usize::try_from(self.bcc.header.number_of_control_points)
        else {
            unreachable!("Number of control points exceed usize::MAX.");
        };
        let looping_curves = self.bcc.looping.iter().filter(|s| **s).count();

        let vertices = if self.bcc.header.dimensions == 3 {
            debug_assert_eq!(self.bcc.control_points.len() % 3, 0);
            let mapper: fn(&[f32]) -> [f32; 3] = match self.bcc.header.up_direction {
                1 => Self::y_up,
                2 => Self::z_up,
                _ => unreachable!("Invalid up direction."),
            };
            let vertices = self
                .bcc
                .control_points
                .chunks(3)
                .map(mapper)
                .collect::<Vec<_>>();
            VertexAttributeValues::Float32x3(vertices)
        } else {
            unreachable!("Can only work with 3d curves.");
        };
        // This will have all control points, +1 for each looping curve to add the first
        // index of the curve at the end of the list, + (number_of_curves - 1) to include
        // primitive restarts
        let indices_len = number_of_control_points + looping_curves + self.bcc.looping.len() - 1;
        let mut indices = Vec::with_capacity(indices_len);
        #[cfg(debug_assertions)]
        let indices_capacity = indices.capacity();

        let first_control_points_iter = self.bcc.first_control_points.windows(2);
        let looping_iter = self.bcc.looping.iter();
        for (first_control_points, looping) in first_control_points_iter.zip(looping_iter) {
            let [l, r] = first_control_points else {
                unreachable!("Window must have 2 values.");
            };
            let last = *r == number_of_control_points;

            let Ok(r) = u32::try_from(*r) else {
                panic!("Could not fit control points in indices list.");
            };
            let Ok(l) = u32::try_from(*l) else {
                panic!("Could not fit control points in indices list.");
            };

            indices.extend(l..r);
            if *looping {
                indices.push(l);
            }
            if !last {
                indices.push(u32::MAX);
            }
        }

        #[cfg(debug_assertions)]
        debug_assert_eq!(indices_capacity, indices.len());

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_indices(bevy_mesh::Indices::U32(indices));

        mesh
    }
}

impl<'a> Meshable for &'a BinaryCurveCollection {
    type Output = BinaryCurveCollectionMeshBuilder<'a>;

    fn mesh(&self) -> Self::Output {
        BinaryCurveCollectionMeshBuilder {
            bcc: self,
            render_asset_usages: Default::default(),
        }
    }
}
