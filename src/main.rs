#![allow(clippy::type_complexity)]

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
        .add_systems(PreUpdate, handle_deleter_interaction.before(handle_focus))
        .add_systems(PreUpdate, handle_enter.before(handle_focus))
        .add_systems(PreUpdate, handle_checkmark_interaction.before(handle_focus))
        .add_systems(PreUpdate, handle_text_interaction.before(handle_focus))
        .add_systems(PreUpdate, handle_input_interaction.before(handle_focus))
        .add_systems(PreUpdate, handle_focus)
        .add_systems(Update, update_todo_model)
        .add_systems(Update, update_input_model)
        .add_systems(Update, display_todos.after(update_todo_model))
        .add_systems(Update, display_text_input.after(update_input_model))
        .add_systems(Update, update_displayed_todos_text.after(update_todo_model))
        .add_systems(
            Update,
            update_displayed_todos_text_checked.after(update_todo_model),
        )
        .add_systems(
            Update,
            update_displayed_todos_checkmark_checked.after(update_todo_model),
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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_ui(mut commands: Commands, mut input_actions: EventWriter<ModelInputAction>) {
    let app_main = commands
        .spawn(NodeBundle {
            style: Style {
                height: Val::Percent(100.),
                width: Val::Percent(100.),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: colors::body_background().into(),
            ..default()
        })
        .id();

    let app_title = commands
        .spawn(TextBundle::from_section("todos", text_styles::title()))
        .id();

    let todo_main = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(550.),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::WHITE.into(),
            ..default()
        })
        .id();

    let todo_input_container = commands
        .spawn((NodeBundle::default(), markers::TodoInputContainer))
        .id();

    let todo_list = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    border: UiRect::top(Val::Px(1.0)),
                    ..default()
                },
                border_color: colors::main_border_top().into(),
                ..default()
            },
            markers::TodoList,
        ))
        .id();
    let todo_footer = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.),
                padding: UiRect::axes(Val::Px(15.0), Val::Px(10.0)),
                ..default()
            },
            ..default()
        })
        .id();
    // TODO: needs to be referenced
    let todo_items_left = commands
        .spawn((
            TextBundle::from_section("3 items left", text_styles::footer()),
            markers::TodoItemsLeftView,
        ))
        .id();

    let todo_filters = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .id();

    let filter_btn = |border_color: Color| ButtonBundle {
        border_color: border_color.into(),
        style: Style {
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::axes(Val::Px(7.0), Val::Px(3.0)),
            margin: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        ..default()
    };

    let todo_filter_all_btn = commands
        .spawn(filter_btn(colors::filters_li_a_selected()))
        .id();

    let todo_filter_all_txt = commands
        .spawn(TextBundle::from_section("All", text_styles::footer()))
        .id();

    let todo_filter_active_btn = commands.spawn(filter_btn(Color::NONE)).id();

    let todo_filter_active_txt = commands
        .spawn(TextBundle::from_section("Active", text_styles::footer()))
        .id();

    let todo_filter_completed_btn = commands.spawn(filter_btn(Color::NONE)).id();

    let todo_filter_completed_txt = commands
        .spawn(TextBundle::from_section("Completed", text_styles::footer()))
        .id();

    let todo_clear_completed_btn = commands.spawn(ButtonBundle::default()).id();

    let todo_clear_completed_txt = commands
        .spawn(TextBundle::from_section(
            "Clear completed",
            text_styles::footer(),
        ))
        .id();

    // app_main
    // - app_title
    // - todo_main
    //   - todo_input_container
    //   - todo_list
    //   - todo_footer
    //     - todo_items_left
    //     - todo_filters
    //     - todo_filter_all_btn
    //       - todo_filter_all_txt
    //     - todo_filter_active_btn
    //       - todo_filter_active_txt
    //     - todo_filter_completed_btn
    //       - todo_filter_completed_txt
    //     - todo_clear_completed_btn
    //       - todo_clear_completed_txt
    commands.entity(app_main).add_child(app_title);
    commands.entity(app_main).add_child(todo_main);
    commands.entity(todo_main).add_child(todo_input_container);
    commands.entity(todo_main).add_child(todo_list);
    commands.entity(todo_main).add_child(todo_footer);
    commands.entity(todo_footer).add_child(todo_items_left);
    commands.entity(todo_footer).add_child(todo_filters);
    commands.entity(todo_filters).add_child(todo_filter_all_btn);
    commands
        .entity(todo_filter_all_btn)
        .add_child(todo_filter_all_txt);
    commands
        .entity(todo_filters)
        .add_child(todo_filter_active_btn);
    commands
        .entity(todo_filter_active_btn)
        .add_child(todo_filter_active_txt);
    commands
        .entity(todo_filters)
        .add_child(todo_filter_completed_btn);
    commands
        .entity(todo_filter_completed_btn)
        .add_child(todo_filter_completed_txt);
    commands
        .entity(todo_footer)
        .add_child(todo_clear_completed_btn);
    commands
        .entity(todo_clear_completed_btn)
        .add_child(todo_clear_completed_txt);

    input_actions.send(ModelInputAction::Create("".to_string()));
}

