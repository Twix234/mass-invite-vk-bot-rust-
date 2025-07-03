use std::io::{self, Write};
use rpassword::read_password;
pub fn is_pin_complex(pin: &str) -> bool {
    if pin.len() < 4 || pin.len() > 8 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let banned = [
        "0000", "1111", "2222", "3333", "4444", "5555", "6666", "7777", "8888", "9999",
        "1234", "4321", "1122", "1212", "2112", "1004", "2000"
    ];
    if banned.contains(&pin) { return false; }
    let chars: Vec<char> = pin.chars().collect();
    let all_same = chars.iter().all(|&c| c == chars[0]);
    let sequential = chars.windows(2).all(|w| (w[1] as u8) == (w[0] as u8) + 1);
    let reverse_seq = chars.windows(2).all(|w| (w[0] as u8) == (w[1] as u8) + 1);
    !(all_same || sequential || reverse_seq)
}
pub fn input_pin(prompt: &str) -> String {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let pin = read_password().unwrap();
        if is_pin_complex(&pin) {
            print!("Повторите PIN-код: ");
            io::stdout().flush().unwrap();
            let pin2 = read_password().unwrap();
            if pin == pin2 {
                return pin;
            } else {
                println!("\nPIN-коды не совпадают!");
            }
        } else {
            println!("\nPIN слишком простой или некорректный. Попробуйте другой.");
        }
    }
}