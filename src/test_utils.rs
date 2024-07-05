#![allow(dead_code)]
use teloxide::types::UpdateKind;

use teloxide::{
    dptree::deps,
    prelude::*,
    types::{ChatId, Me},
};

use crate::intercept::LAST_SENT_MESSAGE;
use crate::{bot_schema, get_bot_storage, MyDialogue, State};

/*
    Constants
*/
pub const TEST_USER_ID: i64 = 123456789;
pub const TEST_GROUP_ID: i64 = -123456789;
pub const TEST_UPDATE_ID: i32 = 1;
pub const TEST_USER_NAME: &str = "test_user";
pub const TEST_USER_FIRST_NAME: &str = "Test";
pub const TEST_USER_LAST_NAME: &str = "User";

/*
    The main function
*/
pub async fn run_update(
    update: Update,
    state: State,
) -> Result<MyDialogue, Box<dyn std::error::Error>> {
    let bot = get_bot();

    let me: Me = serde_json::from_str(&make_bot_string())?;

    let storage = get_bot_storage().await;
    let dialogue = MyDialogue::new(storage.clone(), ChatId(TEST_USER_ID));
    dialogue
        .update(state)
        .await
        .expect("Failed to update dialogue");

    let dependencies = deps![me, bot, storage, update];
    // If you pass in 'update' as a dependency, it will handle it like a normal update. Very useful
    // to know!

    let result = bot_schema::schema().dispatch(dependencies).await;
    // This makes the update go through the schema
    if let ControlFlow::Break(result) = result {
        // If it returned `ControlFlow::Break`, everything is fine, but we need to check, if the
        // handler didn't error out
        assert!(result.is_ok(), "Error in handler: {:?}", result);
    } else {
        panic!("Unhandled update!");
    }

    Ok(dialogue)
}

/*
    Helper functions
*/
pub fn get_bot_id() -> i64 {
    // Every token starts with a bot id
    let token = dotenvy::var("TELOXIDE_TOKEN").unwrap();
    let parts: Vec<&str> = token.split(':').collect();
    parts[0].parse::<i64>().unwrap()
}

pub fn make_bot_string() -> String {
    format!(
        r#"{{"id":{bot_id},"is_bot":true,"first_name":"Test","last_name":"Bot","username":"test_bot","language_code":"en","can_join_groups":false,"can_read_all_group_messages":false,"supports_inline_queries":true}}"#,
        bot_id = get_bot_id()
    )
}

pub fn escape_control_characters(input: &str) -> String {
    // Serde doesn't love these characters
    input
        .chars()
        .flat_map(|c| match c {
            '\n' => "\\n".chars().collect::<Vec<char>>(),
            '\t' => "\\t".chars().collect::<Vec<char>>(),
            '\\' => "\\\\".chars().collect::<Vec<char>>(),
            '\"' => "\\\"".chars().collect::<Vec<char>>(),
            '\'' => "\\'".chars().collect::<Vec<char>>(),
            _ => vec![c],
        })
        .collect()
}

pub fn get_bot() -> Bot {
    dotenvy::dotenv().ok();
    Bot::from_env()
}

pub async fn get_dialogue() -> MyDialogue {
    let dialogue = MyDialogue::new(get_bot_storage().await, ChatId(TEST_USER_ID));
    return dialogue;
}

pub async fn get_state() -> State {
    let dialogue = get_dialogue().await;
    return dialogue.get().await.unwrap().unwrap();
}

pub async fn full_reset_test_user() {
    // If user has more attached to him, reset it here too if you want
    let dialogue = get_dialogue().await;
    dialogue
        .update(State::default())
        .await
        .expect("Failed to update dialogue");
    *LAST_SENT_MESSAGE.lock().unwrap() = None;
}

/*
    Checking functions (you can add more of them, test just the state, just the text, etc)
*/

pub async fn check_the_state_and_text(state: State, text: &str) {
    let lock = LAST_SENT_MESSAGE.lock().unwrap().clone();
    // Without the clone, if something fails, the mutex will be poisoned, making a lot of
    // PoisonErrors in tests, which is bad

    let last_sent_message = lock.clone().unwrap();
    assert_eq!(last_sent_message.text().unwrap(), text);
    assert_eq!(get_state().await, state);
}

/*
    User input functions
*/

pub enum ChatType {
    Private,
    Group,
    Supergroup,
    Gigagroup,
    Channel,
}

pub fn make_chat_string(chat_type: ChatType) -> String {
    match chat_type {
        ChatType::Private => format!(
            r#"{{"id":{user_id},"type":"private","username":"{username}","first_name":"{first_name}","last_name":"{last_name}","bio":null,"has_private_forwards":null,"has_restricted_voice_and_video_messages":null,"emoji_status_custom_emoji_id":null}}"#,
            user_id = TEST_USER_ID,
            username = TEST_USER_NAME,
            first_name = TEST_USER_FIRST_NAME,
            last_name = TEST_USER_LAST_NAME
        ),
        ChatType::Group => format!(
            r#"{{"id":{chat_id},"type":"group","title":"Test Group"}}"#,
            chat_id = TEST_GROUP_ID
        ),
        ChatType::Supergroup => format!(
            r#"{{"id":{chat_id},"type":"supergroup","title":"Test Group"}}"#,
            chat_id = TEST_GROUP_ID
        ),
        ChatType::Gigagroup => format!(
            r#"{{"id":{chat_id},"type":"gigagroup","title":"Test Group"}}"#,
            chat_id = TEST_GROUP_ID
        ),
        ChatType::Channel => format!(
            r#"{{"id":{chat_id},"type":"channel","title":"Test Channel"}}"#,
            chat_id = TEST_GROUP_ID
        ),
    }
}

