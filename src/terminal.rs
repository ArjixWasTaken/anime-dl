use cruet::Inflector;
use term_painter::{
    Color::{Green, Red, White, Yellow},
    ToStyle,
};

pub static mut VERBOSITY: u64 = 0;

macro_rules! decl_log {
    ($level:ident, $color:ident, $debug_level:literal) => {
        pub fn $level(message: impl ToString) {
            unsafe {
                if VERBOSITY <= $debug_level {
                    return;
                }
            }

            $color.with(|| {
                println!(
                    "[{}]: {}",
                    stringify!($level).to_title_case(),
                    message.to_string()
                )
            });
        }
    };
}

decl_log!(error, Red, 0);
decl_log!(info, White, 1);
decl_log!(success, Green, 2);
decl_log!(debug, Yellow, 3);
