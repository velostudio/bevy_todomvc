//! handle_mouse_input: Input<MouseButton> -> Event<TodoAction>
//! -> update_todo_model: Event<TodoAction> -> (Todo, Model)
//! ->-> display_todos: (Todo, Model) -> (Text2dBundle, View)
//! ->-> update_displayed_todos: (Todo, Model) -> (Text2dBundle, View)

use bevy::prelude::*;

#[derive(Event)]
struct SetFocus(Option<Entity>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<TodoAction>()
        .add_event::<SetFocus>()
        .init_resource::<Focus>()
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, handle_typing.before(handle_focus))
        .add_systems(PreUpdate, handle_delete_todo_click.before(handle_focus))
        .add_systems(PreUpdate, handle_enter.before(handle_focus))
        .add_systems(PreUpdate, handle_check_todo_click.before(handle_focus))
        .add_systems(PreUpdate, handle_todo_text_click.before(handle_focus))
        .add_systems(PreUpdate, handle_text_input_click.before(handle_focus))
        .add_systems(PreUpdate, handle_focus)
        .add_systems(Update, update_todo_model)
        .add_systems(Update, display_todos.after(update_todo_model))
        .add_systems(Update, update_displayed_todos_text.after(update_todo_model))
        .add_systems(
            Update,
            update_displayed_todos_checked.after(update_todo_model),
        )
        .add_systems(PostUpdate, remove_displayed_todos)
        .run();
}

#[derive(Component)]
struct TodoList;

#[derive(Component)]
struct TodoInput;

#[derive(Component)]
struct TodoRootView;

#[derive(Component)]
struct TodoTextView;

#[derive(Component)]
struct TodoCheckView;

#[derive(Component)]
struct TodoDeleteView;

fn setup(mut commands: Commands, mut set_focus: EventWriter<SetFocus>) {
    commands.spawn(Camera2dBundle::default());
    let main = commands
        .spawn(NodeBundle {
            style: Style {
                height: Val::Percent(100.),
                width: Val::Percent(100.),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    let todo_input_btn = commands
        .spawn((
            ButtonBundle {
                border_color: Color::GREEN.into(),
                style: Style {
                    justify_content: JustifyContent::Center,
                    overflow: Overflow::clip(),
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(10.)),
                    width: Val::Px(200.),
                    height: Val::Px(40.),
                    border: UiRect::all(Val::Px(4.)),
                    ..Default::default()
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            },
            TodoInput,
        ))
        .id();
    let todo_input_txt = commands
        .spawn((
            TextBundle {
                text: Text::from_section("", default()),
                ..default()
            },
            TodoInput,
        ))
        .id();
    let todo_list = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ..Default::default()
            },
            TodoList,
        ))
        .id();
    commands.entity(todo_input_btn).add_child(todo_input_txt);
    commands.entity(main).add_child(todo_input_btn);
    commands.entity(main).add_child(todo_list);
    set_focus.send(SetFocus(Some(todo_input_txt)));
}

fn handle_delete_todo_click(
    mut actions: EventWriter<TodoAction>,
    mut delete_interaction_q: Query<
        (&Interaction, &View),
        (Changed<Interaction>, With<TodoDeleteView>),
    >,
    mut set_focus: EventWriter<SetFocus>,
) {
    for (interaction, view) in delete_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            actions.send(TodoAction::Delete(view.0));
            set_focus.send(SetFocus(None));
        }
    }
}

fn handle_todo_text_click(
    mut check_interaction_q: Query<
        (&Interaction, Entity),
        (Changed<Interaction>, With<TodoTextView>),
    >,
    mut todo_text_q: Query<(Entity, &Parent, &Text), With<TodoTextView>>,
    mut set_focus: EventWriter<SetFocus>,
) {
    for (interaction, clicked_entity) in check_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            for (entity, parent, _) in todo_text_q.iter_mut() {
                if parent.get() == clicked_entity {
                    set_focus.send(SetFocus(Some(entity)));
                }
            }
        }
    }
}

