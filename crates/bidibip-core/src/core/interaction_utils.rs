use std::fmt::Display;
use serenity::all::ComponentInteractionData;
use crate::modules::{BidibipModule, LoadModule};

pub fn make_custom_id<Module: BidibipModule + LoadModule<Module>>(action: &str, id: Option<impl Display>) -> String {
    format!("{}::{}{}", Module::name(), action, match id {
        None => { String::new() }
        Some(id) => { format!("::{}", id) }
    })
}

pub trait InteractionUtils {
    fn get_custom_id_data<Module: BidibipModule + LoadModule<Module>>(&self, action: &str) -> Option<String>;
}

impl InteractionUtils for ComponentInteractionData {
    fn get_custom_id_data<Module: BidibipModule + LoadModule<Module>>(&self, action: &str) -> Option<String> {
        let mut split = self.custom_id.split("::");
        let module = match split.next() {
            None => { return None }
            Some(module) => { module }
        };
        if module != Module::name() {
            return None;
        }
        let data_action = match split.next() {
            None => { return None }
            Some(data_action) => { data_action }
        };
        if data_action != action {
            return None;
        }
        match split.next() {
            None => { Some(String::new()) }
            Some(payload) => { Some(payload.to_string()) }
        }
    }
}