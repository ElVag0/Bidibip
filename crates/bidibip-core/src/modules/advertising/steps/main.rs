use crate::core::error::BidibipError;
use crate::core::utilities::{TruncateText, Username};
use crate::modules::advertising::ad_utils::{ButtonOption, TextOption};
use crate::modules::advertising::steps::fixed_term::FixedTermInfos;
use crate::modules::advertising::steps::freelance::FreelanceInfos;
use crate::modules::advertising::steps::internship::{Compensation, InternshipInfos};
use crate::modules::advertising::steps::open_ended::OpenEndedInfos;
use crate::modules::advertising::steps::recruiter::RecruiterInfos;
use crate::modules::advertising::steps::volunteering::VolunteeringInfos;
use crate::modules::advertising::steps::worker::WorkerInfos;
use crate::modules::advertising::steps::work_study::WorkStudyInfos;
use crate::modules::advertising::steps::{ResetStep, SubStep};
use serde::{Deserialize, Serialize};
use serenity::all::{ButtonStyle, ChannelId, Colour, Context, CreateActionRow, CreateEmbed, CreateEmbedAuthor, CreateMessage, ForumTagId, GuildChannel, Http, Interaction, Message, MessageId, User};
use serenity::builder::{CreateButton, EditMessage};
use crate::core::interaction_utils::make_custom_id;
use crate::core::message_reference::MessageReference;
use crate::modules::advertising::{Advertising, AdvertisingTags};
use crate::on_fail;

