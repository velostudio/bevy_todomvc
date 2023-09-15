//! handle_mouse_input: Input<MouseButton> -> Event<ModelTodoAction>
//! -> update_todo_model: Event<ModelTodoAction> -> SpawnOrMutate(Todo, Model)
//! ->-> display_todos: (Todo, Model) -> Spawn((Text2dBundle, View))
//! ->-> update_displayed_todos: (Todo, Model) -> Mutate((Text2dBundle, View))

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<ModelTodoAction>()
        .add_event::<ModelInputAction>()
        .add_event::<SetFocus>()
        .init_resource::<Focus>()
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_ui)
        .add_systems(PreUpdate, handle_typing.before(handle_focus))
        .add_systems(
            PreUpdate,
            handle_interaction_for_delete.before(handle_focus),
        )
        .add_systems(PreUpdate, handle_enter.before(handle_focus))
        .add_systems(PreUpdate, handle_check_todo_click.before(handle_focus))
        .add_systems(PreUpdate, handle_todo_text_click.before(handle_focus))
        .add_systems(PreUpdate, handle_text_input_click.before(handle_focus))
        .add_systems(PreUpdate, handle_focus)
        .add_systems(Update, update_todo_model)
        .add_systems(Update, update_input_model)
        .add_systems(Update, display_todos.after(update_todo_model))
        .add_systems(Update, display_text_input.after(update_input_model))
        .add_systems(Update, update_displayed_todos_text.after(update_todo_model))
        .add_systems(
            Update,
            update_displayed_todos_checked.after(update_todo_model),
        )
        .add_systems(
            Update,
            update_displayed_input_text.after(update_input_model),
        )
        .add_systems(PostUpdate, remove_displayed_todos)
        .run();
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Focus(pub Option<Entity>);

#[derive(Event)]
struct SetFocus(Option<Entity>);

#[derive(Component)]
struct TodoInputContainer;

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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_ui(mut commands: Commands, mut input_actions: EventWriter<ModelInputAction>) {
    let main = commands
        .spawn(NodeBundle {
            style: Style {
                height: Val::Percent(100.),
                width: Val::Percent(100.),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .id();
    let todo_input_container = commands
        .spawn((NodeBundle::default(), TodoInputContainer))
        .id();
    let todo_list = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            TodoList,
        ))
        .id();

    // main
    // - todo_input_container
    // - todo_list
    commands.entity(main).add_child(todo_input_container);
    commands.entity(main).add_child(todo_list);

    input_actions.send(ModelInputAction::Create("".to_string()));
}

fn handle_interaction_for_delete(
    mut actions: EventWriter<ModelTodoAction>,
    mut delete_interaction_q: Query<
        (&Interaction, &View),
        (Changed<Interaction>, With<TodoDeleteView>),
    >,
    mut set_focus: EventWriter<SetFocus>,
) {
    for (interaction, view) in delete_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            actions.send(ModelTodoAction::Delete(view.0));
            set_focus.send(SetFocus(None));
        }
    }
}

fn handle_todo_text_click(
    mut check_interaction_q: Query<
        (&Interaction, Entity),
        (Changed<Interaction>, With<TodoTextView>),
    >,
    mut todo_text_q: Query<(Entity, &Parent), (With<Text>, With<TodoTextView>)>,
    mut set_focus: EventWriter<SetFocus>,
) {
    for (interaction, clicked_entity) in check_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            for (entity, parent) in todo_text_q.iter_mut() {
                if parent.get() == clicked_entity {
                    set_focus.send(SetFocus(Some(entity)));
                }
            }
        }
    }
}

fn handle_text_input_click(
    mut check_interaction_q: Query<(&Interaction, Entity), (Changed<Interaction>, With<TodoInput>)>,
    mut todo_text_q: Query<(Entity, &Parent), (With<Text>, With<TodoInput>)>,
    mut set_focus: EventWriter<SetFocus>,
) {
    for (interaction, clicked_entity) in check_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            for (entity, parent) in todo_text_q.iter_mut() {
                if parent.get() == clicked_entity {
                    set_focus.send(SetFocus(Some(entity)));
                }
            }
        }
    }
}

fn handle_check_todo_click(
    mut actions: EventWriter<ModelTodoAction>,
    model: Query<&ModelTodoChecked, ModelOnly>,
    mut check_interaction_q: Query<
        (&Interaction, &View),
        (Changed<Interaction>, With<TodoCheckView>),
    >,
) {
    for (interaction, view) in check_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            actions.send(ModelTodoAction::UpdateChecked(
                view.0,
                !model.get(view.0).unwrap().0,
            ));
        }
    }
}

