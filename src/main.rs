//! handle_mouse_input: Input<MouseButton> -> Event<TodoAction>
//! -> update_todo_model: Event<TodoAction> -> (Todo, Model)
//! ->-> display_todos: (Todo, Model) -> (Text2dBundle, View)
//! ->-> update_displayed_todos: (Todo, Model) -> (Text2dBundle, View)

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<TodoAction>()
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, handle_mouse_input)
        .add_systems(Update, update_todo_model)
        .add_systems(Update, display_todos.after(update_todo_model))
        // .add_systems(Update, update_displayed_todos.after(update_todo_model))
        .run();
}

fn setup(mut commands: Commands) {
    println!("Hello, world!");
    commands.spawn(Camera2dBundle::default());
}

fn handle_mouse_input(
    mut temp: Local<usize>,
    mouse: Res<Input<MouseButton>>,
    mut actions: EventWriter<TodoAction>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        println!("new todo");
        actions.send(TodoAction::Create(format!("new todo {}", *temp)));
        *temp += 1;
    }
}

// TODO: handle ordering of todos

/// Flush after this
fn update_todo_model(
    mut commands: Commands,
    mut actions: EventReader<TodoAction>,
    mut todos: Query<&mut Todo, ModelOnly>,
) {
    for action in actions.iter() {
        match action {
            TodoAction::Create(text) => {
                commands.spawn((
                    Todo {
                        text: text.clone(),
                        checked: false,
                    },
                    Model,
                ));
            }
            TodoAction::Delete(e) => {
                commands.entity(*e).despawn();
            }
            TodoAction::UpdateChecked(e, checked) => {
                todos.get_mut(*e).unwrap().checked = *checked;
            }
            TodoAction::UpdateText(e, text) => {
                todos.get_mut(*e).unwrap().text = text.clone();
            }
        }
    }
}

fn display_todos(mut commands: Commands, todos: Query<&Todo, (Added<Todo>, ModelOnly)>) {
    for todo in todos.iter() {
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(&todo.text, default()),
                ..default()
            },
            View,
        ));
    }
}

fn update_displayed_todos(
    mut commands: Commands,
    todos: Query<&Todo, (Changed<Todo>, ModelOnly)>,
    views: Query<&mut Text, ViewOnly>,
) {
    for todo in todos.iter() {
        todo!()
    }
}

#[derive(Component)]
struct Todo {
    text: String,
    checked: bool,
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
    UpdateChecked(ModelEntity, bool),
    UpdateText(ModelEntity, String),
}
