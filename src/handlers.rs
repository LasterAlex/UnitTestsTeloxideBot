use crate::{
    intercept::{TestingIntercept, TestingInterceptAndReturnMessage},
    text, HandlerResult, MyDialogue, State,
};
use teloxide::{
    dispatching::dialogue::GetChatId,
    macros::BotCommands,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum StartCommand {
    #[command()]
    Start,
}

/*
    Just some simple example handlers to test
*/

pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let keyboard = InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback("Add", "add"),
        InlineKeyboardButton::callback("Subtract", "subtract"),
    ]]);
    bot.send_message(msg.chat.id, text::WHAT_DO_YOU_WANT)
        .reply_markup(keyboard)
        .intercept()
        .await?;
    dialogue.update(State::WhatDoYouWant).await?;
    Ok(())
}

pub async fn what_is_the_first_number(
    bot: Bot,
    dialogue: MyDialogue,
    call: CallbackQuery,
) -> HandlerResult {
    let chat_id = call.clone().chat_id().unwrap();
    bot.edit_message_reply_markup(chat_id, call.message.unwrap().id)
        .intercept()
        .await?;
    bot.send_message(chat_id, text::ENTER_THE_FIRST_NUMBER)
        .intercept()
        .await?;
    dialogue
        .update(State::GetFirstNumber {
            operation: call.data.unwrap(),
        })
        .await?;
    Ok(())
}

pub async fn what_is_the_second_number(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state_data: String,
) -> HandlerResult {
    let message_text = match message.text() {
        // Just extracting the text from the message
        Some(text) => text,
        None => {
            bot.send_message(message.chat.id, text::PLEASE_SEND_TEXT)
                .intercept()
                .await?;
            return Ok(());
        }
    };
    let first_number = match message_text.parse::<i32>() {
        // And then parsing it
        Ok(number) => number,
        Err(_) => {
            bot.send_message(message.chat.id, text::PLEASE_ENTER_A_NUMBER)
                .intercept()
                .await?;
            return Ok(());
        }
    };
    bot.send_message(message.chat.id, text::ENTER_THE_SECOND_NUMBER)
        .intercept()
        .await?;
    dialogue
        .update(State::GetSecondNumber {
            first_number,
            operation: state_data,
        })
        .await?;
    Ok(())
}

pub async fn get_result(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state_data: (i32, String),
) -> HandlerResult {
    let message_text = match message.text() {
        // Who cares about DRY anyway
        Some(text) => text,
        None => {
            bot.send_message(message.chat.id, text::PLEASE_SEND_TEXT)
                .intercept()
                .await?;
            return Ok(());
        }
    };
    let second_number = match message_text.parse::<i32>() {
        Ok(number) => number,
        Err(_) => {
            bot.send_message(message.chat.id, text::PLEASE_ENTER_A_NUMBER)
                .intercept()
                .await?;
            return Ok(());
        }
    };

    let (first_number, operation) = state_data;
    let result = match operation.as_str() {
        "add" => first_number + second_number,
        "subtract" => first_number - second_number,
        _ => unreachable!(),
    };

    bot.send_message(
        message.chat.id,
        text::YOUR_RESULT.to_owned() + result.to_string().as_str(),
    )
    .intercept()
    .await?;
    dialogue.update(State::default()).await?;
    Ok(())
}

#[cfg(test)] // This prevents it from compiling in non-test mode
mod tests {
    use crate::test_utils::*;

    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial] // Because everything is async, and we are testing the same user, we need this.
              // Otherwise, there is a lot of race conditions.
    async fn test_start() {
        full_reset_test_user().await; // We need to reset everything, because i have encountered
                                      // SUCH horrible bugs due to not resetting, they were IMPOSSIBLE to debug.
                                      // Just add this to the start of every test and you will be ok.
        let state = State::default();
        let message = make_message("/start", true, ChatType::Private);
        // Create the environment that the test needs. In this case, it is just the state and the
        // message update, but if there is some db data or something, you need to redifine it here.

        run_update(make_message_update(message), state)
            .await
            .unwrap();
        // Actually running the update

        check_the_state_and_text(State::WhatDoYouWant, text::WHAT_DO_YOU_WANT).await;
        // Checking the state and the text. If you need to check more stuff,
        // like db or reply markup - add it here.
    }

    #[tokio::test]
    #[serial]
    async fn test_what_is_the_first_number() {
        full_reset_test_user().await;
        // Everything else is the same
        let state = State::WhatDoYouWant;
        let call = make_callback_query("add", ChatType::Private);
        run_update(make_callback_query_update(call), state)
            .await
            .unwrap();

        check_the_state_and_text(
            State::GetFirstNumber {
                operation: "add".to_string(),
            },
            text::ENTER_THE_FIRST_NUMBER,
        )
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn test_message_errors() {
        full_reset_test_user().await;
        let state = State::GetFirstNumber {
            operation: "add".to_string(),
        };
        let message = make_message("not a number", false, ChatType::Private);
        run_update(make_message_update(message), state)
            .await
            .unwrap();

        check_the_state_and_text(
            State::GetFirstNumber {
                // Technically, this is redundant, and you can check just the text
                operation: "add".to_string(),
            },
            text::PLEASE_ENTER_A_NUMBER,
        )
        .await;

        // Because we use the same state, we can just add the next test to the same function!
        // Dont overuse it though, it can get confusing really fast
        let message = make_photo(false, ChatType::Private);
        run_update(make_message_update(message), get_state().await) // This is using the state that is
            // stored in redis, in this case - the same state as before, but if the previous test changed
            // the state, the new updated state will be returned by get_state() function.
            .await
            .unwrap();
        check_the_state_and_text(
            State::GetFirstNumber {
                operation: "add".to_string(),
            },
            text::PLEASE_SEND_TEXT,
        )
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn test_what_is_the_second_number() {
        full_reset_test_user().await;
        let state = State::GetFirstNumber {
            operation: "add".to_string(),
        };
        let message = make_message("1", false, ChatType::Private);
        run_update(make_message_update(message), state)
            .await
            .unwrap();

        check_the_state_and_text(
            State::GetSecondNumber {
                first_number: 1,
                operation: "add".to_string(),
            },
            text::ENTER_THE_SECOND_NUMBER,
        )
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn test_add_result() {
        full_reset_test_user().await;
        let state = State::GetSecondNumber {
            first_number: 1,
            operation: "add".to_string(),
        };
        let message = make_message("2", false, ChatType::Private);
        run_update(make_message_update(message), state)
            .await
            .unwrap();

        check_the_state_and_text(State::Start, &(text::YOUR_RESULT.to_owned() + "3")).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_subtract_result() {
        let state = State::GetSecondNumber {
            first_number: 1,
            operation: "subtract".to_string(),
        };
        let message = make_message("2", false, ChatType::Private);
        run_update(make_message_update(message), state)
            .await
            .unwrap();

        check_the_state_and_text(State::Start, &(text::YOUR_RESULT.to_owned() + "-1")).await;
    }
}