fn handle_enter(
    mut actions: EventWriter<ModelTodoAction>,
    mut todo_input_q: Query<(&mut Text, Entity), With<TodoInput>>,
    keys: Res<Input<KeyCode>>,
    focus: Res<Focus>,
) {
    if focus.is_none() {
        return;
    }
    if keys.just_pressed(KeyCode::Return) {
        if let Ok((mut todo_input_text, _)) = todo_input_q.get_mut(focus.unwrap()) {
            actions.send(ModelTodoAction::Create(
                todo_input_text.sections[0].value.clone(),
            ));
            todo_input_text.sections[0].value = "".to_string();
        }
    }
}

fn handle_focus(mut set_focus_events: EventReader<SetFocus>, mut focus: ResMut<Focus>) {
    for ev in set_focus_events.iter() {
        *focus = Focus(ev.0)
    }
}

/// The question is how to express this at creation (setup_ui)
///
/// We want the equivalent of JS `input.addEventListener('oninput', (e) => { model.value = e.target.value })`
fn handle_typing(
    mut evr_char: EventReader<ReceivedCharacter>,
    mut todo_actions: EventWriter<ModelTodoAction>,
    mut input_actions: EventWriter<ModelInputAction>,
    mut todo_input_q: Query<(&Text, &View), (With<TodoInput>, Without<TodoTextView>)>,
    todo_text_q: Query<(&Text, &View), With<TodoTextView>>,
    focus: Res<Focus>,
) {
    if focus.is_none() {
        return;
    }
    for ev in evr_char.iter() {
        if !ev.char.is_control() {
            if let Ok((text, view)) = todo_input_q.get_mut(focus.unwrap()) {
                input_actions.send(ModelInputAction::UpdateText(
                    view.0,
                    format!("{}{}", text.sections[0].value, ev.char.to_string()),
                ));
            }

            if let Ok((text, view)) = todo_text_q.get(focus.unwrap()) {
                todo_actions.send(ModelTodoAction::UpdateText(
                    view.0,
                    format!("{}{}", text.sections[0].value, ev.char.to_string()),
                ));
            }
        }
    }
}

/// Flush after this
fn update_todo_model(
    mut commands: Commands,
    mut actions: EventReader<ModelTodoAction>,
    mut todo_text: Query<&mut ModelTodoText, ModelOnly>,
    mut todo_checked: Query<&mut ModelTodoChecked, ModelOnly>,
) {
    for action in actions.iter() {
        match action {
            ModelTodoAction::Create(text) => {
                commands.spawn((ModelTodoText(text.clone()), ModelTodoChecked(false), Model));
            }
            ModelTodoAction::Delete(e) => {
                commands.entity(*e).despawn_recursive();
            }
            ModelTodoAction::UpdateChecked(e, checked) => {
                todo_checked.get_mut(*e).unwrap().0 = *checked;
            }
            ModelTodoAction::UpdateText(e, text) => {
                todo_text.get_mut(*e).unwrap().0 = text.clone();
            }
        }
    }
}

fn update_input_model(
    mut commands: Commands,
    mut actions: EventReader<ModelInputAction>,
    mut input_text: Query<&mut ModelInputText, ModelOnly>,
) {
    for action in actions.iter() {
        match action {
            ModelInputAction::Create(text) => {
                commands.spawn((ModelInputText(text.clone()), Model));
            }
            ModelInputAction::UpdateText(e, text) => {
                input_text.get_mut(*e).unwrap().0 = text.clone();
            }
        }
    }
}

fn display_text_input(
    mut commands: Commands,
    inputs: Query<(ModelEntity, &ModelInputText), (Added<ModelInputText>, ModelOnly)>,
    todo_input_container: Query<Entity, With<TodoInputContainer>>,
    mut set_focus: EventWriter<SetFocus>,
) {
    for (model_entity, input) in inputs.iter() {
        let todo_input = commands
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
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                },
                View(model_entity),
                TodoInput,
            ))
            .id();
        let todo_input_text = commands
            .spawn((
                TextBundle {
                    text: Text::from_section(input.0.clone(), default()),
                    ..default()
                },
                View(model_entity),
                TodoInput,
            ))
            .id();

        // - todo_input_container
        //   - todo_input
        //      - todo_input_text
        commands
            .entity(todo_input_container.single())
            .add_child(todo_input);
        commands.entity(todo_input).add_child(todo_input_text);

        set_focus.send(SetFocus(Some(todo_input_text)));
    }
}

/// Helper function
fn display_checked(checked: &ModelTodoChecked) -> &'static str {
    if checked.0 {
        "[x]"
    } else {
        "[ ]"
    }
}

