use jayson::{de::VisitorError, json, Error, Jayson};

#[derive(PartialEq, Debug, Jayson)]
#[jayson(error = "Error", tag = "sometag")]
enum Tag {
    A,
    B,
}

#[derive(PartialEq, Debug, Jayson)]
#[jayson(error = "Error")]
struct Example {
    x: String,
    t1: Tag,
    t2: Box<Tag>,
    n: Box<Nested>,
}

#[derive(PartialEq, Debug, Jayson)]
#[jayson(error = "Error")]
struct Nested {
    y: Option<Vec<String>>,
    z: Option<String>,
}

#[test]
fn test_de() {
    let j = r#" {"x": "X", "t1": { "sometag": "A" }, "t2": { "sometag": "B" }, "n": {"y": ["Y", "Y"]}} "#;
    let actual: Example = json::from_str(j).unwrap();
    let expected = Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Box::new(Tag::B),
        n: Box::new(Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        }),
    };
    assert_eq!(actual, expected);
}
