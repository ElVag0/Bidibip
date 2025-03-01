use crate::core::error::BidibipError;
use crate::core::utilities::Username;
use crate::modules::advertising::ad_utils::{ButtonOption, TextOption};
use crate::modules::advertising::steps::fixed_term::FixedTermInfos;
use crate::modules::advertising::steps::freelance::FreelanceInfos;
use crate::modules::advertising::steps::internship::InternshipInfos;
use crate::modules::advertising::steps::open_ended::OpenEndedInfos;
use crate::modules::advertising::steps::recruiter::RecruiterInfos;
use crate::modules::advertising::steps::volunteering::VolunteeringInfos;
use crate::modules::advertising::steps::worker::WorkerInfos;
use crate::modules::advertising::steps::workstudy::WorkStudyInfos;
use crate::modules::advertising::steps::{ResetStep, SubStep};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, CreateEmbed, CreateMessage, GuildChannel, Http, Message};
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
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct MainSteps {
    pub title: TextOption,
    pub kind: ButtonOption<Contract>,
    pub description: TextOption,
    pub is_recruiter: ButtonOption<What>,
    pub contact: ButtonOption<Contact>,
    pub other_urls: TextOption,
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
}

#[serenity::async_trait]
impl SubStep for MainSteps {
    async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.title.is_unset() {
            self.title.try_init(&ctx.http, thread, "Donne un titre à ton annonce").await?;
            return Ok(false);
        }

        if self.description.is_unset() {
            self.description.try_init(&ctx.http, thread, "Décris ton annonce, en quoi elle consiste, qui tu es etc...").await?;
            return Ok(false);
        }

        if self.kind.is_unset() {
            self.kind.try_init(&ctx.http, thread, "Quel type de contrat recherches-tu ?", vec![
                ("🤝 Bénévolat (non rémunéré)", Contract::Volunteering(VolunteeringInfos::default())),
                ("🪂 Stage", Contract::Internship(InternshipInfos::default())),
                ("🤓 Alternance (rémunéré)", Contract::WorkStudy(WorkStudyInfos::default())),
                ("🧐 Freelance", Contract::Freelance(FreelanceInfos::default())),
                ("😎 CDD (rémunéré)", Contract::FixedTerm(FixedTermInfos::default())),
                ("🤯 CDI (rémunéré)", Contract::OpenEnded(OpenEndedInfos::default())),
            ]).await?;
            return Ok(false);
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
            self.is_recruiter.try_init(&ctx.http, thread, "Quel type de contrat recherches-tu ?", vec![
                ("🔧 Je cherche du travail", What::Worker(WorkerInfos::default())),
                ("🕵️‍♀️ Je recrute", What::Recruiter(RecruiterInfos::default())),
            ]).await?;
            return Ok(false);
        }
        if let Some(recruiter) = self.is_recruiter.value_mut() {
            if !match recruiter {
                What::Recruiter(infos) => { infos.advance(ctx, thread).await? }
                What::Worker(infos) => { infos.advance(ctx, thread).await? }
            } { return Ok(false); }
        }

        match self.contact.value_mut() {
            None => {
                self.contact.try_init(&ctx.http, thread, "Comment peut-on te contacter ?", vec![
                    ("Discord", Contact::Discord),
                    ("Autre", Contact::Other(TextOption::default())),
                ]).await?;
                return Ok(false);
            }
            Some(contact) => {
                if let Contact::Other(other) = contact {
                    other.try_init(&ctx.http, thread, "Indique au moins un moyen de contact (mail etc...)").await?;
                    return Ok(false);
                }
            }
        }

        if self.other_urls.is_unset() {
            self.other_urls.try_init(&ctx.http, thread, "Précises d'autres liens utils").await?;
            return Ok(false);
        }

        println!("finished");
        on_fail!(thread.send_message(&ctx.http, self.create_message()).await, "Failed to send final message")?;
        Ok(true)
    }

    async fn receive_message(&mut self, ctx: &Context, thread: &ChannelId, message: &Message) -> Result<(), BidibipError> {
        self.title.try_set(&ctx.http, thread, message).await?;
        self.description.try_set(&ctx.http, thread, message).await?;
        if let Some(Contact::Other(other)) = self.contact.value_mut() {
            other.try_set(&ctx.http, thread, message).await?;
        }
        self.other_urls.try_set(&ctx.http, thread, message).await?;
        Ok(())
    }
    async fn clicked_button(&mut self, ctx: &Context, thread: &ChannelId, action: &str) -> Result<(), BidibipError> {
        self.title.reset(&ctx.http, thread, action).await?;
        self.description.reset(&ctx.http, thread, action).await?;
        if let Some(Contact::Other(other)) = self.contact.value_mut() {
            other.reset(&ctx.http, thread, action).await?;
        }
        self.other_urls.reset(&ctx.http, thread, action).await?;
        self.contact.try_set(&ctx.http, thread, action).await?;
        self.is_recruiter.try_set(&ctx.http, thread, action).await?;
        self.kind.try_set(&ctx.http, thread, action).await?;
        Ok(())
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
    fn create_message(&self) -> CreateMessage {
        let mut message = CreateMessage::new();

        let title = match &self.title.value() {
            None => { "[Titre manquant]" }
            Some(title) => { title.as_str() }
        };

        let description = match &self.description.value() {
            None => { "[Aucune description]" }
            Some(title) => { title.as_str() }
        };

        let mut main_embed = CreateEmbed::new()
            .title(title)
            .description(format!("Annonce de {}:\n{}", "TODO : INSERT AUTHOR", description));

        if let Some(kind) = &self.kind.value() {
            match kind {
                Contract::Volunteering(_) => {}
                Contract::Internship(_) => {}
                Contract::Freelance(infos) => {
                    main_embed = main_embed.field("Durée", match infos.duration.value() {
                        None => { "[non spécifié]" }
                        Some(duration) => { duration.as_str() }
                    }, false);
                }
                Contract::WorkStudy(_) => {}
                Contract::FixedTerm(_) => {}
                Contract::OpenEnded(_) => {}
            }
        }

        message = message.embed(main_embed);

        message
    }
}