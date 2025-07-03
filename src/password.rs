use std::io::{self, Write};
use rpassword::read_password;

pub fn is_password_complex(password: &str, vk_token: &str, user_id: &str) -> bool {
    password.len() >= 8 &&
    password.chars().any(|c| c.is_ascii_digit()) &&
    password.chars().any(|c| c.is_ascii_uppercase()) &&
    password != vk_token &&
    password != user_id &&
    !["12345678", "password", "qwerty", "11111111"].contains(&password)
}

pub fn input_password(prompt: &str, vk_token: &str, user_id: &str) -> String {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let password = read_password().unwrap();
        if is_password_complex(&password, vk_token, user_id) {
            print!("Повторите пароль: ");
            io::stdout().flush().unwrap();
            let password2 = read_password().unwrap();
            if password == password2 {
                return password;
            } else {
                println!("\nПароли не совпадают!");
            }
        } else {
            println!("\nПароль слишком простой. Минимум 8 символов, хотя бы одна цифра и одна заглавная буква.");
        }
    }
}