use teloxide::{prelude::*, utils::command::BotCommands};
use teloxide::dispatching::dialogue::{Dialogue, InMemStorage};
use teloxide::types::BotCommand;
use teloxide::dispatching::UpdateHandler;
use teloxide::dispatching::dialogue;
use std::env;
use dotenv::dotenv;
use std::sync::LazyLock;

use crate::google_apis::weather_api::{ get_weater_information, get_weater_information_from_location };

type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

static TELEGRAM_TOKEN: LazyLock<String> = LazyLock::new(|| {
    env::var("TELEGRAM_BOT_TOKEN").expect("TELOXIDE_TOKEN no está definida")
});
static GOOGLE_API_TOKEN: LazyLock<String> = LazyLock::new(|| {
    env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY no está definida")
});

#[derive(Clone, Default, Debug)]
enum State {
    #[default]
    Start,
    WaitingLocation,
    WaitingLocationAttach,
    WaitingProvince {
        location: String,
    },
    WaitingCountry {
        location: String,
        province: String,
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Enable commands in the bot")]
pub enum Command {
    #[command(description = "Start bot 🚀")]
    Start,
    #[command(description = "Get weather forecast 🌤️")]
    GetWeather,
    #[command(description = "Get weather for my location📍")]
    GetWeatherLocation,
    #[command(description = "Cancel operation ❌")]
    Cancel,
    #[command(description = "Help menu 📜")]
    Help,
}

pub async fn run_bot() {
    dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::new(TELEGRAM_TOKEN.clone());

    let commands = Command::bot_commands()
        .iter()
        .map(|cmd| BotCommand::new(cmd.command.clone(), cmd.description.clone()))
        .collect::<Vec<_>>();

    bot.set_my_commands(commands).send().await.expect("The menu could not be set!");

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;
    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Start].endpoint(handle_start_command))
                .branch(case![Command::GetWeather].endpoint(handle_get_weather_command))
                .branch(case![Command::Help].endpoint(handle_help_command))
                .branch(case![Command::GetWeatherLocation].endpoint(handle_get_weather_location_command)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::WaitingLocation].endpoint(dialogue_handler))
        .branch(case![State::WaitingLocationAttach].endpoint(dialogue_handler))
        .branch(case![State::WaitingProvince { location }].endpoint(dialogue_handler))
        .branch(case![State::WaitingCountry { location, province }].endpoint(dialogue_handler));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Cancelling the dialogue.").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn handle_start_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Welcome to the Weather Bot!").await?;
    Ok(())
}

async fn handle_help_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn handle_get_weather_command(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    bot.send_message(msg.chat.id, "What is your location?").await?;
    dialogue.update(State::WaitingLocation).await?;
    Ok(())
}

async fn handle_get_weather_location_command(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    bot.send_message(msg.chat.id, "Attach your location 📍\n📎 >> location 🌐").await?;
    dialogue.update(State::WaitingLocationAttach).await?;
    Ok(())
}

async fn dialogue_handler(bot: Bot, msg: Message, dialogue: MyDialogue, state: State) -> HandlerResult {
    match state {
        State::Start => {
            bot.send_message(msg.chat.id, "Welcome to the Weather Bot!").await?;
            dialogue.update(State::Start).await?;
        }
        State::WaitingLocation => {
            if let Some(text) = msg.text() {
                bot.send_message(msg.chat.id, format!("In which province or state is {} located?", text)).await?;
                dialogue.update(State::WaitingProvince { location: text.into() }).await?;
            } else {
                bot.send_message(msg.chat.id, "Please send me a valid name").await?;
            }
        }
        State::WaitingLocationAttach => {
            if let Some(location) = msg.location() {
                let weather_info = get_weater_information_from_location(location.latitude, location.longitude, GOOGLE_API_TOKEN.clone()).await?;
                bot.send_message(msg.chat.id, weather_info).await?;
            } else {
                bot.send_message(msg.chat.id, "❌ Please attach a valid location").await?;
            }
            dialogue.exit().await?;
        }
        State::WaitingProvince { location } => {
            if let Some(text) = msg.text() {
                bot.send_message(msg.chat.id, format!("In which country is {} ({})?", location, text)).await?;
                dialogue.update(State::WaitingCountry { location, province: text.into() }).await?;
            } else {
                bot.send_message(msg.chat.id, "Please send me a valid name").await?;
                dialogue.update(State::WaitingProvince { location }).await?;
            }
        }
        State::WaitingCountry { location, province } => {
            if let Some(text) = msg.text() {
                let country = text.to_string();
                bot.send_message(msg.chat.id, format!("Obtaining the climate for {}, {} en {}...", location, province, country)).await?;
                let weather_info: String = get_weater_information(location.clone(), province.clone(), country.clone(), GOOGLE_API_TOKEN.clone()).await?;
                bot.send_message(msg.chat.id, weather_info).await?;
                dialogue.exit().await?;
            } else {
                bot.send_message(msg.chat.id, "Please send me a valid name" ).await?;
                dialogue.update(State::Start).await?;
            }
        }
    }
    Ok(())
}