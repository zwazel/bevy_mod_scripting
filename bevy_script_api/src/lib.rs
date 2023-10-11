extern crate bevy;

pub mod error;
#[cfg(feature = "lua")]
pub mod lua;
#[cfg(feature = "rhai")]
pub mod rhai;
#[cfg(feature = "rune")]
pub mod rune;

pub mod common;

pub mod script_ref;
pub mod sub_reflect;
pub mod wrappers;

pub use {script_ref::*, sub_reflect::*};

pub mod prelude {
    #[cfg(feature = "lua")]
    pub use crate::{
        impl_lua_newtype,
        lua::{
            bevy::LuaBevyAPIProvider, std::LuaVec, FromLuaProxy, LuaProxyable, ReflectLuaProxyable,
            ToLuaProxy,
        },
    };

    #[cfg(feature = "rhai")]
    pub use crate::rhai::{
        bevy::RhaiBevyAPIProvider,
        std::{RhaiCopy, RhaiVec},
        FromRhaiProxy, ReflectRhaiProxyable, RhaiProxyable, ToRhaiProxy,
    };

    pub use crate::{common::bevy::GetWorld, impl_script_newtype, ValueIndex};
}

// re-export derive macros from other langs
pub use bevy_mod_scripting_derive::impl_script_newtype;
#[cfg(feature = "lua")]
pub use bevy_mod_scripting_lua_derive::impl_lua_newtype; //LuaProxy};

pub(crate) mod generated;

pub use parking_lot;
