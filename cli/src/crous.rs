use macros::{generate_crous_data, generate_crous_enum};

generate_crous_data!("src/data/crous.json");
generate_crous_enum!("src/data/crous.json");
