#[macro_use]
extern crate clap;

include!("src/cli.rs");

fn main() {
    // this might fail in CI
    // let mut app = build_cli();
    // app.gen_completions("anime", Shell::Bash, "completions/");
    // app.gen_completions("anime", Shell::Fish, "completions/");
    // app.gen_completions("anime", Shell::Zsh, "completions/");
    // app.gen_completions("anime", Shell::PowerShell, "completions/");
}
