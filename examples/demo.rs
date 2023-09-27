use denvars::Deserializer;
use serde::Deserialize;

fn main() {
    std::env::set_var("A_STRING", "BAR");
    std::env::set_var("ESCAPED_STRING", r#""q\x20w\u{20}e""#);
    std::env::set_var("AN_INT", "42");
    std::env::set_var("A_FLOAT", "42.1");
    std::env::set_var("A_SEQ", "a,b,\"a\\x20w\\u{20}e\"");

    let de: Deserializer = Deserializer::from_env_vars();

    #[derive(Debug, serde_derive::Deserialize)]
    struct Foo {
        a_string: String,
        escaped_string: String,
        an_int: i32,
        a_float: f32,
        a_seq: Vec<String>,
    }

    let foo = Foo::deserialize(de).unwrap();
    debug_assert_eq!(foo.a_string, "BAR");
    debug_assert_eq!(foo.escaped_string, "q w e");
    debug_assert_eq!(foo.an_int, 42);
    debug_assert_eq!(foo.a_float, 42.1);
    debug_assert_eq!(
        foo.a_seq,
        vec!["a".to_owned(), "b".to_owned(), "a w e".to_owned()]
    );
}
