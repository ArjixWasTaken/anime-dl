use crate::types::Provider;

pub static Provider: Provider = Provider {
    name: "AnimeOnsen",
    base_url: "AnimeOnsen is a free anime streaming website.",
};

pub fn provider() {
    println!("Hello from provider!");
}