fn handle_text_input_click(
    mut check_interaction_q: Query<(&Interaction, Entity), (Changed<Interaction>, With<TodoInput>)>,
    mut todo_text_q: Query<(Entity, &Parent, &Text), With<TodoInput>>,
    mut set_focus: EventWriter<SetFocus>,
) {
    for (interaction, clicked_entity) in check_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            for (entity, parent, _) in todo_text_q.iter_mut() {
                if parent.get() == clicked_entity {
                    set_focus.send(SetFocus(Some(entity)));
                }
            }
        }
    }
}

fn handle_check_todo_click(
    mut actions: EventWriter<TodoAction>,
    model: Query<&TodoChecked, ModelOnly>,
    mut check_interaction_q: Query<
        (&Interaction, &View),
        (Changed<Interaction>, With<TodoCheckView>),
    >,
) {
    for (interaction, view) in check_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            actions.send(TodoAction::UpdateChecked(
                view.0,
                !model.get(view.0).unwrap().0,
            ));
        }
    }
}

fn handle_enter(
    mut actions: EventWriter<TodoAction>,
    mut todo_input_q: Query<(&mut Text, Entity), With<TodoInput>>,
    keys: Res<Input<KeyCode>>,
    focus: Res<Focus>,
) {
    if focus.is_none() {
        return;
    }
    if keys.just_pressed(KeyCode::Return) {
        if let Ok((mut todo_input_text, _)) = todo_input_q.get_mut(focus.unwrap()) {
            actions.send(TodoAction::Create(
                todo_input_text.sections[0].value.clone(),
            ));
            todo_input_text.sections[0].value = "".to_string();
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Focus(pub Option<Entity>);

fn handle_focus(mut set_focus_events: EventReader<SetFocus>, mut focus: ResMut<Focus>) {
    for ev in set_focus_events.iter() {
        *focus = Focus(ev.0)
    }
}

fn handle_typing(
    mut evr_char: EventReader<ReceivedCharacter>,
    mut todo_input_q: Query<&mut Text, (With<TodoInput>, Without<TodoTextView>)>,
    mut actions: EventWriter<TodoAction>,
    todo_text_q: Query<&View, With<TodoTextView>>,
    focus: Res<Focus>,
) {
    if focus.is_none() {
        return;
    }
    for ev in evr_char.iter() {
        if !ev.char.is_control() {
            // TODO: MVC is not used for todo input, but for todo list it is, should it be improved?
            if let Ok(mut todo_input_text) = todo_input_q.get_mut(focus.unwrap()) {
                todo_input_text.sections[0].value = format!(
                    "{}{}",
                    todo_input_text.sections[0].value,
                    ev.char.to_string(),
                );
            }

            if let Ok(view) = todo_text_q.get(focus.unwrap()) {
                actions.send(TodoAction::UpdateText(view.0, ev.char.to_string()));
            }
        }
    }
}

/// Flush after this
fn update_todo_model(
    mut commands: Commands,
    mut actions: EventReader<TodoAction>,
    mut todo_text: Query<&mut TodoText, ModelOnly>,
    mut todo_checked: Query<&mut TodoChecked, ModelOnly>,
) {
    for action in actions.iter() {
        match action {
            TodoAction::Create(text) => {
                commands.spawn((TodoText(text.clone()), TodoChecked(false), Model));
            }
            TodoAction::Delete(e) => {
                commands.entity(*e).despawn_recursive();
            }
            TodoAction::UpdateChecked(e, checked) => {
                todo_checked.get_mut(*e).unwrap().0 = *checked;
            }
            TodoAction::UpdateText(e, text) => {
                todo_text.get_mut(*e).unwrap().0 = text.clone();
            }
        }
    }
}

fn display_todos(
    mut commands: Commands,
    todos: Query<(&TodoText, ModelEntity), (Added<TodoText>, ModelOnly)>,
    todo_list_q: Query<Entity, With<TodoList>>,
) {
    for (todo, model_entity) in todos.iter() {
        let todo_list = todo_list_q.single();
        let todo_item = commands
            .spawn((
                NodeBundle {
                    style: Style {
                        margin: UiRect::all(Val::Px(5.)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                View(model_entity),
                TodoRootView,
            ))
            .id();
        let todo_check_btn = commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(40.),
                        height: Val::Px(40.),
                        justify_content: JustifyContent::Center,
                        overflow: Overflow::clip(),
                        ..Default::default()
                    },
                    background_color: Color::NONE.into(),
                    ..Default::default()
                },
                View(model_entity),
                TodoCheckView,
            ))
            .id();
        let todo_check_txt = commands
            .spawn((
                TextBundle {
                    text: Text::from_section("o", default()),
                    ..default()
                },
                View(model_entity),
            ))
            .id();
        commands.entity(todo_check_btn).add_child(todo_check_txt);
        let todo_btn = commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(200.),
                        height: Val::Px(40.),
                        overflow: Overflow::clip(),
                        ..Default::default()
                    },
                    background_color: Color::NONE.into(),
                    ..Default::default()
                },
                View(model_entity),
                TodoTextView,
            ))
            .id();
        let todo_txt = commands
            .spawn((
                TextBundle {
                    text: Text::from_section(&todo.0, default()),
                    ..default()
                },
                View(model_entity),
                TodoTextView,
            ))
            .id();

        let todo_delete_btn = commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        width: Val::Px(40.),
                        height: Val::Px(40.),
                        margin: UiRect {
                            left: Val::Px(10.),
                            right: Val::Px(10.),
                            ..Default::default()
                        },
                        overflow: Overflow::clip(),
                        ..Default::default()
                    },
                    background_color: Color::NONE.into(),
                    ..Default::default()
                },
                View(model_entity),
                TodoDeleteView,
            ))
            .id();
        let todo_delete_txt = commands
            .spawn((
                TextBundle {
                    text: Text::from_section("x", default()),
                    ..default()
                },
                View(model_entity),
            ))
            .id();
        commands.entity(todo_btn).add_child(todo_txt);
        commands.entity(todo_item).add_child(todo_check_btn);
        commands.entity(todo_delete_btn).add_child(todo_delete_txt);
        commands.entity(todo_item).add_child(todo_btn);
        commands.entity(todo_item).add_child(todo_delete_btn);
        commands.entity(todo_list).add_child(todo_item);
    }
}

