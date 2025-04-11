use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, UserId};

#[derive(Serialize, Deserialize, Default)]
struct UserTickets {
    thread: ChannelId,
}

#[derive(Serialize, Deserialize, Default)]
struct ModoConfig {
    modo_channel: ChannelId,
    tickets: HashMap<UserId, UserTickets>,
}

#[derive(Deserialize)]
struct OldModo {
    opened_tickets: HashMap<UserId, ChannelId>,
}

pub async fn migrate(_: &Context) {
    let old : OldModo = serde_json::from_str(&fs::read_to_string("./saved/old/config/modo.json").unwrap()).unwrap();

    let mut new = ModoConfig::default();

    for ticket in old.opened_tickets {
        new.tickets.insert(ticket.0, UserTickets { thread: ticket.1 });
    }

    fs::write("./saved/old/config/modo_new.json", serde_json::to_string_pretty(&new).unwrap()).unwrap();
}