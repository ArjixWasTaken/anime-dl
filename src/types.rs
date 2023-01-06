#![allow(dead_code)]

pub type ProviderCommand = fn();

#[derive(Debug)]
pub struct Provider {
    pub name: &'static str,
    pub description: &'static str,
    pub command: ProviderCommand,
}
