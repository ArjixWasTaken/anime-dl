#![allow(non_upper_case_globals)]

use crate::types::Provider;
mod animeonsen;

pub fn search(provider_name: &str, query: &str) {
    if let Some(result) = match provider_name {
        "animeonsen" => Some(animeonsen::search(&query)),
        _ => None,
    } {
        println!(
            "[Search] [provider={}] [query=\"{}\"]: {:#?}",
            provider_name, query, result
        );
    } else {
        println!("No provider found with name: {}", provider_name);
    }
}
