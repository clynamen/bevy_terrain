mod ui;

use std::sync::mpsc;
use bevy_terrain::{terrain::{terrain_example, Terrain}, terrain_shader::MyMaterialWithVertexColorSupport};
use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_render::{
    pipeline::PrimitiveTopology,
    mesh::{Mesh, VertexAttributeValues, Indices},
};
use bevy_terrain::terrain_rtin::rtin_terrain_example;
use ui::{setup_ui, ButtonMaterials, button_system};

use bevy::{
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        shader::{ShaderStage, ShaderStages},
    },
};

fn main() {

    terrain_example();

    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(FlyCameraPlugin)
        .add_asset::<MyMaterialWithVertexColorSupport>()
        .init_resource::<ButtonMaterials>()
        .add_startup_system(setup.system())
        .add_system(button_system.system())
        .run();
}


/// set up a simple 3D scene
fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_materials: ResMut<Assets<MyMaterialWithVertexColorSupport>>,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut render_graph: ResMut<RenderGraph>
) {
    // let terrain_mesh = make_terrain_mesh();
    // let terrain_mesh = terrain_example();
    let terrain_mesh = rtin_terrain_example();

    let terrain_mesh_handle = meshes.add(terrain_mesh);

    let pipeline_handle = bevy_terrain::terrain_shader::add_MyMaterialWithVertexColorSupport(
        pipelines, shaders, render_graph);

    let terrain_material = terrain_materials.add(MyMaterialWithVertexColorSupport {});

    // add entities to the world
    commands
        // terrain
        .spawn(MeshBundle {
            // mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            mesh: terrain_mesh_handle,
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        }).with(Terrain{})
        // cube
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            transform: Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)),
            ..Default::default()
        })
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
            material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 4.0, 0.0)),
            ..Default::default()
        })
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            ..Default::default()
        })
        // light
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 4.0, 0.0)),
            ..Default::default()
        })
        // camera
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(10.0, 30.0, 10.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        })
        .with(FlyCamera::default());

    setup_ui(commands,
        asset_server,
        button_materials);
}


fn make_terrain_mesh() -> Mesh {

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let vertices : Vec<[f32; 3]> = vec![
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
    ];
    let normals = vec![[0.0, 0.0, 1.0]; 3];

    let indices = vec![0, 1, 2]; 
    let uvs = vec![[0.0, 0.0, 0.0]; vertices.len()];

    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float3(vertices));
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float3(normals));
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float3(uvs));
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}