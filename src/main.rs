use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<TodoAction>()
        .add_systems(Startup, setup)
        .run();
}

fn setup() {
    println!("Hello, world!");
}

/// Marker component to indicate that this entity is part of the Model
///
/// Mutually exclusive with [`View`]
#[derive(Component)]
struct Model;

/// Marker component to indicate that this entity is part of the View
///
/// Mutually exclusive with [`Model`]
#[derive(Component)]
struct View;

type ModelOnly = (With<Model>, Without<View>);

type ViewOnly = (Without<Model>, With<View>);

type ModelEntity = Entity;

type ViewEntity = Entity;

#[derive(Event)]
enum TodoAction {
    Create(String),
    Delete(ModelEntity),
    UpdateStatus(ModelEntity, bool),
    UpdateText(ModelEntity, String),
}
