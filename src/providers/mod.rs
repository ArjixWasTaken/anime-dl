#![allow(non_upper_case_globals)]

use crate::types::Provider;
mod animeonsen;

static providers: &'static [&'static Provider; 1] = &[&animeonsen::Provider];

pub fn get_providers() -> &'static [&'static Provider] {
    return providers;
}
