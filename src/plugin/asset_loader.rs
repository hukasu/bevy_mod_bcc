//! Bevy asset loader for `bcc` files

use bevy_asset::AssetLoader;
use bevy_reflect::TypePath;

use crate::{BinaryCurveCollection, BinaryCurveCollectionParserError};

/// Asset loader for [`BinaryCurveCollection`] files
#[derive(Default, TypePath)]
pub struct BinaryCurveCollectionAssetLoader;

impl AssetLoader for BinaryCurveCollectionAssetLoader {
    type Asset = BinaryCurveCollection;
    type Settings = ();
    type Error = BinaryCurveCollectionParserError;

    async fn load(
        &self,
        reader: &mut dyn bevy_asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy_asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        BinaryCurveCollection::parse_async(reader).await
    }

    fn extensions(&self) -> &[&str] {
        &["bcc"]
    }
}
