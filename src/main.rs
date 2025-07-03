mod config;
mod vk_api;
mod pin;
mod password;

use config::{BotConfig, save_config_secure, load_config_secure, config_exists, delete_config};
use pin::input_pin;
use password::input_password;
use vk_api::*;
use std::{io::{self, Write}};
use reqwest::Client;
use tokio::time::{sleep, Duration};
use std::collections::HashSet;
use rand::Rng;
use rpassword::read_password;

fn prompt_manual() -> BotConfig {
    print!("Введите VK_TOKEN: ");
    io::stdout().flush().unwrap();
    let mut token = String::new();
    io::stdin().read_line(&mut token).unwrap();
    print!("Введите USER_ID: ");
    io::stdout().flush().unwrap();
    let mut user_id = String::new();
    io::stdin().read_line(&mut user_id).unwrap();
    BotConfig {
        vk_token: token.trim().to_string(),
        user_id: user_id.trim().parse().expect("Ошибка USER_ID"),
    }
}

fn prompt_url() -> BotConfig {
    println!("Вставьте URL после авторизации приложения VK:");
    let mut url = String::new();
    io::stdin().read_line(&mut url).unwrap();
    let url = url.trim();
    let hash_part = url.split('#').nth(1).unwrap_or("");
    let mut token = None;
    let mut user_id = None;
    for param in hash_part.split('&') {
        let mut kv = param.splitn(2, '=');
        match (kv.next(), kv.next()) {
            (Some("access_token"), Some(val)) => token = Some(val.to_string()),
            (Some("user_id"), Some(val)) => user_id = Some(val.to_string()),
            _ => {}
        }
    }
    let token = token.expect("Не найден VK_TOKEN");
    let user_id = if let Some(id) = user_id {
        id
    } else {
        print!("Введите USER_ID: ");
        io::stdout().flush().unwrap();
        let mut id = String::new();
        io::stdin().read_line(&mut id).unwrap();
        id.trim().to_string()
    };
    BotConfig {
        vk_token: token,
        user_id: user_id.parse().expect("Ошибка USER_ID"),
    }
}

fn prompt_menu() -> BotConfig {
    println!("1. Авто: Вставить адрес после авторизации приложения VK");
    println!("2. Ввести вручную VK_TOKEN и USER_ID");
    print!("Выберите способ (1/2): ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    match choice.trim() {
        "1" => prompt_url(),
        "2" => prompt_manual(),
        _ => {
            println!("Некорректный выбор.");
            prompt_menu()
        }
    }
}

fn ask_reset() -> bool {
    println!("Хотите сбросить пароль и PIN-код? (y/n): ");
    let mut ans = String::new();
    io::stdin().read_line(&mut ans).unwrap();
    ans.trim().eq_ignore_ascii_case("y")
}

async fn bot_run(cfg: BotConfig) -> Result<(), String> {
    let client = Client::new();
    let mut invited: HashSet<u64> = HashSet::new();
    let mut last_msg_id = 0;
    let delay_min = 8;
    let delay_max = 30;
    let mut rng = rand::thread_rng();

    println!("🔁 Ожидание команды !invite...");

    loop {
        let dialogs = get_conversations(&client, &cfg.vk_token).await.map_err(|e| e.to_string())?;
        for d in dialogs {
            if d.last_message.text.trim().to_lowercase() == "!invite"
                && d.last_message.from_id == cfg.user_id as i64
                && d.last_message.id != last_msg_id
            {
                last_msg_id = d.last_message.id;
                let chat_id = (d.last_message.peer_id - 2000000000) as u64;
                println!("📥 Команда найдена! Чат: {}", chat_id);

                let friends = get_friends(&client, &cfg.vk_token).await.map_err(|e| e.to_string())?;
                println!("👥 Друзей найдено: {}", friends.len());

                for user_id in friends {
                    if !invited.contains(&user_id) {
                        let delay = rng.gen_range(delay_min..=delay_max);
                        sleep(Duration::from_millis(delay)).await;
                        add_user_to_chat(&client, &cfg.vk_token, chat_id, user_id).await;
                        invited.insert(user_id);
                    }
                }
                println!("✅ Все приглашения отправлены.");
            }
        }
        sleep(Duration::from_secs(2)).await;
    }
}

#[tokio::main]
async fn main() {
    let mut fail_count = 0;
    let mut config: Option<BotConfig> = None;

    if config_exists() {
        loop {
            use rpassword::read_password;

            print!("Введите пароль: ");
            io::stdout().flush().unwrap();
            let password = read_password().unwrap();

            print!("Введите PIN-код: ");
            io::stdout().flush().unwrap();
            let pin = read_password().unwrap();

            match load_config_secure(password.trim(), pin.trim()) {
                Ok(cfg) => {
                    config = Some(cfg);
                    break;
                }
                Err(_) => {
                    fail_count += 1;
                    if fail_count >= 5 {
                        println!("Превышено число попыток. Все данные сброшены.");
                        delete_config();
                        config = Some(prompt_menu());
                        let password = input_password("Придумайте пароль: ", &config.as_ref().unwrap().vk_token, &config.as_ref().unwrap().user_id.to_string());
                        let pin = input_pin("Придумайте PIN-код: ");
                        save_config_secure(config.as_ref().unwrap(), &password, &pin).expect("Ошибка сохранения!");
                        break;
                    } else if fail_count >= 2 {
                        println!("Неверный пароль или PIN. Попытка {}/5.", fail_count);
                        if ask_reset() {
                            delete_config();
                            config = Some(prompt_menu());
                            let password = input_password("Придумайте пароль: ", &config.as_ref().unwrap().vk_token, &config.as_ref().unwrap().user_id.to_string());
                            let pin = input_pin("Придумайте PIN-код: ");
                            save_config_secure(config.as_ref().unwrap(), &password, &pin).expect("Ошибка сохранения!");
                            break;
                        }
                    } else {
                        println!("Неверный пароль или PIN. Попытка {}/5.", fail_count);
                    }
                }
            }
        }
    } else {
        config = Some(prompt_menu());
        let password = input_password("Придумайте пароль: ", &config.as_ref().unwrap().vk_token, &config.as_ref().unwrap().user_id.to_string());
        let pin = input_pin("Придумайте PIN-код: ");
        save_config_secure(config.as_ref().unwrap(), &password, &pin).expect("Ошибка сохранения!");
    }

    if let Some(cfg) = config {
        if let Err(e) = bot_run(cfg).await {
            println!("‼️ Ошибка: {}", e);
            std::fs::write("last_error.log", e).ok();
        }
    }
}