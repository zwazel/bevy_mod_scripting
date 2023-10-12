use bevy::{
    asset::Error,
    asset::{AssetLoader, LoadedAsset},
    reflect::{TypePath, TypeUuid},
};
use bevy_mod_scripting_core::prelude::*;
use std::sync::Arc;

#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "e4f7d00d-5acd-45fb-a29c-5a44c5447f5c"]
/// A rhai code file in bytes
pub struct RhaiFile {
    pub bytes: Arc<[u8]>,
}

impl CodeAsset for RhaiFile {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Default)]
/// Asset loader for lua scripts
pub struct RhaiLoader;

impl AssetLoader for RhaiLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), Error>> {
        load_context.set_default_asset(LoadedAsset::new(RhaiFile {
            bytes: bytes.into(),
        }));
        Box::pin(async move { Ok(()) })
    }

    fn extensions(&self) -> &[&str] {
        &["rhai"]
    }
}