/// Interaction -> Event<ModelTodoAction> + Event<SetFocus>
fn handle_deleter_interaction(
    mut delete_interaction_q: Query<
        (&Interaction, &View),
        (Changed<Interaction>, With<markers::TodoDeleterView>),
    >,
    mut actions: EventWriter<ModelTodoAction>,
    mut set_focus: EventWriter<SetFocus>,
) {
    for (interaction, view) in delete_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            actions.send(ModelTodoAction::Delete(view.0));
            set_focus.send(SetFocus(None));
        }
    }
}

/// Interaction -> Event<SetFocus>
fn handle_text_interaction(
    mut check_interaction_q: Query<
        (&Interaction, Entity),
        (Changed<Interaction>, With<markers::TodoTextView>),
    >,
    mut todo_text_q: Query<(Entity, &Parent), (With<Text>, With<markers::TodoTextView>)>,
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

/// Interaction -> Event<SetFocus>
fn handle_input_interaction(
    mut check_interaction_q: Query<
        (&Interaction, Entity),
        (Changed<Interaction>, With<markers::TodoInput>),
    >,
    todo_text_q: Query<(Entity, &Parent), (With<Text>, With<markers::TodoInput>)>,
    mut set_focus: EventWriter<SetFocus>,
) {
    for (interaction, clicked_entity) in check_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            for (entity, parent) in todo_text_q.iter() {
                if parent.get() == clicked_entity {
                    set_focus.send(SetFocus(Some(entity)));
                }
            }
        }
    }
}

