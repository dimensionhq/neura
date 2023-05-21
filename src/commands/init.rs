use crate::{cli::prompts, config::Config, constants::models::MODELS, models::model::Model};
use colored::Colorize;
use miette::Result;

pub async fn execute() -> Result<()> {
    // The `init` command will take care of creating the necessary files and folders required for `neura` to work
    // It will populate the `neura.toml` file with the values selected by the user
    let mut config = Config::new();

    // First, we ask the user which model they want to use for their project
    let model_selection = prompts::Select {
        message: "Which model would you like to use?".into(),
        paged: true,
        selected: Some(1),
        items: MODELS.iter().map(|l| l.into()).collect(),
    };

    // Then, we create a `Model` instance from the user's input
    let model = Model::from_input(&MODELS[model_selection.run().unwrap()]);

    config.set_model(model);

    // Then, we save the config to the `neura.toml` file
    config.save();

    // Let's check if the user's `.env` exists and contains the `NEURA_API_KEY` variable
    // If it doesn't, we'll ask the user to provide it

    if !Config::dotenv_exists() {
        println!(
            "It seems like you don't have a `{}` file. Let's create one!",
            ".env".bright_cyan(),
        );

        let api_key = prompts::Input {
            message: "Neura API Key".into(),
            default: None,
            allow_empty: false,
        }
        .run()
        .unwrap();

        config.create_dotenv();

        config.set_env("NEURA_API_KEY".into(), api_key);

        println!(
            "The `{}` file has been created and the `{}` variable has been added!",
            ".env".bright_cyan(),
            "NEURA_API_KEY".bright_cyan(),
        );
    } else {
        if !config.check_env_var_exists("NEURA_API_KEY".to_string()) {
            println!(
                "It seems like your `{}` file doesn't contain the `{}` variable. Let's add it!",
                ".env".bright_cyan(),
                "NEURA_API_KEY".bright_cyan(),
            );

            let api_key = prompts::Input {
                message: "Neura API Key".into(),
                default: None,
                allow_empty: false,
            }
            .run()
            .unwrap();

            config.set_env("NEURA_API_KEY".into(), api_key);

            println!(
                "The `{}` variable has been added to your `{}` file!",
                "NEURA_API_KEY".bright_cyan(),
                ".env".bright_cyan(),
            );
        }
    }

    println!(
        "🚀 Your project has been initialized with the {} model.",
        config.model.unwrap().to_string().bright_green(),
    );

    println!(
        "💁 You can now run `{}` to start an ai-powered debugging copilot!",
        "neura".bright_cyan()
    );

    Ok(())
}