fn display_todos(
    mut commands: Commands,
    todos: Query<
        (ModelEntity, &ModelTodoText, &ModelTodoChecked),
        (Added<ModelTodoText>, Added<ModelTodoChecked>, ModelOnly),
    >,
    todo_list_q: Query<Entity, With<TodoList>>,
) {
    for (model_entity, todo, checked) in todos.iter() {
        let todo_list = todo_list_q.single();
        let todo_item = commands
            .spawn((
                NodeBundle {
                    style: Style {
                        margin: UiRect::all(Val::Px(5.)),
                        ..default()
                    },
                    ..default()
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
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                },
                View(model_entity),
                TodoCheckView,
            ))
            .id();
        let todo_check_txt = commands
            .spawn((
                TextBundle {
                    text: Text::from_section(display_checked(checked), default()),
                    ..default()
                },
                View(model_entity),
                TodoCheckView,
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
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
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
                            ..default()
                        },
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
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

        // todo_list
        // - [todo_item]
        //   - todo_check_btn
        //   - todo_btn
        //      - todo_txt
        //   - todo_delete_btn
        //      - todo_delete_txt
        commands.entity(todo_list).add_child(todo_item);
        commands.entity(todo_item).add_child(todo_check_btn);
        commands.entity(todo_item).add_child(todo_btn);
        commands.entity(todo_btn).add_child(todo_txt);
        commands.entity(todo_item).add_child(todo_delete_btn);
        commands.entity(todo_delete_btn).add_child(todo_delete_txt);
    }
}

fn update_displayed_todos_text(
    todos_text: Query<&ModelTodoText, (Changed<ModelTodoText>, ModelOnly)>,
    mut views: Query<(&mut Text, &View, &TodoTextView), ViewOnly>,
) {
    for (mut text, view, _) in views.iter_mut() {
        if let Ok(todo) = todos_text.get(view.0) {
            text.sections[0].value = todo.0.clone();
        }
    }
}

/// these updates are push-based but we kinda want pull-based from an authoring perspective
/// so that it's easier to locate a specific entity
/// this should be equivalent to...
/// We need to store a reference to a model entity on every dependent view entity
/// We also need to store a reference to a model entity on every view entity that sends an action which is not ideal
fn update_displayed_todos_checked(
    model_checked: Query<&ModelTodoChecked, (Changed<ModelTodoChecked>, ModelOnly)>,
    mut views: Query<(&mut Text, &View, Option<&TodoCheckView>), ViewOnly>,
) {
    for (mut text, view, maybe_checkbox) in views.iter_mut() {
        if let Ok(checked) = model_checked.get(view.0) {
            text.sections[0].style.color = if checked.0 { Color::GRAY } else { Color::WHITE };
            if maybe_checkbox.is_some() {
                text.sections[0].value = display_checked(checked).to_string();
            }
        }
    }
}

fn update_displayed_input_text(
    input_text: Query<&ModelInputText, (Changed<ModelInputText>, ModelOnly)>,
    mut views: Query<(&mut Text, &View, &TodoInput), ViewOnly>,
) {
    for (mut text, view, _) in views.iter_mut() {
        if let Ok(todo) = input_text.get(view.0) {
            text.sections[0].value = todo.0.clone();
        }
    }
}

fn remove_displayed_todos(
    mut commands: Commands,
    views: Query<(Entity, &View), (ViewOnly, With<TodoRootView>)>,
    mut removed: RemovedComponents<Model>,
) {
    for entity in removed.iter() {
        // TODO: O(n^2) is too expensive here, should we have 2-way-relationship?
        for (view_entity, view) in views.iter() {
            if view.0 == entity {
                commands.entity(view_entity).despawn_recursive();
            }
        }
    }
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
struct View(ModelEntity);

type ModelOnly = (With<Model>, Without<View>);

type ViewOnly = (Without<Model>, With<View>);

type ModelEntity = Entity;

/// Combined with `ModelTodoText` and `ModelTodoChecked`,
/// this is functionally equivalent to
/// ```rs
/// struct Todo {
///     text: String,
///     checked: bool,
/// }
///
/// struct Todos(Vec<Todo>);
///
/// impl Todos {
///     fn create(&mut self, text: String);
///     fn delete(&mut self, idx: usize);
///     fn update_checked(&mut self, idx: usize, checked: bool);
///     fn update_text(&mut self, idx: usize, text: String);
/// }
/// ```
///
/// Components are a stand-in for properties.
/// Events are a stand-in for methods.
#[derive(Event)]
enum ModelTodoAction {
    Create(String),
    Delete(ModelEntity),
    UpdateText(ModelEntity, String),
    UpdateChecked(ModelEntity, bool),
}

#[derive(Component)]
struct ModelTodoText(String);

#[derive(Component)]
struct ModelTodoChecked(bool);

#[derive(Event)]
enum ModelInputAction {
    Create(String),
    UpdateText(ModelEntity, String),
}

#[derive(Component)]
struct ModelInputText(String);
