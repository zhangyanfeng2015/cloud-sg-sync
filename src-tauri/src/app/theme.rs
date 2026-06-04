pub fn normalize_theme_accent(raw: &str) -> String {
    match raw {
        "sky" | "cyan" | "emerald" | "lime" | "amber" | "coral" | "grape" | "graphite" => {
            raw.to_string()
        }
        "ocean" => "cyan".into(),
        "teal" => "emerald".into(),
        "sunset" => "amber".into(),
        "rose" => "coral".into(),
        "violet" => "grape".into(),
        "indigo" => "sky".into(),
        "slate" => "graphite".into(),
        _ => "sky".into(),
    }
}
