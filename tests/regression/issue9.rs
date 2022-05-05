use jayson::{json, Error};

#[test]
fn main() {
    let result = json::from_str::<bool, Error>(" true && false ");
    assert!(result.is_err());
}
