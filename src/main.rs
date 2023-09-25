#![allow(clippy::type_complexity)]

use bevy::{
    prelude::*,
    window::{PresentMode, PrimaryWindow},
};
use bevy_cosmic_edit::*;
use tree_builder::EntityTreeExt;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevy • TodoMVC".into(),
                        present_mode: PresentMode::AutoVsync,
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_plugins(CosmicEditPlugin::default())
        .add_event::<ModelTodoAction>()
        .add_event::<ModelInputAction>()
        .add_event::<SetFocus>()
        .init_resource::<Focus>()
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_ui)
        .add_systems(PreUpdate, handle_deleter_interaction.before(handle_focus))
        .add_systems(PreUpdate, handle_checkmark_interaction.before(handle_focus))
        .add_systems(PreUpdate, handle_text_interaction.before(handle_focus))
        .add_systems(PreUpdate, handle_input_interaction.before(handle_focus))
        .add_systems(PreUpdate, handle_enter.before(handle_focus))
        .add_systems(PreUpdate, handle_cosmic_change.before(handle_focus))
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
        .add_systems(
            Update,
            update_focus_main_input
                .after(update_input_model)
                .after(update_todo_model),
        )
        .add_systems(
            Update,
            update_focus_todo
                .after(update_input_model)
                .after(update_todo_model),
        )
        .add_systems(PostUpdate, remove_displayed_todos)
        .run();
}

#[derive(Event)]
struct SetFocus(Option<Entity>);

fn setup(mut commands: Commands) {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
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
        .spawn((
            NodeBundle {
                style: Style {
                    justify_content: JustifyContent::Start,
                    overflow: Overflow::clip(),
                    align_items: AlignItems::Start,
                    margin: UiRect::all(Val::Px(10.)),
                    min_width: Val::Px(500.),
                    height: Val::Px(40.),
                    ..default()
                },
                ..default()
            },
            markers::TodoInputContainer,
        ))
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

    app_main
        .tree((
            app_title,
            todo_main.tree((
                todo_input_container,
                todo_list,
                todo_footer.tree((
                    todo_items_left,
                    todo_filters,
                    todo_filter_all_btn.tree(todo_filter_all_txt),
                    todo_filter_active_btn.tree(todo_filter_active_txt),
                    todo_filter_completed_btn.tree(todo_filter_completed_txt),
                    todo_clear_completed_btn.tree(todo_clear_completed_txt),
                )),
            )),
        ))
        .build(&mut commands);

    input_actions.send(ModelInputAction::Create("".to_string()));
}

/// Interaction -> Event<ModelTodoAction>
fn handle_deleter_interaction(
    mut delete_interaction_q: Query<
        (&Interaction, &View),
        (Changed<Interaction>, With<markers::TodoDeleterView>),
    >,
    mut actions: EventWriter<ModelTodoAction>,
) {
    for (interaction, view) in delete_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            actions.send(ModelTodoAction::Delete(view.0));
        }
    }
}

/// Interaction -> Event<ModelTodoAction> +  Event<ModelInputAction>
fn handle_text_interaction(
    mut check_interaction_q: Query<
        (&Interaction, &View),
        (Changed<Interaction>, With<markers::TodoTextView>),
    >,
    todo_model: Query<(&ModelTodoEdit, Entity), ModelOnly>,
    input_model: Query<(&ModelInputEdit, Entity), ModelOnly>,
    mut todo_actions: EventWriter<ModelTodoAction>,
    mut input_actions: EventWriter<ModelInputAction>,
) {
    for (interaction, view) in check_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            for (todo_edit, todo_entity) in todo_model.iter() {
                if todo_edit.0 {
                    todo_actions.send(ModelTodoAction::Edit(todo_entity, false));
                }
            }
            for (input_edit, todo_entity) in input_model.iter() {
                if input_edit.0 {
                    input_actions.send(ModelInputAction::Edit(todo_entity, false));
                }
            }
            todo_actions.send(ModelTodoAction::Edit(view.0, true));
        }
    }
}