/// Interaction -> Event<ModelTodoAction>
fn handle_checkmark_interaction(
    mut check_interaction_q: Query<
        (&Interaction, &View),
        (Changed<Interaction>, With<markers::TodoCheckmarkView>),
    >,
    model: Query<&ModelTodoChecked, ModelOnly>,
    mut actions: EventWriter<ModelTodoAction>,
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

/// Input<KeyCode> + Res<Focus> -> Event<ModelTodoAction>
///
/// But this system also directly updates the `Text` which it probably shouldn't (consider splitting)
fn handle_enter(
    mut actions: EventWriter<ModelTodoAction>,
    mut todo_input_q: Query<&mut Text, With<markers::TodoInput>>,
    keys: Res<Input<KeyCode>>,
    focus: Res<Focus>,
) {
    let Some(focus) = **focus else {
        return;
    };
    if keys.just_pressed(KeyCode::Return) {
        if let Ok(mut todo_input_text) = todo_input_q.get_mut(focus) {
            actions.send(ModelTodoAction::Create(
                todo_input_text.sections[0].value.clone(),
            ));
            todo_input_text.sections[0].value = "".to_string();
        }
    }
}

/// Event<SetFocus> -> Res<Focus>
fn handle_focus(mut set_focus_events: EventReader<SetFocus>, mut focus: ResMut<Focus>) {
    for ev in set_focus_events.iter() {
        *focus = Focus(ev.0)
    }
}

/// The question is how to express this at creation (setup_ui)
///
/// We want the equivalent of JS `input.addEventListener('oninput', (e) => { model.value = e.target.value })`
///
/// Event<ReceivedCharacter> -> Event<ModelInputAction> + Event<ModelTodoAction>
///
/// But this system also directly updates the `Text` which it probably shouldn't (consider splitting)
fn handle_typing(
    mut evr_char: EventReader<ReceivedCharacter>,
    focus: Res<Focus>,
    todo_text_q: Query<(&Text, &View), With<markers::TodoTextView>>,
    mut todo_input_q: Query<
        (&Text, &View),
        (With<markers::TodoInput>, Without<markers::TodoTextView>),
    >,
    mut todo_actions: EventWriter<ModelTodoAction>,
    mut input_actions: EventWriter<ModelInputAction>,
) {
    let Some(focus) = **focus else {
        return;
    };
    for ev in evr_char.iter() {
        if !ev.char.is_control() {
            if let Ok((text, view)) = todo_input_q.get_mut(focus) {
                input_actions.send(ModelInputAction::UpdateText(
                    view.0,
                    format!("{}{}", text.sections[0].value, ev.char),
                ));
            }

            if let Ok((text, view)) = todo_text_q.get(focus) {
                todo_actions.send(ModelTodoAction::UpdateText(
                    view.0,
                    format!("{}{}", text.sections[0].value, ev.char),
                ));
            }
        }
    }
}

/// Flush after this
///
/// Event<ModelTodoAction> -> Model
fn update_todo_model(
    mut actions: EventReader<ModelTodoAction>,
    mut commands: Commands,
    mut todo_text: Query<&mut ModelTodoText, ModelOnly>,
    mut todo_checked: Query<&mut ModelTodoChecked, ModelOnly>,
    mut todo_edit: Query<&mut ModelTodoEdit, ModelOnly>,
) {
    for action in actions.iter() {
        match action {
            ModelTodoAction::Create(text) => {
                commands.spawn((
                    ModelTodoText(text.clone()),
                    ModelTodoChecked(false),
                    ModelTodoEdit(false),
                    Model,
                ));
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
            ModelTodoAction::Edit(e, edit) => {
                todo_edit.get_mut(*e).unwrap().0 = *edit;
            }
        }
    }
}

/// Flush after this
///
/// Event<ModelInputAction> -> Model
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

/// Whenever a model (input) is created
/// display it by creating a view and appending it to the target parent view
///
/// ModelInputText -> View + Event<SetFocus>
fn display_text_input(
    inputs: Query<(ModelInputEntity, &ModelInputText), (Added<ModelInputText>, ModelOnly)>,
    todo_input_container: Query<Entity, With<markers::TodoInputContainer>>,
    mut commands: Commands,
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
                        min_width: Val::Percent(100.),
                        height: Val::Px(40.),
                        border: UiRect::all(Val::Px(4.)),
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                },
                View(model_entity),
                markers::TodoInput,
            ))
            .id();
        let todo_input_text = commands
            .spawn((
                TextBundle {
                    text: Text::from_section(input.0.clone(), text_styles::todo()),
                    ..default()
                },
                View(model_entity),
                markers::TodoInput,
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
fn display_checked(checked: &ModelTodoChecked) -> TextStyle {
    if checked.0 {
        text_styles::checkmark()
    } else {
        text_styles::checkmark_complete()
    }
}

/// Whenever a model (todo) is created,
/// display it by creating a view and appending it to the target parent view
///
/// ModelTodo{Text,Checked} -> View
fn display_todos(
    todos: Query<
        (ModelTodoEntity, &ModelTodoText, &ModelTodoChecked),
        (Added<ModelTodoText>, Added<ModelTodoChecked>, ModelOnly),
    >,
    todo_list_q: Query<Entity, With<markers::TodoList>>,
    mut commands: Commands,
) {
    // an outer reference
    let todo_list = todo_list_q.single();
    // some loop
    for (model_entity, todo, checked) in todos.iter() {
        // constructing a view
        let todo_item = commands
            .spawn((
                NodeBundle {
                    #[cfg(feature = "debug")]
                    background_color: Color::RED.into(),
                    style: Style {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(Val::Px(1.0)),
                        width: Val::Percent(100.),
                        ..default()
                    },
                    border_color: colors::todo_list_item_border_bottom().into(),
                    ..default()
                },
                View(model_entity),
                markers::TodoRootView,
            ))
            .id();
        let todo_check_btn = commands
            .spawn((
                ButtonBundle {
                    #[cfg(feature = "debug")]
                    background_color: Color::BLUE.into(),
                    style: Style {
                        width: Val::Px(40.),
                        height: Val::Px(40.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        overflow: Overflow::clip(),
                        padding: UiRect::axes(Val::Auto, Val::Px(15.)),
                        ..default()
                    },
                    ..default()
                },
                View(model_entity),
                markers::TodoCheckmarkView,
            ))
            .id();
        let todo_check_txt = commands
            .spawn((
                TextBundle {
                    #[cfg(feature = "debug")]
                    background_color: Color::FUCHSIA.into(),
                    text: Text::from_sections([
                        TextSection::new("[", text_styles::todo()),
                        TextSection::new("x", display_checked(checked)),
                        TextSection::new("]", text_styles::todo()),
                    ]),
                    ..default()
                },
                View(model_entity),
                markers::TodoCheckmarkView,
            ))
            .id();
        let todo_text_btn = commands
            .spawn((
                ButtonBundle {
                    #[cfg(feature = "debug")]
                    background_color: Color::GREEN.into(),
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Px(40.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                },
                View(model_entity),
                markers::TodoTextView,
            ))
            .id();
        let todo_text_txt = commands
            .spawn((
                TextBundle {
                    #[cfg(feature = "debug")]
                    background_color: Color::GOLD.into(),
                    style: Style {
                        flex_grow: 1.0,
                        ..default()
                    },
                    text: Text::from_section(&todo.0, text_styles::todo()),
                    ..default()
                },
                View(model_entity),
                markers::TodoTextView,
            ))
            .id();

        let todo_delete_btn = commands
            .spawn((
                ButtonBundle {
                    #[cfg(feature = "debug")]
                    background_color: Color::YELLOW.into(),
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
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
                    ..default()
                },
                View(model_entity),
                markers::TodoDeleterView,
            ))
            .id();
        let todo_delete_txt = commands
            .spawn((
                TextBundle {
                    #[cfg(feature = "debug")]
                    background_color: Color::TURQUOISE.into(),
                    text: Text::from_section("x", text_styles::destroy()),
                    ..default()
                },
                View(model_entity),
            ))
            .id();

        // assembling and inserting a view
        // todo_list
        // - [todo_item]
        //   - todo_check_btn
        //      - todo_check_txt
        //   - todo_text_btn
        //      - todo_text_txt
        //   - todo_delete_btn
        //      - todo_delete_txt
        commands.entity(todo_list).add_child(todo_item);
        commands.entity(todo_item).add_child(todo_check_btn);
        commands.entity(todo_check_btn).add_child(todo_check_txt);
        commands.entity(todo_item).add_child(todo_text_btn);
        commands.entity(todo_text_btn).add_child(todo_text_txt);
        commands.entity(todo_item).add_child(todo_delete_btn);
        commands.entity(todo_delete_btn).add_child(todo_delete_txt);
    }
}

/// Whenever a model (todo.text) is updated, views that depend on it are updated
///
/// ModelTodoText -> View
fn update_displayed_todos_text(
    todos_text: Query<&ModelTodoText, (Changed<ModelTodoText>, ModelOnly)>,
    mut views: Query<(&mut Text, &View), (With<markers::TodoTextView>, ViewOnly)>,
) {
    // outer loop, library-provided
    for (mut text, view) in views.iter_mut() {
        if let Ok(todo) = todos_text.get(view.0) {
            // inner logic, user-provided
            text.sections[0].value = todo.0.clone();
        }
    }
}

/// these updates are push-based but we kinda want pull-based from an authoring perspective
/// so that it's easier to locate a specific entity
/// this should be equivalent to...
/// We need to store a reference to a model entity on every dependent view entity
/// We also need to store a reference to a model entity on every view entity that sends an action which is not ideal
///
/// Whenever a model (todo.checked) is updated, views that depend on it are updated
///
/// ModelTodoChecked -> View
fn update_displayed_todos_text_checked(
    model_todo_checked: Query<&ModelTodoChecked, (Changed<ModelTodoChecked>, ModelOnly)>,
    mut views: Query<(&mut Text, &View), (ViewOnly, With<markers::TodoTextView>)>,
) {
    // outer loop, library-provided
    for (mut text, view) in views.iter_mut() {
        if let Ok(checked) = model_todo_checked.get(view.0) {
            // inner logic, user-provided
            text.sections[0].style.color = if checked.0 {
                colors::todo_list_item_completed_color()
            } else {
                colors::body_color()
            };
        }
    }
}

fn update_displayed_todos_checkmark_checked(
    model_todo_checked: Query<&ModelTodoChecked, (Changed<ModelTodoChecked>, ModelOnly)>,
    mut views: Query<(&mut Text, &View), (ViewOnly, With<markers::TodoCheckmarkView>)>,
) {
    // outer loop, library-provided
    for (mut text, view) in views.iter_mut() {
        if let Ok(checked) = model_todo_checked.get(view.0) {
            // inner logic, user-provided
            text.sections[1].style = display_checked(checked);
        }
    }
}

/// Whenever a model (input.text) is updated, views that depend on it are updated
///
/// ModelInputText -> View
fn update_displayed_input_text(
    model_input_text: Query<&ModelInputText, (Changed<ModelInputText>, ModelOnly)>,
    mut views: Query<(&mut Text, &View), (With<markers::TodoInput>, ViewOnly)>,
) {
    // outer loop, library-provided
    for (mut text, view) in views.iter_mut() {
        if let Ok(todo) = model_input_text.get(view.0) {
            // inner logic, user-provided
            text.sections[0].value = todo.0.clone();
        }
    }
}

/// Whenever a model is removed, views that depend on it are updated
///
/// Model -> View
fn remove_displayed_todos(
    mut removed: RemovedComponents<Model>,
    views: Query<(Entity, &View), (ViewOnly, With<markers::TodoRootView>)>,
    mut commands: Commands,
) {
    let models_to_views = views
        .iter()
        .map(|(entity, view)| (view.0, entity))
        .collect::<std::collections::HashMap<_, _>>();
    for model_entity in removed.iter() {
        if let Some(view_entity) = models_to_views.get(&model_entity) {
            commands.entity(*view_entity).despawn_recursive();
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
///
/// This currently also "tracks" the model entity
#[derive(Component)]
struct View(Entity);

/// This type alias has the effect of marking a `Model` and not a `View`
/// equivalent to `Marker::Model` for `enum Marker { Model, View }`
type ModelOnly = (With<Model>, Without<View>);

/// This type alias has the effect of marking a `View` and not a `Model`
/// equivalent to `Marker::View` for `enum Marker { Model, View }`
type ViewOnly = (Without<Model>, With<View>);

/// Probably unnecessary type alias, documents the intent
type ModelTodoEntity = Entity;

/// Probably unnecessary type alias, documents the intent
type ModelInputEntity = Entity;

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
/// Entities are a stand-in for references.
#[derive(Event)]
enum ModelTodoAction {
    Create(String),
    Delete(ModelTodoEntity),
    UpdateText(ModelTodoEntity, String),
    UpdateChecked(ModelTodoEntity, bool),
    Edit(ModelTodoEntity, bool),
}

/// See [`ModelTodoAction`].
#[derive(Component)]
struct ModelTodoText(String);

/// See [`ModelTodoAction`].
#[derive(Component)]
struct ModelTodoChecked(bool);

#[derive(Component)]
struct ModelTodoEdit(bool);

/// Combined with `ModelInputText`,
/// this is functionally equivalent to
/// ```rs
/// struct Input {
///     text: String,
/// }
///
/// impl Todos {
///     fn create(&mut self, text: String);
///     fn update_text(&mut self, idx: usize, text: String);
/// }
/// ```
///
/// Components are a stand-in for properties.
/// Events are a stand-in for methods.
/// Entities are a stand-in for references.
#[derive(Event)]
enum ModelInputAction {
    Create(String),
    UpdateText(ModelInputEntity, String),
}

/// See [`ModelInputAction`].
#[derive(Component)]
struct ModelInputText(String);

/// https://todomvc.com/examples/vanillajs/node_modules/todomvc-app-css/index.css
///
/// ```css
/// body {
///     background: #f5f5f5;
///     color: #4d4d4d;
/// }
/// .todoapp {
///     background: #fff;
///     box-shadow: 0 2px 4px 0 rgba(0, 0, 0, 0.2),
///                 0 25px 50px 0 rgba(0, 0, 0, 0.1);
/// }
/// .todoapp input::input-placeholder {
///     color: #e6e6e6;
/// }
/// .todoapp h1 {
///     color: rgba(175, 47, 47, 0.15);
/// }
/// .new-todo {
///     background: rgba(0, 0, 0, 0.003);
///     box-shadow: inset 0 -2px 1px rgba(0,0,0,0.03);
/// }
/// .main {
///     border-top: 1px solid #e6e6e6;
/// }
/// .toggle-all + label:before {
///     color: #e6e6e6;
/// }
/// .toggle-all:checked + label:before {
///     color: #737373;
/// }
/// .todo-list li {
///     border-bottom: 1px solid #ededed;
/// }
/// .todo-list li.completed label {
///     color: #d9d9d9;
/// }
/// .todo-list li .destroy {
///     color: #cc9a9a;
/// }
/// .todo-list li .destroy:hover {
///     color: #af5b5e;
/// }
/// .footer {
///     color: #777;
///     border-top: 1px solid #e6e6e6;
/// }
/// .footer:before {
///     box-shadow: 0 1px 1px rgba(0, 0, 0, 0.2),
///                 0 8px 0 -3px #f6f6f6,
///                 0 9px 1px -3px rgba(0, 0, 0, 0.2),
///                 0 16px 0 -6px #f6f6f6,
///                 0 17px 2px -6px rgba(0, 0, 0, 0.2);
/// }
/// .info {
///     color: #bfbfbf;
///     text-shadow: 0 1px 0 rgba(255, 255, 255, 0.5);
/// }
/// .filters li a:hover {
///     border-color: rgba(175, 47, 47, 0.1);
/// }
/// .filters li a.selected {
///     border-color: rgba(175, 47, 47, 0.2);
/// }
/// ```
mod colors {
    #![allow(unused)]
    use bevy::prelude::Color;

    pub fn body_background() -> Color {
        hex("#f5f5f5")
    }
    pub fn body_color() -> Color {
        hex("#4d4d4d")
    }
    pub fn todoapp_background() -> Color {
        hex("#fff")
    }
    pub fn todoapp_boxshadow_0() -> Color {
        rgba(0, 0, 0, 0.2)
    }
    pub fn todoapp_boxshadow_1() -> Color {
        rgba(0, 0, 0, 0.1)
    }
    pub fn todoapp_inputplaceholder_color() -> Color {
        hex("#e6e6e6")
    }
    pub fn todoapp_h1_color() -> Color {
        rgba(175, 47, 47, 0.15)
    }
    pub fn new_todo_background() -> Color {
        rgba(0, 0, 0, 0.003)
    }
    pub fn new_todo_boxshadow_0() -> Color {
        rgba(0, 0, 0, 0.03)
    }
    pub fn main_border_top() -> Color {
        hex("#e6e6e6")
    }
    pub fn toggle_all_checked() -> Color {
        hex("#737373")
    }
    pub fn toggle_all_checked_background() -> Color {
        hex("#e6e6e6")
    }
    pub fn todo_list_item_border_bottom() -> Color {
        hex("#ededed")
    }
    pub fn todo_list_item_completed_color() -> Color {
        hex("#d9d9d9")
    }
    pub fn todo_list_item_destroy_color() -> Color {
        hex("#cc9a9a")
    }
    pub fn todo_list_item_destroy_hover_color() -> Color {
        hex("#af5b5e")
    }
    pub fn footer_color() -> Color {
        hex("#777")
    }
    pub fn footer_bordertop() -> Color {
        hex("#e6e6e6")
    }
    pub fn footer_before_boxshadow_0() -> Color {
        rgba(0, 0, 0, 0.2)
    }
    pub fn footer_before_boxshadow_1() -> Color {
        hex("#f6f6f6")
    }
    pub fn info_color() -> Color {
        hex("#bfbfbf")
    }
    pub fn info_textshadow() -> Color {
        rgba(255, 255, 255, 0.5)
    }
    pub fn filters_li_a_hover() -> Color {
        rgba(175, 47, 47, 0.1)
    }
    pub fn filters_li_a_selected() -> Color {
        rgba(175, 47, 47, 0.2)
    }

    fn rgb(r: u8, g: u8, b: u8) -> Color {
        rgba(r, g, b, 1.0)
    }
    fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
        Color::rgba(r as f32 / 256.0, g as f32 / 256.0, b as f32 / 256.0, a)
    }
    fn hex(s: &str) -> Color {
        Color::hex(s).unwrap()
    }
}

mod text_styles {
    #![allow(unused)]

    use bevy::prelude::{default, Color, TextStyle};

    use crate::colors;

    pub fn footer() -> TextStyle {
        TextStyle {
            font_size: 14.0 * 1.2,
            color: colors::footer_color(),
            ..default()
        }
    }

    pub fn todo() -> TextStyle {
        TextStyle {
            font_size: 24.0 * 1.2,
            color: colors::body_color(),
            ..default()
        }
    }

    pub fn checkmark() -> TextStyle {
        TextStyle {
            font_size: 24.0 * 1.2,
            color: Color::LIME_GREEN,
            ..default()
        }
    }

    pub fn checkmark_complete() -> TextStyle {
        TextStyle {
            font_size: 24.0 * 1.2,
            color: Color::NONE,
            ..default()
        }
    }

    pub fn destroy() -> TextStyle {
        TextStyle {
            font_size: 24.0 * 1.2,
            color: colors::todo_list_item_destroy_color(),
            ..default()
        }
    }

    pub fn title() -> TextStyle {
        TextStyle {
            font_size: 100.0 * 1.2,
            color: colors::todoapp_h1_color(),
            ..default()
        }
    }
}

mod markers {
    use bevy::prelude::Component;

    #[derive(Component)]
    pub struct TodoInputContainer;

    #[derive(Component)]
    pub struct TodoList;

    #[derive(Component)]
    pub struct TodoInput;

    #[derive(Component)]
    pub struct TodoRootView;

    #[derive(Component)]
    pub struct TodoTextView;

    #[derive(Component)]
    pub struct TodoCheckmarkView;

    #[derive(Component)]
    pub struct TodoDeleterView;

    #[derive(Component)]
    pub struct TodoItemsLeftView;

    #[derive(Component)]
    pub struct TodoFilters;

    #[derive(Component)]
    pub struct TodoClearCompleted;

    #[derive(Component)]
    pub struct TodoFilterAll;

    #[derive(Component)]
    pub struct TodoFilterActive;

    #[derive(Component)]
    pub struct TodoFilterCompleted;
}