pub fn make_from_string() -> String {
    // To make the raw strings just a little more readable
    format!(
        r#"{{"id":{user_id},"is_bot":false,"first_name":"{first_name}","last_name":"{last_name}","username":"{username}","language_code":"en"}}"#,
        user_id = TEST_USER_ID,
        username = TEST_USER_NAME,
        first_name = TEST_USER_FIRST_NAME,
        last_name = TEST_USER_LAST_NAME
    )
}

pub fn make_message(text: &str, is_command: bool, chat_type: ChatType) -> Message {
    // Commands are a bit special
    let command = match is_command {
        true => format!(
            r#"{{"type":"bot_command","offset":0,"length":{length}}}"#,
            length = text.len()
        ),
        false => "".to_string(),
    };
    // Veeeeeeeeeeeery ugly, but a lot easier than making a full object, and it works perfectly
    let message_str = format!(
        r#"{{"message_id":{message_id},"message_thread_id":null,"date":1234567890,"chat":{chat},"via_bot":null,"from":{from},"text":"{message_text}","entities":[{command}],"is_topic_message":false,"is_automatic_forward":false,"has_protected_content":false}}"#,
        message_text = escape_control_characters(text),
        chat = make_chat_string(chat_type),
        from = make_from_string(),
        message_id = 1,
        command = command
    );
    // Because Message implements serde::Deserialize, we can deserialize it from a string
    let message: Message = serde_json::from_str(message_str.as_str()).unwrap();
    return message;
}

pub fn make_callback_query(data: &str, chat_type: ChatType) -> CallbackQuery {
    let callback_query_str = format!(
        r#"{{"id":"{callback_id}","from":{from},"message":{{"message_id":{last_message_id},"message_thread_id":null,"date":1234567890,"chat":{chat},"via_bot":null,"from":{bot},"text":"{last_message_text}","entities":[],"is_topic_message":false,"is_automatic_forward":false,"has_protected_content":false}},"chat_instance":"{chat_instance}","data":"{callback_data}"}}"#,
        chat = make_chat_string(chat_type),
        from = make_from_string(),
        callback_id = 1,
        last_message_id = 1,
        last_message_text = "text", // Just in case you need it
        bot = make_bot_string(),
        chat_instance = 1,
        callback_data = data
    );
    let callback_query: CallbackQuery = serde_json::from_str(callback_query_str.as_str()).unwrap();
    return callback_query;
}

pub fn make_photo(is_media_group: bool, chat_type: ChatType) -> Message {
    let message_str = format!(
        r#"{{"message_id":{message_id},"message_thread_id":null,"date":1234567890,"chat":{chat},"via_bot":null,"from":{from},"photo":[{{"file_id":"1234567890","file_unique_id":"1234567890","file_size":932,"width":90,"height":56}},{{"file_id":"1234567890","file_unique_id":"1234567890","file_size":13483,"width":320,"height":200}},{{"file_id":"1234567890","file_unique_id":"1234567890","file_size":60882,"width":800,"height":500}},{{"file_id":"1234567890","file_unique_id":"1234567890","file_size":116270,"width":1280,"height":800}}],"caption_entities":[]{media_group_id},"is_topic_message":false,"is_automatic_forward":false,"has_protected_content":false}}"#,
        chat = make_chat_string(chat_type),
        from = make_from_string(),
        message_id = 1,
        media_group_id = if is_media_group {
            ",\"media_group_id\":\"1\""
        } else {
            ""
        },
    );
    let message: Message = serde_json::from_str(&message_str).unwrap();
    return message;
}

pub fn make_webapp_data(data: &str, chat_type: ChatType) -> Message {
    let message_str = format!(
        "{{\"message_id\":{message_id},\"message_thread_id\":null,\"date\":1234567890,\"chat\":{chat},\"via_bot\":null,\"web_app_data\":{{\"data\":\"{data}\",\"button_text\":\"Test button text\"}}}}",
        message_id = 1,
        chat = make_chat_string(chat_type),
        data = escape_control_characters(data)
    );
    let message: Message = serde_json::from_str(&message_str).unwrap();
    return message;
}

/*
    If you want to add more messages/callbacks/etc:
    1) Make some handler that handles that type of update
    2) Add println!(serde_json::to_string(&what_you_want_to_test).unwrap()) to the handler;
    3) Copy the output and make it into a function like the ones above
    4) Contribute it to here, if you want to. In the perfect example, every message type would've been here
*/

/*
    Making updates funtions
*/

pub fn make_message_update(message: Message) -> Update {
    Update {
        id: TEST_UPDATE_ID,
        kind: UpdateKind::Message(message),
    }
}

pub fn make_callback_query_update(callback_query: CallbackQuery) -> Update {
    Update {
        id: TEST_UPDATE_ID,
        kind: UpdateKind::CallbackQuery(callback_query),
    }
}
