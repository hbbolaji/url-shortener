const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn base62_encode(mut num: u64) -> String {
    if num == 0 {
        return "0".to_string();
    }

    let mut result = String::new();
    while num > 0 {
        let remainder = (num % 62) as usize;
        result.push(ALPHABET[remainder] as char);
        num /= 62;
    }

    result
}

pub fn base62_decode(encoded: &str) -> Option<u64> {
    let mut result: u64 = 0;
    for c in encoded.chars() {
        let value = match c {
            '0'..='9' => (c as u64 - '0' as u64) * 62,
            'A'..='Z' => c as u64 - 'A' as u64 + 10,
            'a'..='z' => c as u64 - 'a' as u64 + 36,
            _ => return None,
        };

        result += value;
    }
    Some(result)
}
