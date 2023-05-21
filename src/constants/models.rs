use once_cell::sync::Lazy;

pub static MODELS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        String::from("🤖 GPT 4.0"),
        String::from("🐇 GPT 3.5 Turbo"),
        String::from("💫 Claude v1"),
    ]
});
