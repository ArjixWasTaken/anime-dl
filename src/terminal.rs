use term_painter::{
    Color::{Green, Red},
    ToStyle,
};

pub fn error(message: &str) {
    println!("{}", Red.paint(format!("[Error]: {}", message)));
}

pub fn success(message: &str) {
    println!("{}", Green.paint(format!("[Success]: {}", message)));
}
