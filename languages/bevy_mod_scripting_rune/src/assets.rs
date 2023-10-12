use bevy::{
    asset::Error,
    asset::{AssetLoader, LoadedAsset},
    reflect::{TypePath, TypeUuid},
};
use bevy_mod_scripting_core::prelude::*;
use std::sync::Arc;

#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "073dced5-ce45-44ba-a601-f82b04d268e9"]
/// A rune code file in bytes
pub struct RuneFile {
    pub bytes: Arc<[u8]>,
}

impl CodeAsset for RuneFile {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Default)]
/// Asset loader for Rune scripts
pub struct RuneLoader;

impl AssetLoader for RuneLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), Error>> {
        load_context.set_default_asset(LoadedAsset::new(RuneFile {
            bytes: bytes.into(),
        }));
        Box::pin(async move { Ok(()) })
    }

    fn extensions(&self) -> &[&str] {
        &["rn"]
    }
}