/// Interaction -> Event<ModelTodoAction> +  Event<ModelInputAction>
fn handle_input_interaction(
    mut check_interaction_q: Query<
        (&Interaction, &View),
        (Changed<Interaction>, With<markers::TodoInput>),
    >,
    todo_model: Query<(&ModelTodoEdit, Entity), ModelOnly>,
    input_model: Query<(&ModelInputEdit, Entity), ModelOnly>,
    mut todo_actions: EventWriter<ModelTodoAction>,
    mut input_actions: EventWriter<ModelInputAction>,
) {
    for (interaction, view) in check_interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            for (todo_edit, todo_entity) in todo_model.iter() {
                if todo_edit.0 {
                    todo_actions.send(ModelTodoAction::Edit(todo_entity, false));
                }
            }
            for (input_edit, input_entity) in input_model.iter() {
                if input_edit.0 {
                    input_actions.send(ModelInputAction::Edit(input_entity, false));
                }
            }
            input_actions.send(ModelInputAction::Edit(view.0, true));
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

/// Input<KeyCode> + Res<Focus> -> Event<ModelTodoAction> + Event<ModelInputAction>
///
/// But this system also directly updates the `Text` which it probably shouldn't (consider splitting)
fn handle_enter(
    keys: Res<Input<KeyCode>>,
    focus: Res<Focus>,
    mut todo_actions: EventWriter<ModelTodoAction>,
    mut input_actions: EventWriter<ModelInputAction>,
    mut todo_input_q: Query<(&CosmicEditor, &View), With<markers::TodoInput>>,
) {
    let Some(focus) = **focus else {
        return;
    };
    if keys.just_pressed(KeyCode::Return) {
        if let Ok((editor, view)) = todo_input_q.get_mut(focus) {
            todo_actions.send(ModelTodoAction::Create(editor.get_text()));
            input_actions.send(ModelInputAction::UpdateText(view.0, "".to_string()));
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
/// Event<CosmicTextChanged> -> Event<ModelInputAction> + Event<ModelTodoAction>
fn handle_cosmic_change(
    mut evr_cosmic: EventReader<CosmicTextChanged>,
    todo_text_q: Query<&View, With<markers::TodoTextView>>,
    mut todo_input_q: Query<&View, (With<markers::TodoInput>, Without<markers::TodoTextView>)>,
    mut todo_actions: EventWriter<ModelTodoAction>,
    mut input_actions: EventWriter<ModelInputAction>,
) {
    for ev in evr_cosmic.iter() {
        if let Ok(view) = todo_text_q.get(ev.0 .0) {
            todo_actions.send(ModelTodoAction::UpdateText(view.0, ev.0 .1.clone()));
        }
        if let Ok(view) = todo_input_q.get_mut(ev.0 .0) {
            input_actions.send(ModelInputAction::UpdateText(view.0, ev.0 .1.clone()));
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
    mut input_edit: Query<&mut ModelInputEdit, ModelOnly>,
) {
    for action in actions.iter() {
        match action {
            ModelInputAction::Create(text) => {
                commands.spawn((ModelInputText(text.clone()), ModelInputEdit(true), Model));
            }
            ModelInputAction::UpdateText(e, text) => {
                input_text.get_mut(*e).unwrap().0 = text.clone();
            }
            ModelInputAction::Edit(e, edit) => {
                input_edit.get_mut(*e).unwrap().0 = *edit;
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
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let todo_input_container = todo_input_container.single();
    for (model_entity, input) in inputs.iter() {
        let primary_window = windows.single();
        let text_color = text_styles::todo().color;
        let attrs = AttrsOwned::new(Attrs::new().color(bevy_color_to_cosmic(text_color)));
        let todo_input_btn = commands
            .spawn((
                CosmicEditUiBundle {
                    background_color: Color::WHITE.into(),
                    #[cfg(feature = "debug")]
                    border_color: Color::GREEN.into(),
                    style: Style {
                        height: Val::Px(40.),
                        padding: UiRect::all(Val::Px(10.)),
                        width: Val::Percent(100.),
                        border: UiRect::all(Val::Px(4.)),
                        ..default()
                    },
                    cosmic_attrs: CosmicAttrs(attrs.clone()),
                    cosmic_metrics: CosmicMetrics {
                        font_size: text_styles::todo().font_size,
                        line_height: text_styles::todo().font_size * 1.2,
                        scale_factor: primary_window.scale_factor() as f32,
                    },
                    max_lines: CosmicMaxLines(1),
                    max_chars: CosmicMaxChars(25), // TODO: consider removing after https://github.com/StaffEngineer/bevy_cosmic_edit/issues/48
                    text: CosmicText::OneStyle(input.0.clone()),
                    text_position: CosmicTextPosition::Center, // TODO: implement CenterLeft https://github.com/StaffEngineer/bevy_cosmic_edit/issues/51
                    ..default()
                },
                View(model_entity),
                markers::TodoInput,
            ))
            .id();

        todo_input_container
            .tree(todo_input_btn)
            .build(&mut commands);
        set_focus.send(SetFocus(Some(todo_input_btn)));
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
    windows: Query<&Window, With<PrimaryWindow>>,
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

        let text_color = text_styles::todo().color;
        let attrs = AttrsOwned::new(Attrs::new().color(bevy_color_to_cosmic(text_color)));
        let primary_window = windows.single();
        let todo_text_btn = commands
            .spawn((
                CosmicEditUiBundle {
                    background_color: Color::WHITE.into(),
                    style: Style {
                        border: UiRect::all(Val::Px(2.)),
                        width: Val::Percent(100.),
                        height: Val::Px(40.),
                        ..default()
                    },
                    cosmic_attrs: CosmicAttrs(attrs.clone()),
                    cosmic_metrics: CosmicMetrics {
                        font_size: text_styles::todo().font_size,
                        line_height: text_styles::todo().font_size * 1.2,
                        scale_factor: primary_window.scale_factor() as f32,
                    },
                    max_lines: CosmicMaxLines(1),
                    max_chars: CosmicMaxChars(25), // TODO: consider removing after https://github.com/StaffEngineer/bevy_cosmic_edit/issues/48
                    text: CosmicText::OneStyle(todo.0.clone()),
                    text_position: CosmicTextPosition::Center, // TODO: implement CenterLeft https://github.com/StaffEngineer/bevy_cosmic_edit/issues/51
                    ..default()
                },
                View(model_entity),
                markers::TodoTextView,
                ReadOnly,
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
                ReadOnly,
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

        todo_list
            .tree(todo_item.tree((
                todo_check_btn.tree(todo_check_txt),
                todo_text_btn,
                todo_delete_btn.tree(todo_delete_txt),
            )))
            .build(&mut commands);
    }
}

/// Whenever a model (todo.text) is updated, views that depend on it are updated
///
/// ModelTodoText -> View
fn update_displayed_todos_text(
    todos_text: Query<&ModelTodoText, (Changed<ModelTodoText>, ModelOnly)>,
    mut views: Query<(&mut CosmicText, &View), (With<markers::TodoTextView>, ViewOnly)>,
) {
    // outer loop, library-provided
    for (mut text, view) in views.iter_mut() {
        if let Ok(todo) = todos_text.get(view.0) {
            // inner logic, user-provided
            *text = CosmicText::OneStyle(todo.0.clone());
        }
    }
}

pub fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> CosmicColor {
    CosmicColor::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
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
    model_todo_checked: Query<
        (&ModelTodoChecked, &ModelTodoText),
        (Changed<ModelTodoChecked>, ModelOnly),
    >,
    mut views: Query<
        (&mut CosmicAttrs, &mut CosmicText, &View),
        (ViewOnly, With<markers::TodoTextView>),
    >,
) {
    // outer loop, library-provided
    for (mut attrs, mut cosmic_text, view) in views.iter_mut() {
        if let Ok((checked, text)) = model_todo_checked.get(view.0) {
            // inner logic, user-provided
            attrs.0.color_opt = if checked.0 {
                Some(bevy_color_to_cosmic(
                    colors::todo_list_item_completed_color(),
                ))
            } else {
                Some(bevy_color_to_cosmic(colors::body_color()))
            };
            // TODO: Remove this hack. This is done for updating colors immediately. Figure out why set_redraw to true doesn't work in this case.
            *cosmic_text = CosmicText::OneStyle(text.0.clone());
        }
    }
}

// ModelTodoChecked -> View
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
    mut views: Query<(&mut CosmicText, &View), (With<markers::TodoInput>, ViewOnly)>,
) {
    // outer loop, library-provided
    for (mut text, view) in views.iter_mut() {
        if let Ok(todo) = model_input_text.get(view.0) {
            // inner logic, user-provided
            *text = CosmicText::OneStyle(todo.0.clone());
        }
    }
}

// ModelInputEdit -> View + Event<SetFocus>
fn update_focus_main_input(
    model_input_edit: Query<(&ModelInputEdit, Entity), (Changed<ModelInputEdit>, ModelOnly)>,
    views: Query<(Entity, &View), (ViewOnly, With<markers::TodoInput>)>,
    mut set_focus: EventWriter<SetFocus>,
    mut commands: Commands,
) {
    let models_to_views = views
        .iter()
        .map(|(entity, view)| (view.0, entity))
        .collect::<std::collections::HashMap<_, _>>();
    for (edit, model_entity) in model_input_edit.iter() {
        if let Some(view_entity) = models_to_views.get(&model_entity) {
            if edit.0 {
                commands.entity(*view_entity).remove::<ReadOnly>();
            } else {
                commands.entity(*view_entity).insert(ReadOnly);
            }
            set_focus.send(SetFocus(Some(*view_entity)));
        }
    }
}

// ModelTodoEdit -> View + Event<SetFocus>
fn update_focus_todo(
    model_todo_edit: Query<(&ModelTodoEdit, Entity), (Changed<ModelTodoEdit>, ModelOnly)>,
    views: Query<(Entity, &View), (ViewOnly, With<markers::TodoTextView>)>,
    mut set_focus: EventWriter<SetFocus>,
    mut commands: Commands,
) {
    let models_to_views = views
        .iter()
        .map(|(entity, view)| (view.0, entity))
        .collect::<std::collections::HashMap<_, _>>();
    for (edit, model_entity) in model_todo_edit.iter() {
        if let Some(view_entity) = models_to_views.get(&model_entity) {
            if edit.0 {
                commands.entity(*view_entity).remove::<ReadOnly>();
            } else {
                commands.entity(*view_entity).insert(ReadOnly);
            }
            set_focus.send(SetFocus(Some(*view_entity)));
        }
    }
}

/// Whenever a model is removed, views that depend on it are updated
///
/// Model -> View + Event<SetFocus>
fn remove_displayed_todos(
    mut removed: RemovedComponents<Model>,
    views: Query<(Entity, &View), (ViewOnly, With<markers::TodoRootView>)>,
    mut commands: Commands,
    mut set_focus: EventWriter<SetFocus>,
) {
    let models_to_views = views
        .iter()
        .map(|(entity, view)| (view.0, entity))
        .collect::<std::collections::HashMap<_, _>>();
    for model_entity in removed.iter() {
        if let Some(view_entity) = models_to_views.get(&model_entity) {
            commands.entity(*view_entity).despawn_recursive();
            set_focus.send(SetFocus(None));
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
#[derive(Event, Debug)]
enum ModelInputAction {
    Create(String),
    UpdateText(ModelInputEntity, String),
    Edit(ModelInputEntity, bool),
}

/// See [`ModelInputAction`].
#[derive(Component)]
struct ModelInputText(String);

#[derive(Component)]
struct ModelInputEdit(bool);

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
            font_size: 24.0,
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

mod tree_builder {
    /// TODO: Figure out how to make the iterators IntoTreeIterator
    /// TODO: Or better, figure out how to make the iterators IntoTree
    /// TODO: Simplify the implementation (less re-implementation, call methods and functions instead)
    /// TODO: #[inline]
    use bevy::prelude::{BuildChildren, Commands, Entity};

    // dead code
    fn _x<T>(root: Entity, branches: impl IntoIterator<Item = T>) -> Tree
    where
        T: IntoTree,
    {
        // take the current tree and create a new one
        Tree::new(root, branches)
    }

    // dead code
    fn _l(leaf: Entity) -> Tree {
        Tree::new_leaf(leaf)
    }

    // dead code
    fn _c<T, S, I>(children: T) -> TreeIterator<I>
    where
        T: IntoTreeIterator<IterableStorage = S>,
        S: IntoIterator<IntoIter = I>,
        I: Iterator<Item = Tree>,
    {
        children.into_tree_iter()
    }

    // dead code
    /// Convert an `IntoIterator<Item = Tree>`s to a `TreeIterator`
    ///
    /// Effectively shorthand for `TreeIterator::new()`
    pub fn _col<S, I>(children: S) -> TreeIterator<I>
    where
        S: IntoIterator<IntoIter = I>,
        I: Iterator<Item = Tree>,
    {
        TreeIterator::new(children)
    }

    /// Construct a [`Tree`] of entities
    fn build_tree<T, S, I>(root: Entity, children: T) -> Tree
    where
        // T is the thing that becomes an iterator over `Tree`s, e.g. `(Entity, Entity)`
        T: IntoTreeIterator<IterableStorage = S>,
        // S is the storage for the iterator, which becomes an iterator over `Tree`s, e.g. `[Entity, Entity]`
        S: IntoIterator<IntoIter = I>,
        // I is the iterator over `Tree`s
        I: Iterator<Item = Tree>,
    {
        // take the current tree and create a new one
        let branches = children.into_tree_iter();
        Tree::new(root, branches)
    }

    pub trait IteratorAdapter {
        type IntoIter;
        type Iterator;
        type Item;
        fn c(self) -> TreeIterator<Self::Iterator>
        where
            Self::IntoIter: IntoIterator<Item = Self::Item>,
            Self::Iterator: Iterator<Item = Tree>,
            Self::Item: IntoTree;
    }

    impl<S> IteratorAdapter for S
    where
        S: IntoIterator,
        S::Item: IntoTree,
    {
        type IntoIter = S;
        type Iterator = S::IntoIter;
        type Item = Tree;
        /// Construct a `TreeIterator` from an `Iterator` of anything that can be converted into a tree
        fn c(self) -> TreeIterator<Self::Iterator>
        where
            Self::IntoIter: IntoIterator<Item = Self::Item>,
            Self::Iterator: Iterator<Item = Tree>,
            Self::Item: IntoTree,
        {
            TreeIterator::new(self)
        }
    }

    /// A type that simply stores the id of the root entity,
    /// and the id pairs of all branches and their sub-branches, recursively
    #[derive(Debug)]
    pub struct Tree {
        pub id: Entity,
        pub links: Vec<(Entity, Entity)>,
    }

    impl Tree {
        fn new_leaf(root: Entity) -> Self {
            Self {
                id: root,
                links: Vec::new(),
            }
        }

        fn new<T>(root: Entity, branches: impl IntoIterator<Item = T>) -> Tree
        where
            T: IntoTree,
        {
            // take the current entity as the root and create branches
            let branches = branches.into_iter().map(|t| t.into_tree());
            let mut this_tree = Self::new_leaf(root);
            for child_tree in branches {
                this_tree.links.push((this_tree.id, child_tree.id));
                this_tree.links.extend(child_tree.links);
            }
            this_tree
        }

        pub fn build(self, commands: &mut Commands) {
            for (parent, child) in self.links {
                commands.entity(parent).add_child(child);
            }
        }
    }

    /// A type that stores an iterator over trees
    pub struct TreeIterator<I>
    where
        I: Iterator<Item = Tree>,
    {
        iter: I,
    }

    impl<I> TreeIterator<I>
    where
        I: Iterator<Item = Tree>,
    {
        fn new<S>(iterable: S) -> Self
        where
            S: IntoIterator<IntoIter = I>,
        {
            let iter = iterable.into_iter();
            Self { iter }
        }
    }

    impl<S, I> From<S> for TreeIterator<I>
    where
        S: IntoIterator<IntoIter = I>,
        I: Iterator<Item = Tree>,
    {
        fn from(iterable: S) -> Self {
            Self::new(iterable)
        }
    }
    trait Identity {
        type This;
        fn identity(self) -> Self::This;
    }

    impl<A> Identity for A {
        type This = A;
        fn identity(self) -> Self::This {
            self
        }
    }

    impl<I> Iterator for TreeIterator<I>
    where
        I: Iterator<Item = Tree>,
    {
        type Item = Tree;

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next()
        }
    }
    pub trait EntityTreeExt {
        fn tree<T, S, I>(self, children: T) -> Tree
        where
            // T is the thing that becomes an iterator over `Tree`s, e.g. `(Entity, Entity)`
            T: IntoTreeIterator<IterableStorage = S>,
            // S is the iterable storage for the iterator, which becomes an iterator over `Tree`s, e.g. `[Entity, Entity]`
            S: IntoIterator<IntoIter = I>,
            // I is the iterator over `Tree`s
            I: Iterator<Item = Tree>;

        fn l(self) -> Tree;
    }

    impl EntityTreeExt for Entity {
        /// Construct a [`Tree`] of entities
        ///
        /// # Example
        ///
        /// ```rs
        /// # use bevy::prelude::*;
        /// # let world = World::new();
        /// # let mut queue = bevy::ecs::system::CommandQueue::default();
        /// # let mut commands = Commands::new(&mut queue, &world);
        /// # let app_main = Entity::PLACEHOLDER;
        /// # let app_title = Entity::PLACEHOLDER;
        /// # let todo_main = Entity::PLACEHOLDER;
        /// # let todo_input_container = Entity::PLACEHOLDER;
        /// # let todo_list = Entity::PLACEHOLDER;
        /// # let todo_footer = Entity::PLACEHOLDER;
        /// # let todo_items_left = Entity::PLACEHOLDER;
        /// # let todo_filters = Entity::PLACEHOLDER;
        /// # let todo_filter_all_btn = Entity::PLACEHOLDER;
        /// # let todo_filter_all_txt = Entity::PLACEHOLDER;
        /// # let todo_filter_active_btn = Entity::PLACEHOLDER;
        /// # let todo_filter_active_txt = Entity::PLACEHOLDER;
        /// # let todo_filter_completed_btn = Entity::PLACEHOLDER;
        /// # let todo_filter_completed_txt = Entity::PLACEHOLDER;
        /// # let todo_clear_completed_btn = Entity::PLACEHOLDER;
        /// # let todo_clear_completed_txt = Entity::PLACEHOLDER;
        /// // app_main
        /// // - app_title
        /// // - todo_main
        /// //   - todo_input_container
        /// //   - todo_list
        /// //   - todo_footer
        /// //     - todo_items_left
        /// //     - todo_filters
        /// //     - todo_filter_all_btn
        /// //       - todo_filter_all_txt
        /// //     - todo_filter_active_btn
        /// //       - todo_filter_active_txt
        /// //     - todo_filter_completed_btn
        /// //       - todo_filter_completed_txt
        /// //     - todo_clear_completed_btn
        /// //       - todo_clear_completed_txt
        /// app_main.t((
        ///     app_title,
        ///     todo_main.t((
        ///         todo_input_container,
        ///         todo_list,
        ///         todo_footer.t((
        ///             todo_items_left,
        ///             todo_filters,
        ///             todo_filter_all_btn.t(todo_filter_all_txt),
        ///             todo_filter_active_btn.t(todo_filter_active_txt),
        ///             todo_filter_completed_btn.t(todo_filter_completed_txt),
        ///             todo_clear_completed_btn.t(todo_clear_completed_txt),
        ///         )),
        ///     )),
        /// ))
        /// .build(&mut commands);
        /// ```
        fn tree<T, S, I>(self, children: T) -> Tree
        where
            T: IntoTreeIterator<IterableStorage = S>,
            S: IntoIterator<IntoIter = I>,
            I: Iterator<Item = Tree>,
        {
            build_tree(self, children)
        }

        fn l(self) -> Tree {
            Tree::new_leaf(self)
        }
    }

    pub trait IntoTree {
        fn into_tree(self) -> Tree;
    }

    impl IntoTree for Tree {
        fn into_tree(self) -> Tree {
            self
        }
    }

    impl IntoTree for Entity {
        fn into_tree(self) -> Tree {
            Tree::new_leaf(self)
        }
    }

    pub trait IntoTreeIterator {
        /// A storage type that can be converted into an iterator over `Tree`s
        type IterableStorage;
        fn into_tree_iter<I>(self) -> TreeIterator<I>
        where
            Self::IterableStorage: IntoIterator<IntoIter = I>,
            I: Iterator<Item = Tree>;
    }

    impl<S: Iterator<Item = Tree>> IntoTreeIterator for TreeIterator<S> {
        type IterableStorage = Self;
        fn into_tree_iter<I>(self) -> TreeIterator<I>
        where
            Self::IterableStorage: IntoIterator<IntoIter = I>,
            I: Iterator<Item = Tree>,
        {
            self.into()
        }
    }

    impl IntoTreeIterator for Entity {
        type IterableStorage = [Tree; 1];
        fn into_tree_iter<I>(self) -> TreeIterator<I>
        where
            Self::IterableStorage: IntoIterator<IntoIter = I>,
            I: Iterator<Item = Tree>,
        {
            TreeIterator::new([self.into_tree()])
        }
    }

    impl IntoTreeIterator for Tree {
        type IterableStorage = [Tree; 1];
        fn into_tree_iter<I>(self) -> TreeIterator<I>
        where
            Self::IterableStorage: IntoIterator<IntoIter = I>,
            I: Iterator<Item = Tree>,
        {
            TreeIterator::new([self])
        }
    }

    // impl_intotreeiter!((T0, t0), (T1, t1));
    macro_rules! impl_intotreeiter {
        ($(($T:ident, $t:ident)),*) => {
            // impl <T0, T1> IntoTreeIterator for (T0, T1) where T0: IntoTree, T1: IntoTree {
            impl<$($T),*> IntoTreeIterator for ($($T,)*) where $($T: IntoTree),* {
                // type IterableStorage = [Tree; 0usize + 1usize + 1usize];
                type IterableStorage = [Tree; 0usize $(+ replace_expr!($T 1usize))*];
                fn into_tree_iter<I>(self) -> TreeIterator<I>
                where
                    Self::IterableStorage: IntoIterator<IntoIter = I>,
                    I: Iterator<Item = Tree>,
                {
                    // let (t0, t1) = self;
                    let ($($t,)*) = self;
                    // let source: Self::IterableStorage = [t0.into_tree(), t1.into_tree()];
                    let source: Self::IterableStorage = [$($t.into_tree()),*];
                    TreeIterator::new(source)
                }
            }
        }
    }

    /// Discards the actual tokentree provided (tt), and replaces it with the expression `sub`
    ///
    /// See https://veykril.github.io/tlborm/decl-macros/patterns/repetition-replacement.html?highlight=replace_expr#repetition-replacement
    macro_rules! replace_expr {
        ($_t:tt $sub:expr) => {
            $sub
        };
    }

    bevy::utils::all_tuples!(impl_intotreeiter, 0, 16, T, t);
}
