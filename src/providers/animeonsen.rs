use crate::types::Provider;

pub static Provider: Provider = Provider {
    name: "AnimeOnsen",
    base_url: "https://animeonsen.xyz",
};

pub fn provider() {
    println!("Hello from provider!");
}
