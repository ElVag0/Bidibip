use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, RoleId, UserId};
use crate::core::utilities::Username;

#[derive(Deserialize)]
struct OldWarnItem {
    date: u64,
    link: String,
    message: String,
}

#[derive(Deserialize)]
struct OldWarnItemInstance {
    warns: Vec<OldWarnItem>
}

#[derive(Deserialize)]
struct OldWarns {
    warns: HashMap<UserId, OldWarnItemInstance>
}

#[derive(Serialize, Deserialize, Default)]
pub struct WarnConfig {
    public_warn_channel: ChannelId,
    moderation_warn_channel: ChannelId,
    #[serde(rename = "ban-vocal")]
    ban_vocal: RoleId,
    // Key is user id
    pub warns: HashMap<UserId, WarnedUserList>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct WarnedUserList {
    pub warns: Vec<UserWarn>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserWarn {
    // The warn date
    date: u64,
    // The person who warned
    from: Username,
    // The warned person
    to: Username,
    // Optional contextual link
    link: Option<String>,
    // Warn reason
    reason: String,
    // Warn details
    details: Option<String>,
    // Action
    action: String,
    // Link to the message in the warn channel history
    full_message_link: String,
}


pub fn migrate() {
    let old : OldWarns = serde_json::from_str(&fs::read_to_string("./saved/old/config/warn.json").unwrap()).unwrap();
    let mut new = WarnConfig::default();



    for warn in old.warns {
        let mut elem = WarnedUserList::default();
        for w in &warn.1.warns {
            elem.warns.push(UserWarn {
                date: w.date,
                from: Username::placeholder(),
                to: Username::placeholder(),
                link: Some(w.link.clone()),
                reason: w.message.clone(),
                details: None,
                action: "".to_string(),
                full_message_link: "".to_string(),
            })
        }

        new.warns.insert(warn.0, elem);
    }

    fs::write("./saved/old/config/warn_new.json", serde_json::to_string_pretty(&new).unwrap()).unwrap();
}