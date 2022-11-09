use bevy::{app::AppExit, prelude::*};

use crate::{despawn_kind, AppState, SCALE};

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Debug, Component, PartialEq, Eq, Clone, Copy)]
struct Volume(u32);

// This plugin manages the menu, with 5 different screens:
// - a main menu with "New Game", "Settings", "Quit"
// - a settings menu with two submenus and a back button
// - two settings screen with a setting that can be set and a back button
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // At start, the menu is not enabled. This will be changed in `menu_setup` when
            // entering the `AppState::Menu` state.
            // Current screen in the menu is handled by an independent state from `AppState`
            .add_state(MenuState::Disabled)
            .insert_resource(Volume(7))
            .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(menu_setup))
            // Systems to handle the main menu screen
            .add_system_set(SystemSet::on_enter(MenuState::Main).with_system(main_menu_setup))
            .add_system_set(
                SystemSet::on_exit(MenuState::Main).with_system(despawn_kind::<OnMainMenuScreen>),
            )
            // Systems to handle the settings menu screen
            .add_system_set(
                SystemSet::on_enter(MenuState::Settings).with_system(settings_menu_setup),
            )
            .add_system_set(
                SystemSet::on_exit(MenuState::Settings)
                    .with_system(despawn_kind::<OnSettingsMenuScreen>),
            )
            // Systems to handle the sound settings screen
            .add_system_set(
                SystemSet::on_enter(MenuState::SettingsSound)
                    .with_system(sound_settings_menu_setup),
            )
            .add_system_set(
                SystemSet::on_update(MenuState::SettingsSound)
                    .with_system(setting_button::<Volume>),
            )
            .add_system_set(
                SystemSet::on_exit(MenuState::SettingsSound)
                    .with_system(despawn_kind::<OnSoundSettingsMenuScreen>),
            )
            // Common systems to all screens that handles buttons behaviour
            .add_system_set(
                SystemSet::on_update(AppState::Menu)
                    .with_system(menu_action)
                    .with_system(button_system),
            );
    }
}

// State used for the current menu screen
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MenuState {
    Main,
    Settings,
    SettingsSound,
    Disabled,
}

// Tag component used to tag entities added on the main menu screen
#[derive(Component)]
struct OnMainMenuScreen;

// Tag component used to tag entities added on the settings menu screen
#[derive(Component)]
struct OnSettingsMenuScreen;

// Tag component used to tag entities added on the sound settings menu screen
#[derive(Component)]
struct OnSoundSettingsMenuScreen;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// Tag component used to mark wich setting is currently selected
#[derive(Component)]
struct SelectedOption;

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    Play,
    Settings,
    SettingsSound,
    BackToMainMenu,
    BackToSettings,
    Quit,
}

