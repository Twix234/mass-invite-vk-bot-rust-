use serde::Deserialize;
use reqwest::Client;

#[derive(Deserialize)]
pub struct FriendsResponse {
    pub response: FriendsItems,
}

#[derive(Deserialize)]
pub struct FriendsItems {
    pub items: Vec<u64>,
}

#[derive(Deserialize)]
pub struct ConversationsResponse {
    pub response: ConversationsItems,
}

#[derive(Deserialize)]
pub struct ConversationsItems {
    pub items: Vec<DialogItem>,
}

#[derive(Deserialize, Clone)]
pub struct DialogItem {
    pub last_message: LastMessage,
}

#[derive(Deserialize, Clone)]
pub struct LastMessage {
    pub text: String,
    pub from_id: i64,
    pub peer_id: i64,
    pub id: u64,
}

pub async fn get_friends(client: &Client, token: &str) -> anyhow::Result<Vec<u64>> {
    let url = format!(
        "https://api.vk.com/method/friends.get?v=5.131&access_token={}",
        token
    );
    let resp: FriendsResponse = client.get(url).send().await?.json().await?;
    Ok(resp.response.items)
}

pub async fn get_conversations(client: &Client, token: &str) -> anyhow::Result<Vec<DialogItem>> {
    let url = format!(
        "https://api.vk.com/method/messages.getConversations?count=20&v=5.131&access_token={}",
        token
    );
    let resp: ConversationsResponse = client.get(url).send().await?.json().await?;
    Ok(resp.response.items)
}

pub async fn add_user_to_chat(client: &Client, token: &str, chat_id: u64, user_id: u64) -> bool {
    let url = format!(
        "https://api.vk.com/method/messages.addChatUser?chat_id={}&user_id={}&v=5.131&access_token={}",
        chat_id, user_id, token
    );
    match client.get(url).send().await {
        Ok(resp) => {
            let txt = resp.text().await.unwrap_or_default();
            if txt.contains("\"error\"") {
                println!("❌ Ошибка добавления {}: {}", user_id, txt);
                false
            } else {
                println!("✅ Приглашён: {}", user_id);
                true
            }
        },
        Err(e) => {
            println!("❌ Ошибка сети для {}: {}", user_id, e);
            false
        }
    }
}