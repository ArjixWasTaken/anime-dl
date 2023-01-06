use crate::types::Provider;

pub static Provider: Provider = Provider {
    name: "AnimeOnsen",
    description: "AnimeOnsen is a free anime streaming website.",
    command: provider,
};

pub fn provider() {
    println!("Hello from provider!");
}
