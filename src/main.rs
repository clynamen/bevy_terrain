mod ui;

use bevy_terrain::{gizmo::add_axis_gizmo, terrain::{terrain_example, Terrain}, terrain_shader::MyMaterialWithVertexColorSupport};
use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_render::{
    pipeline::PrimitiveTopology,
    mesh::{Mesh, VertexAttributeValues, Indices},
};
use bevy_terrain::terrain_rtin::rtin_terrain_example;
use ui::{ButtonMaterials, button_system, setup_ui, show_ui_system};

use bevy::{
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{RenderGraph},
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
        .add_system(show_ui_system.system())
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
            mesh: terrain_mesh_handle,
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        }).with(Terrain{})
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

    add_axis_gizmo(commands, meshes, materials, 
        Transform::from_translation(Vec3::new(0f32, 0f32, 0f32)));

    setup_ui(commands,
        asset_server,
        button_materials);
}
