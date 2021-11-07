#![cfg(not(feature = "no_function"))]
use rhai::{Engine, EvalAltResult, FnNamespace, Module, NativeCallContext, Shared, INT};

#[cfg(not(feature = "no_object"))]
#[test]
fn test_functions_trait_object() -> Result<(), Box<EvalAltResult>> {
    trait TestTrait {
        fn greet(&self) -> INT;
    }

    #[derive(Debug, Clone)]
    struct ABC(INT);

    impl TestTrait for ABC {
        fn greet(&self) -> INT {
            self.0
        }
    }

    #[cfg(not(feature = "sync"))]
    type MySharedTestTrait = Shared<dyn TestTrait>;

    #[cfg(feature = "sync")]
    type MySharedTestTrait = Shared<dyn TestTrait + Send + Sync>;

    let mut engine = Engine::new();

    engine
        .register_type_with_name::<MySharedTestTrait>("MySharedTestTrait")
        .register_fn("new_ts", || Shared::new(ABC(42)) as MySharedTestTrait)
        .register_fn("greet", |x: MySharedTestTrait| x.greet());

    assert_eq!(
        engine.eval::<String>("type_of(new_ts())")?,
        "MySharedTestTrait"
    );
    assert_eq!(engine.eval::<INT>("let x = new_ts(); greet(x)")?, 42);

    Ok(())
}

#[test]
fn test_functions_namespaces() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    #[cfg(not(feature = "no_module"))]
    {
        let mut m = Module::new();
        let hash = m.set_native_fn("test", || Ok(999 as INT));
        m.update_fn_namespace(hash, FnNamespace::Global);

        engine.register_static_module("hello", m.into());

        let mut m = Module::new();
        m.set_var("ANSWER", 123 as INT);

        assert_eq!(engine.eval::<INT>("test()")?, 999);

        assert_eq!(engine.eval::<INT>("fn test() { 123 } test()")?, 123);
    }

    engine.register_fn("test", || 42 as INT);

    assert_eq!(engine.eval::<INT>("fn test() { 123 } test()")?, 123);

    assert_eq!(engine.eval::<INT>("test()")?, 42);

    Ok(())
}

#[cfg(not(feature = "no_module"))]
#[test]
fn test_functions_global_module() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    assert_eq!(
        engine.eval::<INT>(
            "
                const ANSWER = 42;
                fn foo() { global::ANSWER }
                foo()
            "
        )?,
        42
    );

    assert!(matches!(*engine.run("
            fn foo() { global::ANSWER }

            {
                const ANSWER = 42;
                foo()
            }
        ").expect_err("should error"),
        EvalAltResult::ErrorInFunctionCall(_, _, err, _)
            if matches!(&*err, EvalAltResult::ErrorVariableNotFound(v, _) if v == "global::ANSWER")
    ));

    let mut module = Module::new();
    module.set_var("ANSWER", 123 as INT);
    engine.register_static_module("global", module.into());

    assert_eq!(
        engine.eval::<INT>(
            "
                const ANSWER = 42;
                fn foo() { global::ANSWER }
                foo()
            "
        )?,
        123
    );

    engine.register_result_fn(
        "do_stuff",
        |context: NativeCallContext, callback: rhai::FnPtr| {
            callback.call_dynamic(&context, None, [])
        },
    );

    #[cfg(not(feature = "no_closure"))]
    assert!(matches!(*engine.run("
            do_stuff(|| {
                const LOCAL_VALUE = 42;
                global::LOCAL_VALUE
            });
        ").expect_err("should error"),
        EvalAltResult::ErrorInFunctionCall(_, _, err, _)
            if matches!(&*err, EvalAltResult::ErrorVariableNotFound(v, _) if v == "global::LOCAL_VALUE")
    ));

    Ok(())
}
