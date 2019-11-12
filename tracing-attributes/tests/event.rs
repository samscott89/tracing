mod support;
use support::*;

use tracing::subscriber::with_default;
use tracing::Level;
use tracing_attributes::event;

fn message(s: &'static str) -> field::MockField {
    field::mock("message").with_value(&tracing::field::debug(format_args!("{}", s)))
}

#[test]
fn override_everything() {
    #[event(target = "my_target", level = "debug")]
    fn my_fn() {}

    #[event(level = "debug", target = "my_target")]
    fn my_other_fn() {}

    let (subscriber, handle) = subscriber::mock()
        .event(
            event::mock()
                .with_target("my_target")
                .at_level(Level::DEBUG)
                .with_fields(message("my_fn")),
        )
        .event(
            event::mock()
                .with_target("my_target")
                .at_level(Level::DEBUG)
                .with_fields(message("my_other_fn")),
        )
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        my_fn();
        my_other_fn();
    });

    handle.assert_finished();
}

#[test]
fn fields() {
    #[event(target = "my_target", level = "debug")]
    fn my_fn(arg1: usize, arg2: bool) {}

    let (subscriber, handle) = subscriber::mock()
        .event(
            event::mock()
                .with_target("my_target")
                .at_level(Level::DEBUG)
                .with_fields(
                    message("my_fn")
                        .and(field::mock("arg1").with_value(&format_args!("2")))
                        .and(field::mock("arg2").with_value(&format_args!("false"))),
                ),
        )
        .event(
            event::mock()
                .with_target("my_target")
                .at_level(Level::DEBUG)
                .with_fields(
                    message("my_fn")
                        .and(field::mock("arg1").with_value(&format_args!("3")))
                        .and(field::mock("arg2").with_value(&format_args!("true"))),
                ),
        )
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        my_fn(2, false);
        my_fn(3, true);
    });

    handle.assert_finished();
}

#[test]
fn skip() {
    struct UnDebug(pub u32);

    #[event(target = "my_target", level = "debug", skip(_arg2, _arg3))]
    fn my_fn(arg1: usize, _arg2: UnDebug, _arg3: UnDebug) {}

    let (subscriber, handle) = subscriber::mock()
        .event(
            event::mock()
                .with_target("my_target")
                .at_level(Level::DEBUG)
                .with_fields(
                    message("my_fn").and(field::mock("arg1").with_value(&format_args!("2"))),
                ),
        )
        .event(
            event::mock()
                .with_target("my_target")
                .at_level(Level::DEBUG)
                .with_fields(
                    message("my_fn").and(field::mock("arg1").with_value(&format_args!("3"))),
                ),
        )
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        my_fn(2, UnDebug(0), UnDebug(1));
        my_fn(3, UnDebug(0), UnDebug(1));
    });

    handle.assert_finished();
}

#[test]
fn generics() {
    #[derive(Debug)]
    struct Foo;

    #[event]
    fn my_fn<S, T: std::fmt::Debug>(arg1: S, arg2: T)
    where
        S: std::fmt::Debug,
    {
    }

    let (subscriber, handle) = subscriber::mock()
        .event(
            event::mock().with_fields(
                message("my_fn")
                    .and(field::mock("arg1").with_value(&format_args!("Foo")))
                    .and(field::mock("arg2").with_value(&format_args!("false"))),
            ),
        )
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        my_fn(Foo, false);
    });

    handle.assert_finished();
}

#[test]
fn methods() {
    #[derive(Debug)]
    struct Foo;

    impl Foo {
        #[event]
        fn my_fn(&self, arg1: usize) {}
    }

    let (subscriber, handle) = subscriber::mock()
        .event(
            event::mock().with_fields(
                message("my_fn")
                    .and(field::mock("self").with_value(&format_args!("Foo")))
                    .and(field::mock("arg1").with_value(&format_args!("42"))),
            ),
        )
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let foo = Foo;
        foo.my_fn(42);
    });

    handle.assert_finished();
}
