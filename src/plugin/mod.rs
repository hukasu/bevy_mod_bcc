//! Bevy plugin for [`BinaryCurveCollection`].
//!
//! Register the asset loader for [`BinaryCurveCollection`].

mod asset_loader;

use bevy_app::{App, Plugin};
use bevy_asset::{AssetApp, AssetPlugin};
use log::error;

use crate::{BinaryCurveCollection, plugin::asset_loader::BinaryCurveCollectionAssetLoader};

/// Bevy plugin for [`BinaryCurveCollection`].
///
/// Register the asset loader for [`BinaryCurveCollection`].
pub struct BinaryCurveCollectionPlugin;

impl Plugin for BinaryCurveCollectionPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AssetPlugin>() {
            error!(
                "AssetPlugin is required by BinaryCurveCollectionPlugin. Make sure to add it.\
            If you are using DefaultPlugins, make sure that the `bevy_asset` feature is enabled."
            );
            return;
        }

        app.init_asset::<BinaryCurveCollection>();
        app.init_asset_loader::<BinaryCurveCollectionAssetLoader>();
    }
}
