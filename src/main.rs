use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup() {
    println!("Hello, world!");
}

/// Marker component to indicate that this entity is part of the Model
#[derive(Component)]
struct Model;

/// Marker component to indicate that this entity is part of the View
#[derive(Component)]
struct View;
