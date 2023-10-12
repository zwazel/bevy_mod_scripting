use crate::{
    assets::{RuneFile, RuneLoader},
    docs::RuneDocFragment,
};
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use bevy_mod_scripting_core::{prelude::*, systems::*, world::WorldPointer};
use rune::{Context, Vm, Module};
use std::marker::PhantomData;

pub mod assets;
pub mod docs;
pub use rune;
pub mod prelude {
    pub use crate::{
        assets::{RuneFile, RuneLoader},
        docs::RuneDocFragment,
        RuneContext, RuneEvent, RuneScriptHost,
    };
    pub use rune;
}

#[derive(Resource)]
pub struct RuneScriptHost<A: Send> {
    pub engine: Vm,
    _ph: PhantomData<A>,
}

#[allow(deprecated)]
impl<A: Send> Default for RuneScriptHost<A> {
    fn default() -> Self {
        let context = Context::default();

        let mut e = Vm::new();
        // prevent shadowing of `state`,`world` and `entity` in variable in scripts
        e.on_def_var(|_, info, _| {
            Ok(info.name() != "state" && info.name() != "world" && info.name() != "entity")
        });

        Self {
            engine: e,
            _ph: Default::default(),
        }
    }
}

pub struct RuneContext;

#[derive(Clone, Event)]
/// A Rune Hook. The result of creating this event will be
/// a call to the rune script with the hook_name and the given arguments
pub struct RuneEvent<A: Clone + 'static> {
    pub hook_name: String,
    pub args: A,
    pub recipients: Recipients,
}

impl<A: Clone + Send + Sync + 'static> ScriptEvent for RuneEvent<A> {
    fn recipients(&self) -> &crate::Recipients {
        &self.recipients
    }
}

impl<A: Send + Clone + Sync + 'static> ScriptHost for RuneScriptHost<A> {
    type ScriptContext = RuneContext;
    type ScriptEvent = RuneEvent<A>;
    type ScriptAsset = RuneFile;
    type APITarget = Vm;
    type DocTarget = RuneDocFragment;

    fn register_with_app_in_set(
        app: &mut bevy::prelude::App,
        schedule: impl ScheduleLabel,
        set: impl SystemSet,
    ) {
        app.add_priority_event::<Self::ScriptEvent>()
            .add_asset::<RuneFile>()
            .init_asset_loader::<RuneLoader>()
            .init_resource::<CachedScriptState<Self>>()
            .init_resource::<ScriptContexts<Self::ScriptContext>>()
            .init_resource::<APIProviders<Self>>()
            .register_type::<ScriptCollection<Self::ScriptAsset>>()
            .register_type::<Script<Self::ScriptAsset>>()
            .register_type::<Handle<RuneFile>>()
            .add_systems(
                schedule,
                (
                    script_add_synchronizer::<Self>,
                    script_remove_synchronizer::<Self>,
                    script_hot_reload_handler::<Self>,
                )
                    .chain()
                    .in_set(set),
            )
            // setup engine
            .add_systems(
                Startup,
                |mut providers: ResMut<APIProviders<Self>>, mut host: ResMut<Self>| {
                    providers
                        .attach_all(&mut host.engine)
                        .expect("Error in adding api's for rune");
                },
            );
    }

    fn setup_script(
        &mut self,
        script_data: &ScriptData,
        ctx: &mut Self::ScriptContext,
        providers: &mut APIProviders<Self>,
    ) -> Result<(), ScriptError> {
        providers.setup_all(script_data, ctx)
    }

    fn load_script(
        &mut self,
        script: &[u8],
        script_data: &ScriptData,
        _: &mut APIProviders<Self>,
    ) -> Result<Self::ScriptContext, ScriptError> {
        let mut scope = Scope::new();
        let mut ast = self
            .engine
            .compile(
                std::str::from_utf8(script).map_err(|e| ScriptError::FailedToLoad {
                    script: script_data.name.to_owned(),
                    msg: e.to_string(),
                })?,
            )
            .map_err(|e| ScriptError::SyntaxError {
                script: script_data.name.to_owned(),
                msg: e.to_string(),
            })?;

        ast.set_source(script_data.name);

        // persistent state for scripts
        scope.push("state", Map::new());

        Ok(RuneContext { ast, scope })
    }

    fn handle_events<'a>(
        &mut self,
        world: &mut World,
        events: &[Self::ScriptEvent],
        ctxs: impl Iterator<Item = (ScriptData<'a>, &'a mut Self::ScriptContext)>,
        providers: &mut APIProviders<Self>,
    ) {
        // safety:
        // - we have &mut World access
        // - we do not use world_ptr after we use the original reference again anywhere in this function
        let world_ptr = unsafe { WorldPointer::new(world) };

        ctxs.for_each(|(fd, ctx)| {
            providers
                .setup_runtime_all(world_ptr.clone(), &fd, ctx)
                .expect("Failed to setup script runtime");

            for event in events.iter() {
                // check if this script should handle this event
                if !event.recipients().is_recipient(&fd) {
                    continue;
                };

                match self.engine.call_fn(
                    &mut ctx.scope,
                    &ctx.ast,
                    &event.hook_name,
                    event.args.clone(),
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        let mut world = world_ptr.write();
                        let mut state: CachedScriptState<Self> = world.remove_resource().unwrap();

                        match *e {
                            EvalAltResult::ErrorFunctionNotFound(..) => {}
                            _ => {
                                let (_, mut error_wrt, _) = state.event_state.get_mut(&mut world);

                                let error = ScriptError::RuntimeError {
                                    script: fd.name.to_string(),
                                    msg: e.to_string(),
                                };
                                error!("{}", error);
                                error_wrt.send(ScriptErrorEvent { error });
                            }
                        }

                        world.insert_resource(state);
                    }
                };
            }

            // executing this at the end here means we execute global statements exactly once
            // all this method call does is set a variable on the AST to NONE so should not affect performance
            ctx.ast.clear_statements();
        });
    }
}
