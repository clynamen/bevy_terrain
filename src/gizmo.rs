use bevy::prelude::*;

pub struct AxisGizmo {

}

pub fn add_axis_gizmo(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    transform: Transform
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.1, subdivisions: 10 })),
            material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
            transform: transform,
            ..Default::default()
        }).with(AxisGizmo{})
        .with_children(|parent: &mut ChildBuilder| {
            parent
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                transform: Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
                ..Default::default()
            })
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
                transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                ..Default::default()
            })
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ..Default::default()
            });
        });
}