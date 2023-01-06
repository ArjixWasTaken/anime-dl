#![allow(dead_code)]

#[derive(Debug)]
pub struct Provider {
    pub name: &'static str,
    pub base_url: &'static str,
}
