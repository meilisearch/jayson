use jayson::Jayson;

#[derive(Jayson)]
enum Enum<const T: i32> {
    Variant,
}

fn main() {}
