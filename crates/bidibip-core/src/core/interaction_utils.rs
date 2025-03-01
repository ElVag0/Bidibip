use crate::modules::{BidibipModule, LoadModule};
use serenity::all::ComponentInteractionData;
use std::fmt::Display;

pub fn make_custom_id<Module: BidibipModule + LoadModule<Module>>(action: &str, id: impl Display) -> String {
    format!("{}::{}::{}", Module::name(), action, id)
}


pub trait InteractionUtils {
    fn get_custom_id_data<Module: BidibipModule + LoadModule<Module>>(&self, action: &str) -> Option<String>;
    fn get_custom_id_action<Module: BidibipModule + LoadModule<Module>>(&self) -> Option<(String, String)>;
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

    fn get_custom_id_action<Module: BidibipModule + LoadModule<Module>>(&self) -> Option<(String, String)> {
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
        match split.next() {
            None => { Some((data_action.to_string(), String::new())) }
            Some(payload) => { Some((data_action.to_string(), payload.to_string())) }
        }
    }
}