use serde::{Deserialize, Serialize};
use serenity::all::{ButtonStyle, Context, CreateActionRow, CreateButton, CreateEmbed, CreateMessage, GuildChannel, Message, User};
use crate::core::error::BidibipError;
use crate::core::interaction_utils::make_custom_id;
use crate::core::utilities::Username;
use crate::modules::advertising::ad_utils::{create_multi_button_options, create_text_input_options, ButtonDescription, Contact};
use crate::modules::advertising::{Advertising, Step};
use crate::modules::advertising::steps::fixed_term::FixedTermInfos;
use crate::modules::advertising::steps::freelance::FreelanceInfos;
use crate::modules::advertising::steps::internship::InternshipInfos;
use crate::modules::advertising::steps::open_ended::OpenEndedInfos;
use crate::modules::advertising::steps::recruiter::RecruiterInfos;
use crate::modules::advertising::steps::volunteering::VolunteeringInfos;
use crate::modules::advertising::steps::worker::WorkerInfos;
use crate::modules::advertising::steps::workstudy::WorkStudyInfos;
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

#[derive(Serialize, Deserialize, Clone)]
pub enum What {
    Recruiter(RecruiterInfos),
    Worker(WorkerInfos),
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct MainSteps {
    step: Step,
    pub title: Option<String>,
    pub kind: Option<Contract>,
    pub description: Option<String>,
    pub is_recruiter: Option<What>,
    pub contact: Option<Contact>,
    pub other_urls: Option<Vec<String>>,
}

impl MainSteps {
    pub async fn advance(&mut self, ctx: &Context, thread: &GuildChannel, user: &User) -> Result<(), BidibipError> {
        if self.title.is_none() {
            if self.step.test_or_set("TITLE") { return Ok(()); }
            create_text_input_options::<Advertising>(&ctx.http, thread, "Donne un titre à ton annonce", Some("title")).await?;
            return Ok(());
        }

        if self.description.is_none() {
            if self.step.test_or_set("DESCRIPTION") { return Ok(()); }
            create_text_input_options::<Advertising>(&ctx.http, thread, "Décris ton annonce, en quoi elle consiste, qui tu es etc...", Some("description")).await?;
            return Ok(());
        }

        match &mut self.kind {
            None => {
                if self.step.test_or_set("KIND") { return Ok(()); }
                create_multi_button_options::<Advertising>(&ctx.http, thread, "Quel type de contrat recherches-tu ?", vec![
                    ButtonDescription::new("job_volunteering", "🤝 Bénévolat (non rémunéré)"),
                    ButtonDescription::new("job_internship", "🪂 Stage"),
                    ButtonDescription::new("job_workstudy", "🤓 Alternance (rémunéré)"),
                    ButtonDescription::new("job_freelance", "🧐 Freelance"),
                    ButtonDescription::new("job_fixedterm", "😎 CDD (rémunéré)"),
                    ButtonDescription::new("job_openended", "🤯 CDI (rémunéré)"),
                ]).await?;
                return Ok(());
            }
            Some(data) => {
                if !match data {
                    Contract::Volunteering(data) => { data.advance(ctx, thread).await? }
                    Contract::Internship(data) => { data.advance(ctx, thread).await? }
                    Contract::Freelance(data) => { data.advance(ctx, thread).await? }
                    Contract::WorkStudy(data) => { data.advance(ctx, thread).await? }
                    Contract::FixedTerm(data) => { data.advance(ctx, thread).await? }
                    Contract::OpenEnded(data) => { data.advance(ctx, thread).await? }
                } {return Ok(())}
            }
        }

        match &mut self.is_recruiter {
            None => {
                if self.step.test_or_set("IS_RECRUITER") { return Ok(()); }
                create_multi_button_options::<Advertising>(&ctx.http, thread, "Que cherches tu ?", vec![
                    ButtonDescription::new("looking_job", "‍🔧 Je cherche du travail"),
                    ButtonDescription::new("looking_employee", "🕵️‍♀️ Je recrute"),
                ]).await?;
                return Ok(());
            }
            Some(data) => {
                let res = match data {
                    What::Recruiter(infos) => { infos.advance(ctx, thread).await? }
                    What::Worker(infos) => { infos.advance(ctx, thread).await? }
                };
                if !res {return Ok(())}
            }
        }


        match &self.contact {
            None => {
                if self.step.test_or_set("CONTACT") { return Ok(()); }
                create_multi_button_options::<Advertising>(&ctx.http, thread, "Comment peut-on te contacter ?", vec![
                    ButtonDescription::new("contact_discord", "Discord"),
                    ButtonDescription::new("contact_other", "Autre")
                ]).await?;
                return Ok(());
            }
            Some(apply_at) => {
                match apply_at {
                    Contact::Discord => {}
                    Contact::Other(location) => {
                        if location.is_none() {
                            if self.step.test_or_set("CONTACT_OTHER") { return Ok(()); }
                            create_text_input_options::<Advertising>(&ctx.http, thread, "Indique au moins un moyen de contact (mail etc...)", Some("contact_other")).await?;
                            return Ok(());
                        }
                    }
                }
            }
        };

        match &self.other_urls {
            None => {
                if self.step.test_or_set("URLS") { return Ok(()); }
                on_fail!(thread.send_message( & ctx.http, CreateMessage::new().content("## ▶  Précises d'autres liens utils")
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(make_custom_id::<Advertising>("edit_urls", thread.id)).style(ButtonStyle::Secondary).label("modifier")])])).await, "Failed to send message")?;
                return Ok(());
            }
            Some(_) => {}
        };

        println!("finished");
        if self.step.test_or_set("FINISHED") { return Ok(()); }
        on_fail!(thread.send_message(&ctx.http, self.create_message(&Username::from_user(user))).await, "Failed to send final message")?;
        Ok(())
    }
    pub fn receive_message(&mut self, message: &Message) {
        match self.step.value() {
            "TITLE" => {
                self.title = Some(message.content.clone());
            }
            "DESCRIPTION" => {
                self.description = Some(message.content.clone());
            }
            "CONTACT_OTHER" => {
                self.contact = Some(Contact::Other(Some(message.content.clone())));
            }
            "URLS" => {
                self.other_urls = Some(vec![message.content.clone()]);
            }
            _ => {
                if let Some(what) = &mut self.is_recruiter {
                    match what {
                        What::Recruiter(infos) => { infos.receive_message(message); }
                        What::Worker(infos) => { infos.receive_message(message); }
                    }
                }
                if let Some(kind) = &mut self.kind {
                    match kind {
                        Contract::Volunteering(infos) => { infos.receive_message(message); }
                        Contract::Internship(infos) => { infos.receive_message(message); }
                        Contract::Freelance(infos) => { infos.receive_message(message); }
                        Contract::WorkStudy(infos) => { infos.receive_message(message); }
                        Contract::FixedTerm(infos) => { infos.receive_message(message); }
                        Contract::OpenEnded(infos) => { infos.receive_message(message); }
                    }
                }
            }
        }
    }

    pub fn clicked_button(&mut self, action: &str) {
        match action {
            "looking_job" => {
                self.is_recruiter = Some(What::Worker(WorkerInfos::default()));
            }
            "looking_employee" => {
                self.is_recruiter = Some(What::Recruiter(RecruiterInfos::default()));
            }
            /***************************/
            "job_freelance" => {
                self.kind = Some(Contract::Freelance(FreelanceInfos::default()));
            }
            "job_volunteering" => {
                self.kind = Some(Contract::Volunteering(VolunteeringInfos::default()));
            }
            "job_internship" => {
                self.kind = Some(Contract::Internship(InternshipInfos::default()));
            }
            "job_workstudy" => {
                self.kind = Some(Contract::WorkStudy(WorkStudyInfos::default()));
            }
            "job_fixedterm" => {
                self.kind = Some(Contract::FixedTerm(FixedTermInfos::default()));
            }
            "job_openended" => {
                self.kind = Some(Contract::OpenEnded(OpenEndedInfos::default()));
            }
            /***************************/
            "contact_discord" => {
                self.contact = Some(Contact::Discord);
            }
            "contact_other" => {
                self.contact = Some(Contact::Other(None));
            }
            /***************************/
            "edit_title" => {
                self.title = None;
            }
            "edit_description" => {
                self.description = None;
            }
            "edit_urls" => {
                self.other_urls = None;
            }
            &_ => {
                if let Some(what) = &mut self.is_recruiter {
                    match what {
                        What::Recruiter(infos) => { infos.clicked_button(action); }
                        What::Worker(infos) => { infos.clicked_button(action); }
                    }
                }

                if let Some(kind) = &mut self.kind {
                    match kind {
                        Contract::Volunteering(infos) => { infos.clicked_button(action); }
                        Contract::Internship(infos) => { infos.clicked_button(action); }
                        Contract::Freelance(infos) => { infos.clicked_button(action); }
                        Contract::WorkStudy(infos) => { infos.clicked_button(action); }
                        Contract::FixedTerm(infos) => { infos.clicked_button(action); }
                        Contract::OpenEnded(infos) => { infos.clicked_button(action); }
                    }
                }
            }
        }
    }


    fn create_message(&self, author: &Username) -> CreateMessage {
        let mut message = CreateMessage::new();


        let title = match &self.title {
            None => { "[Titre manquant]" }
            Some(title) => { title.as_str() }
        };

        let description = match &self.description {
            None => { "[Aucune description]" }
            Some(title) => { title.as_str() }
        };

        let mut main_embed = CreateEmbed::new()
            .title(title)
            .description(format!("Annonce de {}:\n{}", author.full(), description));

        if let Some(kind) = &self.kind {
            match kind {
                Contract::Volunteering(_) => {}
                Contract::Internship(_) => {}
                Contract::Freelance(infos) => {
                    main_embed = main_embed.field("Durée", match &infos.duration {
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