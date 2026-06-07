use rand::seq::SliceRandom;
use rand::thread_rng;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789\
                        !@#$%^&*()_+-=[]{}|;:,.<>?";

pub fn generate(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| *CHARSET.choose(&mut rng).unwrap() as char)
        .collect()
}

pub fn generate_12() -> String {
    generate(12)
}
