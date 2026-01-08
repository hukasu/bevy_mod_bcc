//! Quick example to test that the asset loader is working

use bevy::{color::palettes, prelude::*};
use bevy_asset::RecursiveDependencyLoadState;
use bevy_mod_bcc::{BinaryCurveCollection, plugin::BinaryCurveCollectionPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(BinaryCurveCollectionPlugin);

    app.add_systems(Startup, start_loading);
    app.add_systems(Update, wait_loading.run_if(resource_exists::<LoadingBcc>));

    app.run();
}

/// Holds the [`Handle`] to a loading [`BinaryCurveCollection`]
#[derive(Resource, Deref)]
struct LoadingBcc(Handle<BinaryCurveCollection>);

/// Starts the loading a [`BinaryCurveCollection`] file
fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 75., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            clear_color: ClearColorConfig::Custom(palettes::tailwind::GRAY_100.into()),
            ..Default::default()
        },
    ));
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(0., 5., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.insert_resource(LoadingBcc(asset_server.load("cable_work_pattern.bcc")));
}

/// Waits for the [`BinaryCurveCollection`] to finish loading or failing to load, report the final
/// status, and closes the app.
fn wait_loading(
    mut commands: Commands,
    loading_bcc: Res<LoadingBcc>,
    asset_server: Res<AssetServer>,
    binary_curve_collections: Res<Assets<BinaryCurveCollection>>,
) {
    info!("Loop");
    match asset_server.get_recursive_dependency_load_state(loading_bcc.id()) {
        Some(RecursiveDependencyLoadState::NotLoaded | RecursiveDependencyLoadState::Loading) => (),
        Some(RecursiveDependencyLoadState::Loaded) => {
            info!("Bcc file loaded successfully.");
            let Some(binary_curve_collection) = binary_curve_collections.get(loading_bcc.id())
            else {
                unreachable!("Bcc file should be loaded and available at this point.");
            };
            info!("{:?}", binary_curve_collection.header());
            commands.spawn((
                Mesh3d(asset_server.add(binary_curve_collection.mesh().build())),
                MeshMaterial3d::<StandardMaterial>(
                    asset_server.add(StandardMaterial::from_color(palettes::tailwind::BLUE_500)),
                ),
            ));
            commands.remove_resource::<LoadingBcc>();
        }
        Some(RecursiveDependencyLoadState::Failed(err)) => {
            error!("Failed to load Bcc file due to '{err}'.");
            commands.write_message(AppExit::from_code(1));
        }
        None => {
            error!("Handle to loading Bcc file did not exist on AssetServer.");
            commands.write_message(AppExit::from_code(2));
        }
    }
}
