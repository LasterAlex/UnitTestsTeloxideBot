# An example of unit tests in teloxide

There isn't yet a supported way to make unit tests in teloxide, so, to show how it _can_ be done, I've made this repo.

Although it can be done, it is very clunky and not very beautiful on the inside. But I've tried to make it so that actual tests look neat, so all of the nasty serde stuff is on the inside, and actual tests look something like this:

![image](https://github.com/LasterAlex/UnitTestsTeloxideBot/assets/75775321/9e43c8b7-40ac-4968-9757-a155cb810744)


## What is in the repo?

1) A simple bot that adds or subtracts two entered numbers
2) Readable unit tests of that bot
3) Unreadable insides of test utilities (ok, it's not _that_ bad, but it isn't good)
4) A lot of comments, explaining, how the tests work, to mitigate some of unreadability
5) A filter that resets the redis user state if the branch that he is currently on doesn't exist anymore after an update (no association to tests, just wanted to add it)


## How to run the tests in that bot?

1) Download and start redis-server from your OS of choice [Ubuntu install](https://www.digitalocean.com/community/tutorials/how-to-install-and-secure-redis-on-ubuntu-20-04), [Windows install](https://redis.io/blog/install-redis-windows-11/) [macOS install](https://redis.io/docs/latest/operate/oss_and_stack/install/install-redis/install-redis-on-mac-os/) (if you run anything else, you probably know how to install it)
2) In the terminal, run `git clone git@github.com:LasterAlex/UnitTestsTeloxideBot.git && cd UnitTestsTeloxideBot && cp .example.env .env`
3) Create a `.env` file in the project root directory, following the `.example.env`
4) And then run `cargo test`, the output should look like this:

![image](https://github.com/LasterAlex/UnitTestsTeloxideBot/assets/75775321/d49517d7-4a82-40ae-8a61-7dcfb5d73bba)

Or you can `cargo install cargo-pretty-test` and run `cargo pretty-test` if you prefer

![image](https://github.com/LasterAlex/UnitTestsTeloxideBot/assets/75775321/899f4218-e274-4238-93f8-1829ab0a7870)


## How to integrate tests into an already existing bot?

I can't give a step-by-step guide, every project is different, but i can tell in what general direction you should go.
I've tried to make that example as drop-in as possible in terms of handlers, but you will need to modify them a little bit

1) Clone every file (aside from `main.rs` and `handlers.rs`) and dependency in this repo to yours (you can clean it up into your directories of choosing if you want to) and add the missing `deps![]` and `get_bot_storage` to your `main.rs` file
2) Add the missing fields to your `.env` file
3) Go to your handlers, and add `.intercept()` before every call to send/edit/answer something. e.g:
`bot.send_message(chat_id, text::ENTER_THE_FIRST_NUMBER).await?;`

↓

`bot.send_message(chat_id, text::ENTER_THE_FIRST_NUMBER).intercept().await?;`

You need to add it as a last call before `await?;`, as it intercepts everything and sends it to telegram api!

4) Resolve the import and code problems (rust analyzer and code actions in any IDE/Vim should make it easier)
5) If you see an error like this:

![image](https://github.com/LasterAlex/UnitTestsTeloxideBot/assets/75775321/79ecee1c-1c45-45eb-a178-2b83d84d3913)

It means you need to add a new implementation in the `intercept.rs`. Copy the struct about which the rust compiler complaints, in this case, `JsonRequest<EditMessageReplyMarkup>`, and create an impl, following code comments in `intercept.rs`

6) Check that your code still runs fine
7) Add your tests, following an example in `handlers.rs`
8) Run `cargo test` (or `cargo pretty-test`) in the project root directory, and see, if the tests succeed! And if you see an error like this:

![image](https://github.com/LasterAlex/UnitTestsTeloxideBot/assets/75775321/a54eec8a-5d64-48f8-aa14-b4b7406f5bcf)

Or like this:

![image](https://github.com/LasterAlex/UnitTestsTeloxideBot/assets/75775321/733136f6-75e0-4a1e-8de3-72c558bbf4f3)

You probably forgot to add `.intercept()` to something in that handler


## Where do i ask questions?

[There is an official telegram group](https://t.me/teloxide), and i am a part of it, you can tag me (@laster_alex) to ask questions

Please do it in the group, i want every question to be documented, as anyone else can also have that same question


## How can i contribute?

I don't expect it, but if you want to upgrade or expand this repo, see `CONTRIBUTING.md` for more details
