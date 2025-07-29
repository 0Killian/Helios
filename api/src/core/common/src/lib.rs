mod config;

pub use config::*;
use rand::seq::{IndexedRandom, SliceRandom};

#[macro_export]
macro_rules! hashmap {
    ($($key: expr => $value: expr),*) => {
        {
            let mut map = std::collections::HashMap::new();
            $(map.insert($key, $value);)*
            map
        }
    };
}

pub fn generate_token() -> String {
    const LENGTH: usize = 32;
    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const NUMBERS: &[u8] = b"0123456789";
    const SPECIAL_CHARS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
        abcdefghijklmnopqrstuvwxyz\
        0123456789\
        !@#$%^&*()_+-=[]{}|;:,.<>?";

    let mut rng = rand::rng();
    let mut token_chars = Vec::with_capacity(LENGTH);
    token_chars.push(*UPPERCASE.choose(&mut rng).unwrap());
    token_chars.push(*LOWERCASE.choose(&mut rng).unwrap());
    token_chars.push(*NUMBERS.choose(&mut rng).unwrap());
    token_chars.push(*SPECIAL_CHARS.choose(&mut rng).unwrap());
    for _ in 4..LENGTH {
        token_chars.push(*CHARSET.choose(&mut rng).unwrap());
    }

    token_chars.shuffle(&mut rng);

    token_chars.into_iter().map(|c| c as char).collect()
}
