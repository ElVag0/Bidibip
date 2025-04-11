use std::collections::{HashMap, HashSet};
use std::fs;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, MessageId, UserId};
use crate::core::message_reference::MessageReference;
use crate::core::utilities::Username;

#[derive(Deserialize)]
struct OldRepostChannels {
    bound_channels: Vec<ChannelId>,
    vote: bool
}

#[derive(Deserialize)]
struct OldRepostVotes {
    vote_yes: HashMap<UserId, bool>,
    vote_no: HashMap<UserId, bool>,
    bound_messages: Vec<String>,
    channel_title: String
}

#[derive(Deserialize)]
struct OldRepost {
    reposted_forums: HashMap<ChannelId, OldRepostChannels>,
    repost_votes: HashMap<ChannelId, OldRepostVotes>,
    vote_messages: HashMap<String, ChannelId>
}

#[derive(Default, Serialize, Deserialize, Clone)]
struct VoteConfig {
    thread_name: String,
    source_message_url: String,
    source_thread: ChannelId,
    reposted_message: HashSet<MessageReference>,
    vote_message: MessageReference,
    yes: HashMap<UserId, Username>,
    no: HashMap<UserId, Username>,
}


#[derive(Default, Serialize, Deserialize, Clone)]
struct RepostChannelConfig {
    repost_channel: HashSet<ChannelId>,
    vote_enabled: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct RepostConfig {
    // Forum - RepostChannel
    forums: HashMap<ChannelId, RepostChannelConfig>,
    // Channel - config
    votes: HashMap<ChannelId, VoteConfig>,
}

pub async fn migrate(ctx: &Context) {
    let old : OldRepost = serde_json::from_str(&fs::read_to_string("./saved/old/config/repost.json").unwrap()).unwrap();

    let mut new = RepostConfig::default();

    for forum in &old.reposted_forums {
        let mut new_channel = RepostChannelConfig::default();

        new_channel.vote_enabled = forum.1.vote;

        for old_channel in &forum.1.bound_channels {
            new_channel.repost_channel.insert(old_channel.clone());
        }

        new.forums.insert(forum.0.clone(), new_channel);
    }

    let mut total = old.repost_votes.len();
    let mut it = 0;

    for old in &old.repost_votes {
        it += 1;
        println!("\t- Convert repost votes {} / {}", it, total);
        let mut new_config = VoteConfig::default();

        for no in &old.1.vote_no {
            new_config.no.insert(no.0.clone(), Username::from_user(&no.0.to_user(&ctx.http).await.unwrap()));
        }

        for yes in &old.1.vote_yes {
            new_config.yes.insert(yes.0.clone(), Username::from_user(&yes.0.to_user(&ctx.http).await.unwrap()));
        }

        new_config.thread_name = old.1.channel_title.clone();
        for message in &old.1.bound_messages  {
            let mut spl = message.split("/");
            let channel = spl.next().unwrap();
            let message = spl.next().unwrap();
            new_config.reposted_message.insert(MessageReference::new(MessageId::from(u64::from_str(message).unwrap()), ChannelId::from(u64::from_str(channel).unwrap())));
        }

        let mut spl = old.1.bound_messages.first().unwrap().split("/");
        let channel = spl.next().unwrap();
        let message = spl.next().unwrap();
        new_config.source_message_url = format!("https://discord.com/channels/293047579288272897/{channel}/{message}");
        new_config.source_thread = old.0.clone();

        new.votes.insert(old.0.clone(), new_config);
    }

    total = old.repost_votes.len();
    it = 0;

    for vote_message in old.vote_messages {
        it += 1;
        println!("\t- Convert vote messages {} / {}", it, total);
        let mut spl = vote_message.0.split("/");
        let channel = spl.next().unwrap();
        let message = spl.next().unwrap();

        if let Some(it) = new.votes.get_mut(&vote_message.1) {
            it.vote_message = MessageReference::new(MessageId::from(u64::from_str(message).unwrap()), ChannelId::from(u64::from_str(channel).unwrap()));
        }
    }

    fs::write("./saved/old/config/repost_new.json", serde_json::to_string_pretty(&new).unwrap()).unwrap();
}