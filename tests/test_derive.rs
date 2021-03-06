use jayson::{DeserializeError, DeserializeFromValue, MergeWithError, ValuePointerRef};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum MyError {
    Unexpected(String),
    MissingField(String),
    IncorrectValueKind { accepted: Vec<jayson::ValueKind> },
    UnknownKey { key: String, accepted: Vec<String> },
    CustomMissingField(u8),
    Validation,
}
impl MergeWithError<MyError> for MyError {
    fn merge(
        _self_: Option<Self>,
        other: MyError,
        _merge_location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(other)
    }
}
impl DeserializeError for MyError {
    fn location(&self) -> Option<jayson::ValuePointer> {
        None
    }
    fn incorrect_value_kind(
        _self_: Option<Self>,
        _actual: jayson::ValueKind,
        accepted: &[jayson::ValueKind],
        _location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(Self::IncorrectValueKind {
            accepted: accepted.into(),
        })
    }

    fn missing_field(
        _self_: Option<Self>,
        field: &str,
        _location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(Self::MissingField(field.to_string()))
    }

    fn unknown_key(
        _self_: Option<Self>,
        key: &str,
        accepted: &[&str],
        _location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(Self::UnknownKey {
            key: key.to_string(),
            accepted: accepted.into_iter().map(<_>::to_string).collect(),
        })
    }

    fn unexpected(
        _self_: Option<Self>,
        msg: &str,
        _location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(Self::Unexpected(msg.to_string()))
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[serde(tag = "sometag")]
#[jayson(tag = "sometag")]
enum Tag {
    A,
    B,
}

fn unknown_field_error_gen<E>(k: &str, _accepted: &[&str], location: jayson::ValuePointerRef) -> E
where
    E: DeserializeError,
{
    match E::unexpected(None, k, location) {
        Ok(e) => e,
        Err(e) => e,
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(deny_unknown_fields = unknown_field_error_gen)]
struct Example {
    x: String,
    t1: Tag,
    t2: Box<Tag>,
    n: Box<Nested>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
struct Nested {
    y: Option<Vec<String>>,
    z: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError)]
struct StructWithDefaultAttr {
    x: bool,
    #[serde(default = "create_default_u8")]
    #[jayson(default = create_default_u8())]
    y: u8,
    #[serde(default = "create_default_option_string")]
    #[jayson(default = create_default_option_string())]
    z: Option<String>,
}
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError)]
struct StructWithTraitDefaultAttr {
    #[serde(default)]
    #[jayson(default)]
    y: u8,
}

fn create_default_u8() -> u8 {
    152
}
fn create_default_option_string() -> Option<String> {
    Some("hello".to_owned())
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[serde(tag = "t")]
#[jayson(error = MyError, tag = "t")]
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

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, rename_all = camelCase)]
#[serde(rename_all = "camelCase")]
struct RenamedAllCamelCaseStruct {
    renamed_field: bool,
}
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, rename_all = lowercase)]
#[serde(rename_all = "lowercase")]
struct RenamedAllLowerCaseStruct {
    renamed_field: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t", rename_all = camelCase)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
enum RenamedAllCamelCaseEnum {
    SomeField { my_field: bool },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t")]
#[serde(tag = "t")]
enum RenamedAllFieldsCamelCaseEnum {
    #[jayson(rename_all = camelCase)]
    #[serde(rename_all = "camelCase")]
    SomeField { my_field: bool },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError)]
struct StructWithRenamedField {
    #[jayson(rename = "renamed_field")]
    #[serde(rename = "renamed_field")]
    x: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, rename_all = camelCase)]
struct StructWithRenamedFieldAndRenameAll {
    #[jayson(rename = "renamed_field")]
    #[serde(rename = "renamed_field")]
    x: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, deny_unknown_fields)]
#[serde(deny_unknown_fields)]
struct StructDenyUnknownFields {
    x: bool,
}