// This system handles changing all buttons color based on mouse interaction
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, selected) in &mut interaction_query {
        *color = match (*interaction, selected) {
            (Interaction::Clicked, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

// This system updates the settings when a new value for a setting is selected, and marks
// the button as the one currently selected
fn setting_button<T: Component + PartialEq + Copy>(
    interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
    mut selected_query: Query<(Entity, &mut UiColor), With<SelectedOption>>,
    mut commands: Commands,
    mut setting: ResMut<T>,
) {
    for (interaction, button_setting, entity) in &interaction_query {
        if *interaction == Interaction::Clicked && *setting != *button_setting {
            let (previous_button, mut previous_color) = selected_query.single_mut();
            *previous_color = NORMAL_BUTTON.into();
            commands.entity(previous_button).remove::<SelectedOption>();
            commands.entity(entity).insert(SelectedOption);
            *setting = *button_setting;
        }
    }
}

fn menu_setup(mut menu_state: ResMut<State<MenuState>>) {
    let _ = menu_state.set(MenuState::Main);
}

fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(crate::FONT);
    // Common style for all buttons on the screen
    let button_style = Style {
        size: Size::new(Val::Px(250.0 / SCALE), Val::Px(65.0 / SCALE)),
        margin: UiRect::all(Val::Px(20.0 / SCALE)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_icon_style = Style {
        size: Size::new(Val::Px(30.0 / SCALE), Val::Auto),
        // This takes the icons out of the flexbox flow, to be positioned exactly
        position_type: PositionType::Absolute,
        // The icon will be close to the left border of the button
        position: UiRect {
            left: Val::Px(10.0 / SCALE),
            right: Val::Auto,
            top: Val::Auto,
            bottom: Val::Auto,
        },
        ..default()
    };
    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0 / SCALE,
        color: TEXT_COLOR,
    };

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::CRIMSON.into(),
            ..default()
        })
        .insert(OnMainMenuScreen)
        .with_children(|parent| {
            // Display the game name
            parent.spawn_bundle(
                TextBundle::from_section(
                    "Bevy Game Menu UI",
                    TextStyle {
                        font: font.clone(),
                        font_size: 80.0 / SCALE,
                        color: TEXT_COLOR,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.0 / SCALE)),
                    ..default()
                }),
            );

            // Display three buttons for each action available from the main menu:
            // - new game
            // - settings
            // - quit
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(MenuButtonAction::Play)
                .with_children(|parent| {
                    // let icon = asset_server.load("textures/Game Icons/right.png");
                    // parent.spawn_bundle(ImageBundle {
                    //     style: button_icon_style.clone(),
                    //     image: UiImage(icon),
                    //     ..default()
                    // });
                    parent.spawn_bundle(TextBundle::from_section(
                        "Resume",
                        button_text_style.clone(),
                    ));
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(MenuButtonAction::Settings)
                .with_children(|parent| {
                    // let icon = asset_server.load("textures/Game Icons/wrench.png");
                    // parent.spawn_bundle(ImageBundle {
                    //     style: button_icon_style.clone(),
                    //     image: UiImage(icon),
                    //     ..default()
                    // });
                    parent.spawn_bundle(TextBundle::from_section(
                        "Settings",
                        button_text_style.clone(),
                    ));
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style,
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(MenuButtonAction::Quit)
                .with_children(|parent| {
                    // let icon = asset_server.load("textures/Game Icons/exitRight.png");
                    // parent.spawn_bundle(ImageBundle {
                    //     style: button_icon_style,
                    //     image: UiImage(icon),
                    //     ..default()
                    // });
                    parent.spawn_bundle(TextBundle::from_section("Quit", button_text_style));
                });
        });
}

fn settings_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_style = Style {
        size: Size::new(Val::Px(200.0 / SCALE), Val::Px(65.0 / SCALE)),
        margin: UiRect::all(Val::Px(20.0 / SCALE)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextStyle {
        font: asset_server.load(crate::FONT),
        font_size: 40.0 / SCALE,
        color: TEXT_COLOR,
    };

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::CRIMSON.into(),
            ..default()
        })
        .insert(OnSettingsMenuScreen)
        .with_children(|parent| {
            for (action, text) in [
                (MenuButtonAction::SettingsSound, "Sound"),
                (MenuButtonAction::BackToMainMenu, "Back"),
            ] {
                parent
                    .spawn_bundle(ButtonBundle {
                        style: button_style.clone(),
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    })
                    .insert(action)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            text,
                            button_text_style.clone(),
                        ));
                    });
            }
        });
}

fn sound_settings_menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    volume: Res<Volume>,
) {
    let button_style = Style {
        size: Size::new(Val::Px(200.0 / SCALE), Val::Px(65.0 / SCALE)),
        margin: UiRect::all(Val::Px(20.0 / SCALE)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font: asset_server.load(crate::FONT),
        font_size: 40.0 / SCALE,
        color: TEXT_COLOR,
    };

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::CRIMSON.into(),
            ..default()
        })
        .insert(OnSoundSettingsMenuScreen)
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: Color::CRIMSON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Volume",
                        button_text_style.clone(),
                    ));
                    for volume_setting in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
                        let mut entity = parent.spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(30.0 / SCALE), Val::Px(65.0 / SCALE)),
                                ..button_style.clone()
                            },
                            color: NORMAL_BUTTON.into(),
                            ..default()
                        });
                        entity.insert(Volume(volume_setting));
                        if *volume == Volume(volume_setting) {
                            entity.insert(SelectedOption);
                        }
                    }
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style,
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(MenuButtonAction::BackToSettings)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section("Back", button_text_style));
                });
        });
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<State<MenuState>>,
    mut app_state: ResMut<State<AppState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if *menu_state.current() != MenuState::Disabled && keyboard_input.just_released(KeyCode::Escape)
    {
        app_state.pop().unwrap();
        menu_state.set(MenuState::Disabled).unwrap();
        // No need to check anything beyond this point.
        return;
    }

    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                MenuButtonAction::Quit => app_exit_events.send(AppExit),
                MenuButtonAction::Play => {
                    app_state.pop().unwrap();
                    menu_state.set(MenuState::Disabled).unwrap();
                }
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings).unwrap(),
                MenuButtonAction::SettingsSound => {
                    menu_state.set(MenuState::SettingsSound).unwrap();
                }
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main).unwrap(),
                MenuButtonAction::BackToSettings => {
                    menu_state.set(MenuState::Settings).unwrap();
                }
            }
        }
    }
}
