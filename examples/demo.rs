use denvars::Deserializer;
use serde::Deserialize;

fn main() {
    let de: Deserializer = Deserializer::from_vars([
        ("A_STRING".to_owned(), "BAR".to_owned()),
        ("ESCAPED_STRING".to_owned(), r#""q\x20w\u{20}e""#.to_owned()),
        ("AN_INT".to_owned(), "42".to_owned()),
        ("A_FLOAT".to_owned(), "42.1".to_owned()),
        ("A_SEQ".to_owned(), "a,b,\"a\\x20w\\u{20}e\"".to_owned()),
    ]);

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