fn unknown_field_error(k: &str, _accepted: &[&str], _location: ValuePointerRef) -> MyError {
    MyError::UnknownKey {
        key: k.to_owned(),
        accepted: vec!["don't know".to_string()],
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, deny_unknown_fields = unknown_field_error)]
#[serde(deny_unknown_fields)]
struct StructDenyUnknownFieldsCustom {
    x: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t", deny_unknown_fields)]
#[serde(tag = "t", deny_unknown_fields)]
enum EnumDenyUnknownFields {
    SomeField { my_field: bool },
    Other { my_field: bool, y: u8 },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t", deny_unknown_fields = unknown_field_error)]
#[serde(tag = "t", deny_unknown_fields)]
enum EnumDenyUnknownFieldsCustom {
    SomeField { my_field: bool },
    Other { my_field: bool, y: u8 },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError)]
struct StructMissingFieldError {
    #[jayson(missing_field_error = MyError::MissingField("lol".to_string()))]
    x: bool,
    #[jayson(missing_field_error = MyError::CustomMissingField(1))]
    y: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t")]
enum EnumMissingFieldError {
    A {
        #[jayson(missing_field_error = MyError::CustomMissingField(0))]
        x: bool,
    },
    B {
        x: bool,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t")]
#[serde(tag = "t")]
enum EnumRenamedVariant {
    #[serde(rename = "Apple")]
    #[jayson(rename = "Apple")]
    A { x: bool },
    #[serde(rename = "Beta")]
    #[jayson(rename = "Beta")]
    B,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t")]
#[serde(tag = "t")]
enum EnumRenamedField {
    A {
        #[jayson(rename = "Xylem")]
        #[serde(rename = "Xylem")]
        x: bool,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t")]
#[serde(tag = "t")]
enum EnumRenamedAllVariant {
    #[jayson(rename_all = camelCase)]
    #[serde(rename_all = "camelCase")]
    P { water_potential: bool },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError)]
struct Generic<A> {
    some_field: A,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(where_predicate = __Jayson_E: MergeWithError<MyError>, where_predicate = A: DeserializeFromValue<MyError>)]
struct Generic2<A> {
    #[jayson(error = MyError)]
    some_field: Option<A>,
}

fn map_option(x: Option<u8>) -> Option<u8> {
    match x {
        Some(0) => None,
        Some(x) => Some(x),
        None => Some(1),
    }
}
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
struct FieldMap {
    #[jayson(map = map_option)]
    some_field: Option<u8>,
}

// For AscDesc, we have __Jayson_E where __Jayson_E: MergeWithError<AscDescError>
// Then for the struct that contains AscDesc, we don't want to repeat this whole requirement
// so instead we do: AscDesc: DeserializeFromValue<__Jayson_E>
// but that's only if it's generic! If it's not, we don't even need to have any requirements

// #[jayson(where_predicates_from_fields)]

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(where_predicate = Option<u8> : DeserializeFromValue<__Jayson_E>)]
struct FieldConditions {
    some_field: Option<u8>,
}

pub enum NeverError {}

fn parse_hello(b: bool) -> Result<Hello, NeverError> {
    match b {
        true => Ok(Hello::A),
        false => Ok(Hello::B),
    }
}
fn parse_hello2(b: bool) -> Result<Hello2, NeverError> {
    match b {
        true => Ok(Hello2::A),
        false => Ok(Hello2::B),
    }
}
fn parse_hello3(b: &str) -> Result<Hello3, MyError> {
    match b {
        "A" => Ok(Hello3::A),
        "B" => Ok(Hello3::B),
        _ => Err(MyError::Unexpected("Hello3 from error".to_string())),
    }
}

#[derive(Debug, PartialEq, DeserializeFromValue)]
#[jayson(from(bool) = parse_hello -> NeverError)]
enum Hello {
    A,
    B,
}
#[derive(Debug, PartialEq, DeserializeFromValue)]
#[jayson(error = MyError, from(bool) = parse_hello2 -> NeverError)]
enum Hello2 {
    A,
    B,
}
#[derive(Debug, PartialEq, DeserializeFromValue)]
#[jayson(from(& String) = parse_hello3 -> MyError)]
enum Hello3 {
    A,
    B,
}

#[derive(Debug, PartialEq, DeserializeFromValue)]
#[jayson(where_predicate = Hello: DeserializeFromValue<__Jayson_E>)]
struct ContainsHello {
    _x: Hello,
}

#[derive(Debug, PartialEq, DeserializeFromValue)]
#[jayson(error = MyError)]
struct ContainsHello2 {
    _x: Hello,
}

#[derive(Debug, PartialEq, DeserializeFromValue)]
struct ContainsHello3 {
    #[jayson(needs_predicate)]
    _x: Hello,
}

struct MyValidationError;
impl MergeWithError<MyValidationError> for MyError {
    fn merge(
        _self_: Option<Self>,
        _other: MyValidationError,
        _merge_location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(MyError::Validation)
    }
}

fn validate_it(x: Validated) -> Result<Validated, MyValidationError> {
    if x.x as u16 > x.y {
        Err(MyValidationError)
    } else {
        Ok(x)
    }
}
fn validate_it2(x: Validated2) -> Result<Validated2, MyValidationError> {
    if x.x as u16 > x.y {
        Err(MyValidationError)
    } else {
        Ok(x)
    }
}

#[derive(Debug, DeserializeFromValue)]
#[jayson(validate = validate_it -> MyValidationError)]
struct Validated {
    x: u8,
    y: u16,
}

#[derive(Debug, DeserializeFromValue)]
#[jayson(error = MyError, validate = validate_it2 -> MyValidationError)]
struct Validated2 {
    x: u8,
    y: u16,
}

impl MergeWithError<NeverError> for MyError {
    fn merge(
        _self_: Option<Self>,
        _other: NeverError,
        _merge_location: ValuePointerRef,
    ) -> Result<Self, Self> {
        unreachable!()
    }
}

#[track_caller]
fn compare_with_serde_roundtrip<T>(x: T)
where
    T: Serialize + DeserializeFromValue<MyError> + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_value(&x).unwrap();
    let result: T = jayson::deserialize(json).unwrap();

    assert_eq!(result, x);
}

#[track_caller]
fn compare_with_serde<T>(j: &str)
where
    T: DeserializeOwned + DeserializeFromValue<MyError> + PartialEq + std::fmt::Debug,
{
    let json: Value = serde_json::from_str(j).unwrap();

    let actual_serde: Result<T, _> = serde_json::from_str(j);
    let actual_jayson: Result<T, _> = jayson::deserialize(json);

    match (actual_serde, actual_jayson) {
        (Ok(actual_serde), Ok(actual_jayson)) => {
            assert_eq!(actual_jayson, actual_serde);
        }
        (Err(_), Err(_)) => {}
        (Ok(_), Err(_)) => panic!("jayson fails to deserialize but serde does not"),
        (Err(_), Ok(_)) => panic!("serde fails to deserialize but jayson does not"),
    }
}

#[track_caller]
fn assert_error_matches<T, E>(j: &str, expected: E)
where
    E: DeserializeError + PartialEq + std::fmt::Debug,
    T: DeserializeFromValue<E> + std::fmt::Debug,
{
    let json: Value = serde_json::from_str(j).unwrap();
    let actual: E = jayson::deserialize::<T, _, _>(json).unwrap_err();

    assert_eq!(actual, expected);
}
#[track_caller]
fn assert_ok_matches<T, E>(j: &str, expected: T)
where
    E: DeserializeError + PartialEq + std::fmt::Debug,
    T: DeserializeFromValue<E> + std::fmt::Debug + PartialEq,
{
    let json: Value = serde_json::from_str(j).unwrap();
    let actual: T = jayson::deserialize::<T, _, E>(json).unwrap();

    assert_eq!(actual, expected);
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

    // struct with renamed field, roundtrip
    compare_with_serde_roundtrip(RenamedAllFieldsCamelCaseEnum::SomeField { my_field: true });

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

    // struct default attribute using Default trait, missing field
    compare_with_serde::<StructWithTraitDefaultAttr>(r#"{ }"#);

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

    // struct with renamed field, roundtrip
    compare_with_serde_roundtrip(StructWithRenamedField { x: true });

    // struct with renamed field and rename_all rule, roundtrip
    compare_with_serde_roundtrip(StructWithRenamedFieldAndRenameAll { x: true });
    assert_ok_matches(
        r#"{ "renamed_field": true }"#,
        StructWithRenamedFieldAndRenameAll { x: true },
    );

    // struct with deny_unknown_fields, with unknown fields
    compare_with_serde::<StructDenyUnknownFields>(
        r#"{
            "x": true,
            "y": 8
        }
        "#,
    );

    // struct with deny_unknown_fields, roundtrip
    compare_with_serde_roundtrip(StructDenyUnknownFields { x: true });

    // enum with deny_unknown_fields, with unknown fields
    compare_with_serde::<EnumDenyUnknownFields>(
        r#"{
            "t": "SomeField",
            "my_field": true,
            "other": true
        }
        "#,
    );

    // enum with deny_unknown_fields, missing tag
    compare_with_serde::<EnumDenyUnknownFields>(
        r#"{
            "my_field": true,
            "other": true
        }
        "#,
    );

