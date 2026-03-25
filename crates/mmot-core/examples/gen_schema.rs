use schemars::schema_for;
use mmot_core::schema::Scene;

fn main() {
    let schema = schema_for!(Scene);
    let json = serde_json::to_string_pretty(&schema).expect("failed to serialize schema");
    println!("{json}");
}
