use term_painter::{
    Color::{Green, Red, White, Yellow},
    ToStyle,
};

pub static mut VERBOSITY: u64 = 1;

pub fn error(message: &str) {
    Red.with(|| {
        println!("[Error]: {}", message);
    });
}

pub fn success(message: &str) {
    unsafe {
        if VERBOSITY < 2 {
            return;
        }
    }

    Green.with(|| {
        println!("[Success]: {}", message);
    });
}

pub fn info(message: &str) {
    unsafe {
        if VERBOSITY < 1 {
            return;
        }
    }

    White.with(|| {
        println!("[Info]: {}", message);
    });
}

pub fn debug(message: &str) {
    unsafe {
        if VERBOSITY < 3 {
            return;
        }
    }

    Yellow.with(|| {
        println!("[Debug]: {}", message);
    });
}
