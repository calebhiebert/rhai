#![cfg(not(feature = "unchecked"))]
use rhai::{Engine, EvalAltResult, INT};

#[test]
fn test_max_operations() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    #[cfg(not(feature = "no_optimize"))]
    engine.set_optimization_level(rhai::OptimizationLevel::None);
    engine.set_max_operations(500);

    engine.on_progress(|count| {
        if count % 100 == 0 {
            println!("{}", count);
        }
        None
    });

    engine.run("let x = 0; while x < 20 { x += 1; }")?;

    assert!(matches!(
        *engine.run("for x in 0..500 {}").expect_err("should error"),
        EvalAltResult::ErrorTooManyOperations(_)
    ));

    engine.set_max_operations(0);

    engine.run("for x in 0..10000 {}")?;

    Ok(())
}

#[test]
fn test_max_operations_functions() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    engine.set_max_operations(500);

    engine.on_progress(|count| {
        if count % 100 == 0 {
            println!("{}", count);
        }
        None
    });

    engine.run(
        r#"
            print("Test1");
            let x = 0;

            while x < 28 {
                print(x);
                x += 1;
            }
        "#,
    )?;

    #[cfg(not(feature = "no_function"))]
    engine.run(
        r#"
            print("Test2");
            fn inc(x) { x + 1 }
            let x = 0;
            while x < 20 { x = inc(x); }
        "#,
    )?;

    #[cfg(not(feature = "no_function"))]
    assert!(matches!(
        *engine
            .run(
                r#"
                    print("Test3");
                    fn inc(x) { x + 1 }
                    let x = 0;

                    while x < 36 {
                        print(x);
                        x = inc(x);
                    }
                "#,
            )
            .expect_err("should error"),
        EvalAltResult::ErrorTooManyOperations(_)
    ));

    Ok(())
}

#[test]
fn test_max_operations_eval() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    engine.set_max_operations(500);

    engine.on_progress(|count| {
        if count % 100 == 0 {
            println!("{}", count);
        }
        None
    });

    assert!(matches!(
        *engine
            .run(
                r#"
                    let script = "for x in 0..500 {}";
                    eval(script);
                "#
            )
            .expect_err("should error"),
        EvalAltResult::ErrorInFunctionCall(_, _, err, _) if matches!(*err, EvalAltResult::ErrorTooManyOperations(_))
    ));

    Ok(())
}

#[test]
fn test_max_operations_progress() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();
    #[cfg(not(feature = "no_optimize"))]
    engine.set_optimization_level(rhai::OptimizationLevel::None);
    engine.set_max_operations(500);

    engine.on_progress(|count| {
        if count < 100 {
            None
        } else {
            Some((42 as INT).into())
        }
    });

    assert!(matches!(
        *engine
            .run("for x in 0..500 {}")
            .expect_err("should error"),
        EvalAltResult::ErrorTerminated(x, _) if x.as_int()? == 42
    ));

    Ok(())
}
