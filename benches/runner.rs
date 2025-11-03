use std::hint::black_box;

use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList, Behaviour, BuiltinFunctionArgs, create_builtin_function},
        execution::{
            Agent, DefaultHostHooks, JsResult,
            agent::{GcAgent, Options, RealmRoot},
        },
        scripts_and_modules::script::{Script, ScriptOrErrors, parse_script, script_evaluation},
        types::{
            InternalMethods, IntoValue, Object, PropertyDescriptor, PropertyKey,
            String as JsString, Value,
        },
    },
    engine::{
        context::{Bindable, GcScope, NoGcScope},
        rootable::{HeapRootData, Rootable, Scopable},
    },
};

// inline(never) entry points for callgrind

#[inline(never)]
fn parse_script_entry<'a>(
    agent: &mut Agent,
    source_text: JsString,
    strict_mode: bool,
    gc: NoGcScope<'a, '_>,
) -> ScriptOrErrors<'a> {
    // `Realm` type is private, so we cannot have the realm as a parameter, hence we always use the default realm
    parse_script(
        agent,
        source_text,
        agent.current_realm(gc),
        strict_mode,
        None,
        gc,
    )
}

#[inline(never)]
fn script_evaluation_entry<'a>(
    agent: &mut Agent,
    script: Script<'_>,
    gc: GcScope<'a, '_>,
) -> JsResult<'a, Value<'a>> {
    script_evaluation(agent, script, gc)
}

fn initialize_global(agent: &mut Agent, global: Object, mut gc: GcScope) {
    let global = global.scope(agent, gc.nogc());

    // `print` function, but for benchmarks make it a noop
    fn print<'gc>(
        _agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _args = args.bind(gc.nogc());
        Ok(Value::Undefined)
    }

    let function = create_builtin_function(
        agent,
        Behaviour::Regular(print),
        BuiltinFunctionArgs::new(1, "print"),
        gc.nogc(),
    );
    let property_key = PropertyKey::from_static_str(agent, "print", gc.nogc());
    global
        .get(agent)
        .internal_define_own_property(
            agent,
            property_key.unbind(),
            PropertyDescriptor {
                value: Some(function.into_value().unbind()),
                writable: Some(true),
                enumerable: Some(false),
                configurable: Some(true),
                ..Default::default()
            },
            gc.reborrow(),
        )
        .unwrap();
}

pub struct Runner {
    agent: GcAgent,
    realm: RealmRoot,
}

pub struct ParsedScript {
    runner: Runner,
    script: HeapRootData,
}

impl Runner {
    pub fn new(gc: bool) -> Self {
        let mut agent = GcAgent::new(
            Options {
                disable_gc: !gc,
                print_internals: false,
            },
            &DefaultHostHooks,
        );
        let create_global_object: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> =
            None;
        let create_global_this_value: Option<
            for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>,
        > = None;
        let realm = agent.create_realm(
            create_global_object,
            create_global_this_value,
            Some(initialize_global),
        );

        Self { agent, realm }
    }

    pub fn parse_script(mut self, source_str: &str, strict_mode: bool) -> ParsedScript {
        let script = self
            .agent
            .run_in_realm(&self.realm, |agent, gc| -> HeapRootData {
                let source_text = JsString::from_str(agent, source_str, gc.nogc());
                let script = parse_script_entry(agent, source_text, strict_mode, gc.nogc())
                    .expect("parse error");
                Rootable::to_root_repr(script).unwrap_err()
            });
        ParsedScript {
            runner: self,
            script,
        }
    }
}

impl ParsedScript {
    pub fn run(self) -> Runner {
        let ParsedScript { mut runner, script } = self;

        let script = Script::from_heap_data(script).unwrap();
        runner.agent.run_in_realm(&runner.realm, |agent, gc| {
            let result = script_evaluation_entry(agent, script, gc).expect("execution error");
            black_box(result);
        });

        runner
    }
}