    // enum with deny_unknown_fields, roundtrip 1
    compare_with_serde_roundtrip(EnumDenyUnknownFields::SomeField { my_field: true });

    // enum with deny_unknown_fields, roundtrip 2
    compare_with_serde_roundtrip(EnumDenyUnknownFields::Other {
        my_field: true,
        y: 8,
    });

    // struct with deny_unknown_fields with custom error function
    compare_with_serde::<StructDenyUnknownFieldsCustom>(
        r#"{
            "x": true,
            "y": 8
        }
        "#,
    );

    // struct with deny_unknown_fields with custom error function
    // assert error value is correct

    assert_error_matches::<StructDenyUnknownFieldsCustom, MyError>(
        r#"{
            "x": true,
            "y": 8
        }
        "#,
        unknown_field_error("y", &[], ValuePointerRef::Origin),
    );

    // struct with deny_unknown_fields with custom error function
    compare_with_serde::<EnumDenyUnknownFieldsCustom>(
        r#"{
            "t": "SomeField",
            "my_field": true,
            "other": true
        }
        "#,
    );

    // enum with deny_unknown_fields with custom error function, error check
    assert_error_matches::<EnumDenyUnknownFieldsCustom, MyError>(
        r#"{
            "t": "SomeField",
            "my_field": true,
            "other": true
        }
        "#,
        unknown_field_error("other", &[], ValuePointerRef::Origin),
    );

    // struct with custom missing field error, error check 1
    assert_error_matches::<StructMissingFieldError, MyError>(
        r#"{
            "y": true
        }
        "#,
        MyError::MissingField("lol".to_string()),
    );
    // struct with custom missing field error, error check 2
    assert_error_matches::<StructMissingFieldError, MyError>(
        r#"{
            "x": true
        }
        "#,
        MyError::CustomMissingField(1),
    );

    // enum with custom missing field error, error check 1
    assert_error_matches::<EnumMissingFieldError, MyError>(
        r#"{
            "t": "A"
        }
        "#,
        MyError::CustomMissingField(0),
    );

    // enum with custom missing field error, error check 2
    assert_error_matches::<EnumMissingFieldError, MyError>(
        r#"{
            "t": "B"
        }
        "#,
        MyError::MissingField("x".to_owned()),
    );

    // enum with renamed variants, roundtrip 1
    compare_with_serde_roundtrip(EnumRenamedVariant::A { x: true });
    // enum with renamed variants, roundtrip 2
    compare_with_serde_roundtrip(EnumRenamedVariant::B);

    // enum with renamed field, roundtrip
    compare_with_serde_roundtrip(EnumRenamedField::A { x: true });

    // enum with rename_all variant, roundtrip
    compare_with_serde_roundtrip(EnumRenamedAllVariant::P {
        water_potential: true,
    });

    // generic no bounds, roundtrip
    compare_with_serde_roundtrip(Generic::<EnumRenamedAllVariant> {
        some_field: EnumRenamedAllVariant::P {
            water_potential: true,
        },
    });

    // enum with deny_unknown_fields with custom error function, error check
    assert_error_matches::<EnumDenyUnknownFieldsCustom, MyError>(
        r#"{
            "t": "SomeField",
            "my_field": true,
            "other": true
        }
        "#,
        unknown_field_error("other", &[], ValuePointerRef::Origin),
    );

    assert_ok_matches::<Hello, MyError>("true", Hello::A);

    assert_error_matches::<Validated, MyError>(
        r#"{
            "x": 2,
            "y": 1
        }
        "#,
        MyError::Validation,
    );

    assert_ok_matches::<FieldMap, MyError>(
        r#"{ "some_field": null }"#,
        FieldMap {
            some_field: Some(1),
        },
    );
    assert_ok_matches::<FieldMap, MyError>(
        r#"{  }"#,
        FieldMap {
            some_field: Some(1),
        },
    );
    assert_ok_matches::<FieldMap, MyError>(r#"{ "some_field": 0 }"#, FieldMap { some_field: None });
    assert_ok_matches::<FieldMap, MyError>(
        r#"{ "some_field": 2 }"#,
        FieldMap {
            some_field: Some(2),
        },
    );
}
