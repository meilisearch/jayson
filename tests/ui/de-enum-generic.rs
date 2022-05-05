use jayson::Deserialize;

#[derive(Deserialize)]
enum Enum<const T: i32> {
    Variant,
}

fn main() {}
