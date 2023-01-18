use cruet::Inflector;
use term_painter::{
    Color::{Green, Red, White, Yellow},
    ToStyle,
};

pub static mut VERBOSITY: u64 = 1;

macro_rules! decl_log {
    ($level:ident, $color:ident, $debug_level:literal) => {
        pub fn $level<S: AsRef<str>>(message: S) {
            unsafe {
                if VERBOSITY < $debug_level {
                    return;
                }
            }

            $color.with(|| {
                println!(
                    "[{}]: {}",
                    stringify!($level).to_title_case(),
                    message.as_ref()
                )
            });
        }
    };
}

decl_log!(info, White, 1);
decl_log!(debug, Yellow, 3);
decl_log!(error, Red, 2);
decl_log!(success, Green, 2);
