use bevy::prelude::*;
use bevy_fly_camera::FlyCamera;
use bevy_terrain::terrain_rtin::rtin_load_terrain_bitmap;
use bevy_terrain::terrain_common::Terrain;

pub struct ButtonMaterials {
    shaded: Handle<ColorMaterial>,
    wireframe: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
}


impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            shaded: materials.add(Color::rgb(0.35, 0.35, 0.35).into()),
            wireframe: materials.add(Color::rgb(0.35, 0.35, 0.35).into()),
            hovered: materials.add(Color::rgb(0.55, 0.55, 0.55).into()),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum MeshStyle {
    Shaded, 
    Wireframe
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
                    new_mesh_type = Some(MeshStyle::Wireframe);
                    *material = button_materials.wireframe.clone();
                } else {
                    text.value = "shaded".to_string();
                    new_mesh_type = Some(MeshStyle::Shaded);
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

        let error_threshold = 0.10f32;
        let filename = "terrain.png";

        let mesh = rtin_load_terrain_bitmap(
            filename, error_threshold, 10.0, 
                mesh_type == MeshStyle::Wireframe).unwrap();

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

pub struct Menu{

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
            visible: Visible{
                is_visible: false, 
                is_transparent: false},
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
        }).with(Menu{})
        .with_children(|parent| {
            parent.spawn(TextBundle {
                visible: Visible{
                    is_visible: false, 
                    is_transparent: false
                },
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
            }).with(Menu{});
        });
}


pub fn show_ui_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<
        &mut FlyCamera
    >,
    mut ui_query: Query<(&mut Visible, &Menu)>
) {


   let mut update_camera = false;
   let mut enable_camera_movement = false;
   let mut show_ui = false;

   if keyboard_input.just_pressed(KeyCode::Tab) {
    update_camera = true;
    enable_camera_movement = false;
    show_ui = true;
   } else if keyboard_input.just_released(KeyCode::Tab) {
    update_camera = true;
    enable_camera_movement = true;
    show_ui = false;
   }

    if update_camera {
        for mut camera in camera_query.iter_mut() {
            camera.enabled = enable_camera_movement; 
        } 
        for (mut visible, _menu) in ui_query.iter_mut() {
            visible.is_visible = show_ui;
        } 

    }

}