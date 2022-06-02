use jayson::{de::VisitorError, json, Error, Jayson};
use serde::{Deserialize, Serialize};

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

#[derive(PartialEq, Debug, Jayson)]
#[jayson(error = "Error")]
struct StructWithDefaultAttr {
    x: bool,
    #[serde(default = "create_default_u8")]
    y: u8,
    #[jayson(default = "create_default_option_string")]
    z: Option<String>,
}
fn create_default_u8() -> u8 {
    1
}
fn create_default_option_string() -> Option<String> {
    Some("helllo".to_owned())
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Jayson)]
#[serde(tag = "t")]
#[jayson(error = "Error", tag = "t")]
enum EnumWithOptionData {
    A {
        x: Option<u8>,
    },
    B {
        #[serde(default = "create_default_option_string")]
        x: Option<String>,
        #[serde(default = "create_default_u8")]
        y: u8,
    },
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

    let j = r#"{
            "x": true,
            "y": 10
        }
        "#;
    let actual: StructWithDefaultAttr = json::from_str(j).unwrap();
    let expected = StructWithDefaultAttr {
        x: true,
        y: 10,
        z: create_default_option_string(),
    };
    assert_eq!(actual, expected);

    assert_eq!(actual, expected);

    let j = r#"{
            "x": true,
            "z": null
        }
        "#;
    let actual: StructWithDefaultAttr = json::from_str(j).unwrap();
    let expected = StructWithDefaultAttr {
        x: true,
        y: 1,
        z: None,
    };
    assert_eq!(actual, expected);

    let j = r#"{
            "t": "A"
        }
        "#;
    let actual_serde: EnumWithOptionData = serde_json::from_str(j).unwrap();
    let expected = EnumWithOptionData::A { x: None };
    assert_eq!(actual_serde, expected);

    let actual_jayson: EnumWithOptionData = json::from_str(j).unwrap();
    assert_eq!(actual_jayson, expected);

    let j = r#"{
            "t": "A"
        }
        "#;
    let actual_serde: EnumWithOptionData = serde_json::from_str(j).unwrap();
    let expected = EnumWithOptionData::A { x: None };
    assert_eq!(actual_serde, expected);

    let actual_jayson: EnumWithOptionData = json::from_str(j).unwrap();
    assert_eq!(actual_jayson, expected);

    let j = r#"{
            "t": "B"
        }
        "#;
    let actual_serde: EnumWithOptionData = serde_json::from_str(j).unwrap();
    let expected = EnumWithOptionData::B {
        x: create_default_option_string(),
        y: create_default_u8(),
    };
    assert_eq!(actual_serde, expected);

    let actual_jayson: EnumWithOptionData = json::from_str(j).unwrap();
    assert_eq!(actual_jayson, expected);

    let j = r#"{
            "t": "B",
            "x": null,
            "y": 10
        }
        "#;
    let actual_serde: EnumWithOptionData = serde_json::from_str(j).unwrap();
    let expected = EnumWithOptionData::B { x: None, y: 10 };
    assert_eq!(actual_serde, expected);

    let actual_jayson: EnumWithOptionData = json::from_str(j).unwrap();
    assert_eq!(actual_jayson, expected);
}