fn update_displayed_todos_text(
    mut views: Query<(&mut Text, &View, &TodoTextView), ViewOnly>,
    todos_text: Query<&TodoText, (Changed<TodoText>, ModelOnly)>,
) {
    for (mut text, view, _) in views.iter_mut() {
        if let Some(todo) = todos_text.get(view.0).ok() {
            text.sections[0].value = format!("{}{}", text.sections[0].value, todo.0.clone());
        }
    }
}

fn update_displayed_todos_checked(
    mut views: Query<(&mut Text, &View), ViewOnly>,
    todos_checked: Query<&TodoChecked, (Changed<TodoChecked>, ModelOnly)>,
) {
    for (mut text, view) in views.iter_mut() {
        if let Some(todo) = todos_checked.get(view.0).ok() {
            if todo.0 {
                text.sections[0].style.color = Color::GRAY;
            } else {
                text.sections[0].style.color = Color::WHITE;
            }
        }
    }
}

fn remove_displayed_todos(
    mut commands: Commands,
    views: Query<(Entity, &View, &TodoRootView), ViewOnly>,
    mut removed: RemovedComponents<TodoChecked>,
) {
    for entity in removed.iter() {
        // TODO: O(n^2) is too expensive here, should we have 2-way-relationship?
        for (view_entity, view, _) in views.iter() {
            if view.0 == entity {
                commands.entity(view_entity).despawn_recursive();
            }
        }
    }
}

#[derive(Component)]
struct TodoText(String);

#[derive(Component)]
struct TodoChecked(bool);

/// Marker component to indicate that this entity is part of the Model
///
/// Mutually exclusive with [`View`]
#[derive(Component)]
struct Model;

/// Marker component to indicate that this entity is part of the View
///
/// Mutually exclusive with [`Model`]
#[derive(Component)]
struct View(ModelEntity);

type ModelOnly = (With<Model>, Without<View>);

type ViewOnly = (Without<Model>, With<View>);

type ModelEntity = Entity;

#[derive(Event)]
enum TodoAction {
    Create(String),
    Delete(ModelEntity),
    UpdateChecked(ModelEntity, bool),
    UpdateText(ModelEntity, String),
}
