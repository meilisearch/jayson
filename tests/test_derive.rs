use jayson::{de::VisitorError, json, Error, Jayson};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize, Jayson)]
#[serde(tag = "sometag")]
#[jayson(error = Error, tag = "sometag")]
enum Tag {
    A,
    B,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Jayson)]
#[jayson(error = Error)]
struct Example {
    x: String,
    t1: Tag,
    t2: Box<Tag>,
    n: Box<Nested>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Jayson)]
#[jayson(error = Error)]
struct Nested {
    y: Option<Vec<String>>,
    z: Option<String>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Jayson)]
#[jayson(error = Error)]
struct StructWithDefaultAttr {
    x: bool,
    #[serde(default = "create_default_u8")]
    #[jayson(default = create_default_u8())]
    y: u8,
    #[serde(default = "create_default_option_string")]
    #[jayson(default = create_default_option_string())]
    z: Option<String>,
}
fn create_default_u8() -> u8 {
    152
}
fn create_default_option_string() -> Option<String> {
    Some("hello".to_owned())
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Jayson)]
#[serde(tag = "t")]
#[jayson(error = Error, tag = "t")]
enum EnumWithOptionData {
    A {
        x: Option<u8>,
    },
    B {
        #[serde(default = "create_default_option_string")]
        #[jayson(default = create_default_option_string())]
        x: Option<String>,
        #[serde(default = "create_default_u8")]
        #[jayson(default = create_default_u8())]
        y: u8,
    },
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Jayson)]
#[jayson(error = Error, rename_all = camelCase)]
#[serde(rename_all = "camelCase")]
struct RenamedAllCamelCaseStruct {
    renamed_field: bool,
}
#[derive(PartialEq, Debug, Serialize, Deserialize, Jayson)]
#[jayson(error = Error, rename_all = lowercase)]
#[serde(rename_all = "lowercase")]
struct RenamedAllLowerCaseStruct {
    renamed_field: bool,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Jayson)]
#[jayson(error = Error, tag = "t", rename_all = camelCase)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
enum RenamedAllCamelCaseEnum {
    SomeField { my_field: bool },
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Jayson)]
#[jayson(error = Error)]
struct StructWithRenamedField {
    #[jayson(rename = renamed_field)]
    #[serde(rename = "renamed_field")]
    x: bool,
}

#[track_caller]
fn compare_with_serde_roundtrip<T>(x: T)
where
    T: Serialize + Jayson + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string_pretty(&x).unwrap();
    let actual_jayson: T = json::from_str(json.as_str()).unwrap();

    assert_eq!(actual_jayson, x);
}

fn compare_with_serde<T>(j: &str)
where
    T: DeserializeOwned + Serialize + Jayson + PartialEq + std::fmt::Debug,
{
    let actual_serde = serde_json::from_str(j).unwrap();
    let actual_jayson: T = json::from_str(j).unwrap();

    assert_eq!(actual_jayson, actual_serde);
}

#[test]
fn test_de() {
    // arbitrary struct, roundtrip
    compare_with_serde_roundtrip(Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Box::new(Tag::B),
        n: Box::new(Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        }),
    });

    // struct rename all camel case, roundtrip
    compare_with_serde_roundtrip(RenamedAllCamelCaseStruct {
        renamed_field: true,
    });
    // struct rename all lower case, roundtrip
    compare_with_serde_roundtrip(RenamedAllLowerCaseStruct {
        renamed_field: true,
    });

    // enum rename all variants camel case, roundtrip
    compare_with_serde_roundtrip(RenamedAllCamelCaseEnum::SomeField { my_field: true });

    // struct default attributes serde, roundtrip
    compare_with_serde_roundtrip(StructWithDefaultAttr {
        x: true,
        y: 1,
        z: None,
    });

    // struct default attributes, missing field
    compare_with_serde::<StructWithDefaultAttr>(
        r#"{
            "x": true,
            "y": 10
        }
        "#,
    );

    // enum with optional data inside variant, roundtrip
    compare_with_serde_roundtrip(EnumWithOptionData::A { x: None });

    // enum with optional data inside variant, missing field
    compare_with_serde::<EnumWithOptionData>(r#"{ "t": "A" }"#);

    // enum with optional and defaultable data inside variant, missing fields
    compare_with_serde::<EnumWithOptionData>(r#"{ "t": "B" }"#);

    // enum with optional and defaultable data inside variant, all fields present
    compare_with_serde::<EnumWithOptionData>(
        r#"{
            "t": "B",
            "x": null,
            "y": 10
        }
        "#,
    );

    compare_with_serde_roundtrip(StructWithRenamedField { x: true });
}
