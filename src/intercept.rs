use std::{ops::Deref, sync::Mutex};

use teloxide::{
    payloads::{
        DeleteMessage, EditMessageReplyMarkup, EditMessageText, EditMessageTextSetters,
        PinChatMessage, SendMessage, SendMessageSetters,
    },
    requests::{JsonRequest, Request},
    types::{InlineKeyboardMarkup, Message, ParseMode, ReplyMarkup},
    RequestError,
};

use crate::test_utils::{
    escape_control_characters, make_bot_string, TEST_USER_FIRST_NAME, TEST_USER_LAST_NAME,
    TEST_USER_NAME,
};

pub static LAST_SENT_MESSAGE: Mutex<Option<Message>> = Mutex::new(None);
// Mutex allows us to have a global variable that you can modify. It restricts the code to only one
// access at a time, to avoid race conditions. To use that mutex, you need to call lock().unwrap()
// If the code panics out while using the lock, you will get a PoisonError if you try to access it again

pub fn make_bot_message(
    text: &str,
    chat_id: &str,
    reply_markup: Option<InlineKeyboardMarkup>,
) -> Message {
    // Again, very very ugly, but it works
    let message_str = format!(
        r#"{{"message_id":{message_id},"message_thread_id":null,"date":1234567890,"chat":{{"id":{chat_id},"type":"private","username":"{username}","first_name":"{first_name}","last_name":"{last_name}","bio":null,"has_private_forwards":null,"has_restricted_voice_and_video_messages":null,"emoji_status_custom_emoji_id":null}},"via_bot":null,"from":{bot},"text":"{message_text}","entities":[], "reply_markup":{reply_markup}, "is_topic_message":false,"is_automatic_forward":false,"has_protected_content":false}}"#,
        message_id = 1,
        username = TEST_USER_NAME,
        first_name = TEST_USER_FIRST_NAME,
        last_name = TEST_USER_LAST_NAME,
        chat_id = chat_id,
        message_text = escape_control_characters(text),
        reply_markup = serde_json::to_string(&reply_markup).unwrap(),
        bot = make_bot_string()
    );
    // Maybe you need to make a different case for group/supergroup/gigagroup/channel chats, but i don't think there is a
    // lot of uses for the chat field on already sent message
    let message: Message = serde_json::from_str(&message_str).unwrap();
    message
}

pub trait TestingInterceptAndReturnMessage {
    // This trait just
    // 1. Intercepts the request if it is a test environment, and "sends" it, returning a message
    //    that would've been returned by telegram, without actually sending it to telegram
    // 2. Saves the last sent message in LAST_SENT_MESSAGE for testing purposes
    // !!! IT HAS TO BE THE LAST TRAIT THAT IS CALLED !!!
    async fn intercept(self) -> Result<Message, RequestError>;
}

pub trait TestingIntercept {
    // And this is the "lazy" version of the upper trait, it doesn't return anything, so it is
    // easier to write, but you can't test the output of such requests. In a perfect world this
    // trait would be gone, but im too lazy to implement a fake message for every possible request.
    // If you want - you can contribute your implementation
    // !!! IT HAS TO BE THE LAST TRAIT THAT IS CALLED !!!
    async fn intercept(self) -> Result<(), RequestError>;
}

impl TestingInterceptAndReturnMessage for JsonRequest<SendMessage> {
    async fn intercept(self) -> Result<Message, RequestError> {
        if cfg!(test) {
            let req = self.deref(); // Get an actual request object

            let reply_markup = match req.reply_markup.clone() {
                // Since we are trying to get a
                // message, we need only inline keyboard, as others aren't shown in a message
                // return type
                Some(keyboard) => match keyboard {
                    ReplyMarkup::InlineKeyboard(inline_keyboard) => Some(inline_keyboard),
                    _ => None,
                },
                None => None,
            };

            let message = make_bot_message(
                req.text.as_str(),
                &req.chat_id.to_string().as_str(),
                reply_markup,
            );

            *LAST_SENT_MESSAGE.lock().unwrap() = Some(message.clone());

            return Ok(message);
        }

        self.parse_mode(ParseMode::Html).send().await
    }
}

impl TestingInterceptAndReturnMessage for JsonRequest<EditMessageText> {
    async fn intercept(self) -> Result<Message, RequestError> {
        if cfg!(test) {
            let req = self.deref();
            let message = make_bot_message(
                req.text.as_str(),
                &req.chat_id.to_string().as_str(),
                req.reply_markup.clone(),
            );

            *LAST_SENT_MESSAGE.lock().unwrap() = Some(message.clone());
            return Ok(message);
        }
        self.parse_mode(ParseMode::Html).send().await
    }
}

/*
    If you want to make requests into ones that actually return messages,
    follow the same steps as in test_utils.rs, just print not the user message, but the bot one.
    If you want to implement just a TestingIntercept (so no output), use the examples below,
    just replace the JsonRequest into the needed type
*/

impl TestingIntercept for JsonRequest<DeleteMessage> {
    async fn intercept(self) -> Result<(), RequestError> {
        if cfg!(test) {
            return Ok(());
        }
        self.send().await?;
        Ok(())
    }
}

impl TestingIntercept for JsonRequest<EditMessageReplyMarkup> {
    async fn intercept(self) -> Result<(), RequestError> {
        if cfg!(test) {
            return Ok(());
        }
        self.send().await?;
        Ok(())
    }
}

impl TestingIntercept for JsonRequest<PinChatMessage> {
    async fn intercept(self) -> Result<(), RequestError> {
        if cfg!(test) {
            return Ok(());
        }
        self.send().await?;
        Ok(())
    }
}
