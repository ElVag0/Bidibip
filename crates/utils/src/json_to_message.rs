use anyhow::Error;
use serde::Deserialize;
use serenity::all::{ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateMessage};
use crate::utilities::TruncateText;

#[derive(Deserialize, Debug)]
struct JsonToMessageButton {
    #[serde(rename = "type")]
    button_type: Option<String>,
    texte: String,
    identifiant: String,
}
#[derive(Deserialize, Debug)]
struct JsonToMessageMessageInteraction {
    bouton: Option<JsonToMessageButton>
}

#[derive(Deserialize, Debug)]
struct JsonToMessageMessageEmbed {
    titre: String,
    description: Option<String>,
}

#[derive(Deserialize, Debug)]
struct JsonToMessageMessage {
    textes: Option<Vec<String>>,
    embeds: Option<Vec<JsonToMessageMessageEmbed>>,
    interactions: Option<Vec<JsonToMessageMessageInteraction>>,
}

#[derive(Deserialize, Debug)]
struct JsonToMessageBase {
    messages: Vec<JsonToMessageMessage>,
}

pub fn json_to_message(json: String) -> Result<Vec<CreateMessage>, Error> {
    let data: JsonToMessageBase = serde_json::from_str(json.as_str())?;
    let mut messages = vec![];

    for message in data.messages {
        let mut data = CreateMessage::new();

        if (message.textes.is_none() || message.textes.as_ref().unwrap().is_empty()) && (message.embeds.is_none() || message.embeds.as_ref().unwrap().is_empty()) {
            return Err(Error::msg("Chaque message doit contenir au moins un message ou au moins un embed"));
        }
        if let Some(textes) = &message.textes {
            if !textes.is_empty() {
                let mut full_text = String::new();
                for text in textes {
                    full_text += format!("{}\n", text).as_str();
                }
                data = data.content(full_text.truncate_text(2000));
            }
        }
        if let Some(embeds) = message.embeds {
            for embed in embeds {
                let mut create_embed = CreateEmbed::new().title(embed.titre);
                if let Some(description) = embed.description {
                    create_embed = create_embed.description(description.truncate_text(4096));
                }
                data = data.embed(create_embed);
            }
        }
        if let Some(interactions) = message.interactions
        {
            let mut components = vec![];
            for interaction in interactions {

                if let Some(button) = interaction.bouton {
                    let mut create_button = CreateButton::new(button.identifiant).label(button.texte);
                    if let Some(button_type) = button.button_type {
                        let button_type = match button_type.as_str() {
                            "Primary" => { ButtonStyle::Primary }
                            "Secondary" => { ButtonStyle::Secondary }
                            "Success" => { ButtonStyle::Success }
                            "Danger" => { ButtonStyle::Danger }
                            &_ => { ButtonStyle::Primary }
                        };
                        create_button = create_button.style(button_type);
                    }
                    components.push(CreateActionRow::Buttons(vec![create_button]))
                }
            }
            data = data.components(components);
        }
        messages.push(data);
    }
    Ok(messages)
}
