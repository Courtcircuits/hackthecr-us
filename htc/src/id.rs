pub fn build_id(name: &str) -> String {
    name.to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}
