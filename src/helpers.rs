pub fn missing_field_s(field_name: &str) -> String {
    error!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    String::from("")
}

pub fn missing_field_v(field_name: &str) -> Vec<String> {
    error!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    vec![String::from("")]
}
