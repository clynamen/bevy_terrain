mod ui;

use bevy_terrain::terrain_common::{Terrain, TerrainImageLoadOptions};
use bevy_terrain::{gizmo::add_axis_gizmo, terrain::{terrain_example}, terrain_material::TerrainMaterial};
use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_render::{
    mesh::{Mesh},
};
use bevy_terrain::terrain_rtin::rtin_terrain_example;
use bevy_terrain::terrain_material::add_terrain_material;
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
        .add_asset::<TerrainMaterial>()
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
    materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
    pipelines: ResMut<Assets<PipelineDescriptor>>,
    shaders: ResMut<Assets<Shader>>,
    render_graph: ResMut<RenderGraph>
) {
    let terrain_mesh = rtin_terrain_example();

    let terrain_mesh_handle = meshes.add(terrain_mesh);

    let pipeline_handle = add_terrain_material(
        pipelines, shaders, render_graph);


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
            transform: Transform::from_translation(Vec3::new(0.0, 20.0, 0.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        })
        .with(FlyCamera{
            pitch: 180.0,
            ..Default::default()
        });

    add_axis_gizmo(commands, meshes, materials, 
        Transform::from_translation(Vec3::new(0f32, 0f32, 0f32)));

    setup_ui(commands,
        asset_server,
        button_materials);
}
