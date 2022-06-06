use jayson::{de::VisitorError, json, Error, Jayson};

#[derive(Jayson)]
#[jayson(error = Error)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[test]
fn main() {
    let result = json::from_str::<Point, Error>(r#"{"x": 1, "y": 2, "z": 3}"#);
    assert!(result.is_ok());
}
