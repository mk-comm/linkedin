pub struct FullName {
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
}

impl FullName {
    pub fn split_name(full_name: &str) -> FullName {
        let parts: Vec<&str> = full_name.split_whitespace().collect();
        let first_name = parts[0].to_string();
        let last_name = parts[1..].join(" ");
        let full_name = full_name.to_string();
        FullName {
            first_name,
            last_name,
            full_name,
        }
    }
}