#[derive(Serialize, Deserialize, Clone)]
pub enum Contract {
    Volunteering(VolunteeringInfos),
    Internship(InternshipInfos), // paid or not
    Freelance(FreelanceInfos),
    WorkStudy(WorkStudyInfos),
    FixedTerm(FixedTermInfos), // CDD
    OpenEnded(OpenEndedInfos), // CDI
}
#[serenity::async_trait]
impl ResetStep for Contract {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        match self {
            Contract::Volunteering(obj) => { obj.delete(http, thread).await }
            Contract::Internship(obj) => { obj.delete(http, thread).await }
            Contract::Freelance(obj) => { obj.delete(http, thread).await }
            Contract::WorkStudy(obj) => { obj.delete(http, thread).await }
            Contract::FixedTerm(obj) => { obj.delete(http, thread).await }
            Contract::OpenEnded(obj) => { obj.delete(http, thread).await }
        }
    }

    fn clean_for_storage(&mut self) {
        match self {
            Contract::Volunteering(v) => { v.clean_for_storage() }
            Contract::Internship(v) => { v.clean_for_storage() }
            Contract::Freelance(v) => { v.clean_for_storage() }
            Contract::WorkStudy(v) => { v.clean_for_storage() }
            Contract::FixedTerm(v) => { v.clean_for_storage() }
            Contract::OpenEnded(v) => { v.clean_for_storage() }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum What {
    Recruiter(RecruiterInfos),
    Worker(WorkerInfos),
}
#[serenity::async_trait]
impl ResetStep for What {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        match self {
            What::Recruiter(obj) => { obj.delete(http, thread).await }
            What::Worker(obj) => { obj.delete(http, thread).await }
        }
    }

    fn clean_for_storage(&mut self) {
        match self {
            What::Recruiter(v) => { v.clean_for_storage() }
            What::Worker(v) => { v.clean_for_storage() }
        }
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub enum Contact {
    Discord,
    Other(TextOption),
}
#[serenity::async_trait]
impl ResetStep for Contact {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        match self {
            Contact::Discord => { Ok(()) }
            Contact::Other(obj) => { obj.delete(http, thread).await }
        }
    }

    fn clean_for_storage(&mut self) {
        match self {
            Contact::Discord => {}
            Contact::Other(v) => { v.clean_for_storage() }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct MainSteps {
    #[serde(skip_serializing_if = "TextOption::is_none")]
    #[serde(default)]
    pub title: TextOption,
    #[serde(skip_serializing_if = "ButtonOption::is_none")]
    #[serde(default)]
    pub kind: ButtonOption<Contract>,
    #[serde(skip_serializing_if = "TextOption::is_none")]
    #[serde(default)]
    pub description: TextOption,
    #[serde(skip_serializing_if = "ButtonOption::is_none")]
    #[serde(default)]
    pub is_recruiter: ButtonOption<What>,
    #[serde(skip_serializing_if = "ButtonOption::is_none")]
    #[serde(default)]
    pub contact: ButtonOption<Contact>,
    #[serde(skip_serializing_if = "TextOption::is_none")]
    #[serde(default)]
    pub other_urls: TextOption,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    demo_message: Option<MessageId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub edited_post: Option<MessageReference>,
}

#[serenity::async_trait]
impl ResetStep for MainSteps {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        self.title.delete(http, thread).await?;
        self.kind.delete(http, thread).await?;
        self.description.delete(http, thread).await?;
        self.is_recruiter.delete(http, thread).await?;
        self.contact.delete(http, thread).await?;
        self.other_urls.delete(http, thread).await?;
        Ok(())
    }

    fn clean_for_storage(&mut self) {
        self.title.clean_for_storage();
        self.kind.clean_for_storage();
        self.description.clean_for_storage();
        self.is_recruiter.clean_for_storage();
        self.contact.clean_for_storage();
        self.other_urls.clean_for_storage();
        self.demo_message = None;
        self.edited_post = None;
    }
}

#[serenity::async_trait]
impl SubStep for MainSteps {
    async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.title.is_unset() {
            if self.title.try_init(&ctx.http, thread, "Donne un titre à ton annonce", false).await? {
                return Ok(false);
            }
        }

        if self.description.is_unset() {
            if self.description.try_init(&ctx.http, thread, "Décris ton annonce, en quoi elle consiste, qui tu es etc...", false).await? {
                return Ok(false);
            }
        }

        if self.kind.is_unset() {
            if self.kind.try_init(&ctx.http, thread, "Quel type de contrat recherches-tu ?", vec![
                ("volunteering", "🤝 Bénévolat (non rémunéré)", Contract::Volunteering(VolunteeringInfos::default())),
                ("internship", "🪂 Stage", Contract::Internship(InternshipInfos::default())),
                ("workstudy", "🤓 Alternance (rémunéré)", Contract::WorkStudy(WorkStudyInfos::default())),
                ("freelance", "🧐 Freelance", Contract::Freelance(FreelanceInfos::default())),
                ("fixed", "😎 CDD (rémunéré)", Contract::FixedTerm(FixedTermInfos::default())),
                ("open", "🤯 CDI (rémunéré)", Contract::OpenEnded(OpenEndedInfos::default())),
            ]).await? {
                return Ok(false);
            }
        }

        if let Some(kind) = self.kind.value_mut() {
            if !match kind {
                Contract::Volunteering(data) => { data.advance(ctx, thread).await? }
                Contract::Internship(data) => { data.advance(ctx, thread).await? }
                Contract::Freelance(data) => { data.advance(ctx, thread).await? }
                Contract::WorkStudy(data) => { data.advance(ctx, thread).await? }
                Contract::FixedTerm(data) => { data.advance(ctx, thread).await? }
                Contract::OpenEnded(data) => { data.advance(ctx, thread).await? }
            } { return Ok(false); }
        }

        if self.is_recruiter.is_unset() {
            if self.is_recruiter.try_init(&ctx.http, thread, "Es-tu recruteur ou recherches tu du travail ?", vec![
                ("worker", "🔧 Je cherche du travail", What::Worker(WorkerInfos::default())),
                ("recruiter", "🕵️‍♀️ Je recrute", What::Recruiter(RecruiterInfos::default())),
            ]).await? {
                return Ok(false);
            }
        }

        if let Some(recruiter) = self.is_recruiter.value_mut() {
            if !match recruiter {
                What::Recruiter(infos) => { infos.advance(ctx, thread).await? }
                What::Worker(infos) => { infos.advance(ctx, thread).await? }
            } { return Ok(false); }
        }

        match self.contact.value_mut() {
            None => {
                if self.contact.try_init(&ctx.http, thread, "Comment peut-on te contacter ?", vec![
                    ("discord", "Discord", Contact::Discord),
                    ("other", "Autre", Contact::Other(TextOption::default())),
                ]).await? {
                    return Ok(false);
                }
            }
            Some(contact) => {
                if let Contact::Other(other) = contact {
                    if other.try_init(&ctx.http, thread, "Indique au moins un moyen de contact (mail etc...)", false).await? {
                        return Ok(false);
                    }
                }
            }
        }

        if self.other_urls.is_unset() {
            if self.other_urls.try_init(&ctx.http, thread, "Ajoutes d'autres informations (liens etc...)", true).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn receive_message(&mut self, ctx: &Context, thread: &ChannelId, message: &Message) -> Result<bool, BidibipError> {
        if let Some(Contact::Other(other)) = self.contact.value_mut() {
            if other.try_set(&ctx.http, thread, message).await? {
                return Ok(true);
            }
        }
        Ok(self.title.try_set(&ctx.http, thread, message).await? ||
            self.description.try_set(&ctx.http, thread, message).await? ||
            self.other_urls.try_set(&ctx.http, thread, message).await?)
    }
    async fn on_interaction(&mut self, ctx: &Context, component: &Interaction) -> Result<bool, BidibipError> {
        if let Some(Contact::Other(other)) = self.contact.value_mut() {
            if other.try_edit(&ctx.http, component).await? { return Ok(true); }
        }
        Ok(self.description.try_edit(&ctx.http, component).await? ||
            self.title.try_edit(&ctx.http, component).await? ||
            self.other_urls.try_edit(&ctx.http, component).await? ||
            self.contact.try_set(&ctx.http, component).await? ||
            self.is_recruiter.try_set(&ctx.http, component).await? ||
            self.kind.try_set(&ctx.http, component).await?)
    }

    fn get_dependencies(&mut self) -> Vec<&mut dyn SubStep> {
        let mut deps: Vec<&mut dyn SubStep> = vec![];
        if let Some(what) = self.is_recruiter.value_mut() {
            match what {
                What::Recruiter(infos) => { deps.push(infos); }
                What::Worker(infos) => { deps.push(infos); }
            }
        }
        if let Some(kind) = self.kind.value_mut() {
            match kind {
                Contract::Volunteering(infos) => { deps.push(infos); }
                Contract::Internship(infos) => { deps.push(infos); }
                Contract::Freelance(infos) => { deps.push(infos); }
                Contract::WorkStudy(infos) => { deps.push(infos); }
                Contract::FixedTerm(infos) => { deps.push(infos); }
                Contract::OpenEnded(infos) => { deps.push(infos); }
            }
        }
        deps
    }
}

impl MainSteps {
    fn build_content(&mut self, user: &User) -> Vec<CreateEmbed> {
        let title = match &self.title.value() {
            None => { "[Titre manquant]" }
            Some(title) => { title.as_str() }
        };

        let description = match &self.description.value() {
            None => { "[Aucune description]" }
            Some(title) => { title.as_str() }
        };

        let goal = match self.is_recruiter.value() {
            None => { "[Objectif manquant]".to_string() }
            Some(what) => {
                match what {
                    What::Recruiter(_) => {
                        let kind = match self.kind.value() {
                            None => { "[Type manquant]" }
                            Some(kind) => {
                                match kind {
                                    Contract::Volunteering(_) => { "un.e volontaire" }
                                    Contract::Internship(_) => { "un.e stagiaire" }
                                    Contract::Freelance(_) => { "un.e freelance" }
                                    Contract::WorkStudy(_) => { "un.e alternant.e" }
                                    Contract::FixedTerm(_) => { "pour un.e CDD" }
                                    Contract::OpenEnded(_) => { "pour un.e CDI" }
                                }
                            }
                        };
                        format!("{} recrute {}", Username::from_user(user).safe_full(), kind)
                    }
                    What::Worker(_) => {
                        let kind = match self.kind.value() {
                            None => { "[Type manquant]" }
                            Some(kind) => {
                                match kind {
                                    Contract::Volunteering(_) => { "volontaire" }
                                    Contract::Internship(_) => { "candidat.e pour un stage" }
                                    Contract::Freelance(_) => { "un.e freelance" }
                                    Contract::WorkStudy(_) => { "candidat.e pour une alternance" }
                                    Contract::FixedTerm(_) => { "candidat.e pour un CDD" }
                                    Contract::OpenEnded(_) => { "candidat.e pour un CDI" }
                                }
                            }
                        };
                        format!("{} est {}", Username::from_user(user).safe_full(), kind)
                    }
                }
            }
        };

        let mut main_embed = CreateEmbed::new()
            .author(CreateEmbedAuthor::new(goal).icon_url(if let Some(avatar) = user.avatar_url() { avatar } else { user.default_avatar_url() }))
            .title(title)
            .color(Colour::PURPLE)
            .description(format!("{}", description.truncate_text(4000)));


        let mut fields = vec![];
        let mut embeds = vec![];

        let mut items = self.get_dependencies();
        while let Some(item) = items.pop() {
            item.fill_message(&mut fields, &mut embeds);
            items.append(&mut item.get_dependencies());
        }

        for field in fields {
            main_embed = main_embed.field(field.0.truncate_text(256), field.1.truncate_text(1024), field.2);
        }

        let mut last_embed = CreateEmbed::new()
            .color(Colour::PURPLE)
            .title("Contact")
            .description(match self.contact.value() {
                None => { "[Donnée manquante]".to_string() }
                Some(contact) => {
                    match contact {
                        Contact::Discord => { format!("Discord : {}", Username::from_user(user).full()) }
                        Contact::Other(other) => {
                            match other.value() {
                                None => { "[Donnée manquante]".to_string() }
                                Some(val) => { val.clone().truncate_text(4000) }
                            }
                        }
                    }
                }
            });

        if let Some(other) = self.other_urls.value() {
            last_embed = last_embed.field("Autre", other.truncate_text(1024), false);
        }

        embeds.push(last_embed);

        #[derive(Serialize, Deserialize, Clone, Default)]
        pub struct MainSteps {
            pub contact: ButtonOption<Contact>,
            pub other_urls: TextOption,
            demo_message: Option<MessageId>,
        }
        embeds.insert(0, main_embed);
        embeds
    }

    pub fn edit_message(&mut self, user: &User) -> EditMessage {
        EditMessage::new().embeds(self.build_content(user))
    }

    pub fn create_message(&mut self, user: &User) -> CreateMessage {
        CreateMessage::new().embeds(self.build_content(user))
    }

    pub fn get_tags(&self, config: &AdvertisingTags) -> Vec<ForumTagId> {
        let mut tags = vec![];
        if let Some(What::Recruiter(val)) = self.is_recruiter.value() {
            tags.push(config.recruiter);
            if let Some(val) = val.location.value() {
                match val {
                    crate::modules::advertising::steps::recruiter::Location::Remote => {
                        tags.push(config.remote);
                    }
                    crate::modules::advertising::steps::recruiter::Location::OnSiteFlex(_) => {
                        tags.push(config.on_site_flex);
                    }
                    crate::modules::advertising::steps::recruiter::Location::OnSite(_) => {
                        tags.push(config.on_site);
                    }
                }
            }
        }
        if let Some(What::Worker(val)) = self.is_recruiter.value() {
            tags.push(config.worker);
            if let Some(val) = val.location.value() {
                match val {
                    crate::modules::advertising::steps::worker::Location::Remote => {
                        tags.push(config.remote);
                    }
                    crate::modules::advertising::steps::worker::Location::Anywhere(_) => {
                        tags.push(config.on_site_flex);
                    }
                    crate::modules::advertising::steps::worker::Location::OnSite(_) => {
                        tags.push(config.on_site);
                    }
                }
            }
        }
        if let Some(val) = self.kind.value() {
            match val {
                Contract::Volunteering(_) => {
                    tags.push(config.volunteer);
                    tags.push(config.unpaid);
                }
                Contract::Internship(val) => {
                    tags.push(config.internship);
                    if let Some(Compensation::Yes(_)) = val.compensation.value() {
                        tags.push(config.paid);
                    } else {
                        tags.push(config.unpaid);
                    }
                }
                Contract::Freelance(_) => {
                    tags.push(config.freelance);
                    tags.push(config.paid);
                }
                Contract::WorkStudy(_) => {
                    tags.push(config.work_study);
                    tags.push(config.paid);
                }
                Contract::FixedTerm(_) => {
                    tags.push(config.fixed_term);
                    tags.push(config.paid);
                }
                Contract::OpenEnded(_) => {
                    tags.push(config.open_ended);
                    tags.push(config.paid);
                }
            }
        }
        tags
    }

    pub async fn print_preview_message_in_channel(&mut self, ctx: &Context, thread: &ChannelId, user: &User) -> Result<(), BidibipError> {
        let message = self.create_message(user).content("# Voici ton annonce telle qu'elle sera présentée. Vérifies les informations présentes avant de la publier.").components(vec![CreateActionRow::Buttons(vec![
            CreateButton::new(make_custom_id::<Advertising>("pre-publish", "")).label("Publier").style(ButtonStyle::Primary)
        ])]);

        if let Some(old_message) = self.demo_message {
            #[allow(unused)]
            thread.delete_message(&ctx.http, old_message).await;
        }

        self.demo_message = Some(on_fail!(thread.send_message(&ctx.http, message).await, "Failed to send final message")?.id);
        Ok(())
    }
}