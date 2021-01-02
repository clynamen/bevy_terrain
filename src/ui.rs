use bevy::prelude::*;
use bevy_render::mesh::mesh_resource_provider_system;
use bevy_terrain::{terrain::{Terrain}, terrain_rtin::rtin_load_terrain_bitmap};

pub struct ButtonMaterials {
    shaded: Handle<ColorMaterial>,
    wireframe: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
}


impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            shaded: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            wireframe: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum MeshStyle {
    shaded, 
    wireframe
} 

pub fn button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Mutated<Interaction>, With<Button>),
    >,
    mut terrain_query: Query<
        (Entity, &mut Handle<Mesh>, &Terrain),
    >,
    mut text_query: Query<&mut Text>,
    mut meshes: ResMut<Assets<Mesh>>,
    commands: &mut Commands,
) {
    let mut new_mesh_type = Option::<MeshStyle>::None;

    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();

        let shaded_enabled = text.value == "shaded";

        match *interaction {
            Interaction::Clicked => {
                if shaded_enabled {
                    text.value = "wireframe".to_string();
                    new_mesh_type = Some(MeshStyle::wireframe);
                    *material = button_materials.wireframe.clone();
                } else {
                    text.value = "shaded".to_string();
                    new_mesh_type = Some(MeshStyle::shaded);
                    *material = button_materials.shaded.clone();
                }
            }
            Interaction::Hovered => {
                // text.value = "Hover".to_string();
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                if shaded_enabled {
                    *material = button_materials.wireframe.clone();
                } else {
                    *material = button_materials.shaded.clone();
                }
            }
        }
    }

    if new_mesh_type.is_some() {
        let mesh_type = new_mesh_type.unwrap();

        let error_threshold = 0.25f32;
        let filename = "terrain.png";

        let mesh = rtin_load_terrain_bitmap(
            filename, error_threshold, 10.0, 
                mesh_type == MeshStyle::wireframe).unwrap();

        let new_mesh_handle = meshes.add(mesh);

        for (entity, mut mesh, _terrain) in terrain_query.iter_mut() {
            println!("{:?}", mesh_type);
            // meshes.remove(mesh.id);
            // mesh.id = new_mesh_handle.id;
            commands.remove_one::<Handle<Mesh>>(entity);
            commands.set_current_entity(entity);
            commands.with(new_mesh_handle.clone());
        }
    }
}


pub fn setup_ui(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    commands
        // ui camera
        .spawn(CameraUiBundle::default())
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.shaded.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    value: "shaded".to_string(),
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    style: TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        });
}

