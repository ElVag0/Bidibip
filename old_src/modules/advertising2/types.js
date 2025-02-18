
class AdvertisingKinds {
    static FREELANCE = {type: 0, text: "Freelance", text_small: "Freelance", emote: '🧐', color: '#861de1'};
    static UNPAID = {type: 1, text: "Bénévolat (non rémunéré)", text_small: "Bénévolat", emote: '🤝', color: '#c02727'};
    static INTERN_FREE = {type: 2, text: "Stage (non rémunéré)", text_small: "Stage", emote: '🪂', color: '#c05910'};
    static INTERN_PAID = {type: 3, text: "Stage (rémunéré)", text_small: "Stage", emote: '👨‍🎓', color: '#d0cd0d'};
    static WORK_STUDY = {type: 4, text: "Alternance (rémunéré)", text_small: "Alternance", emote: '🤓', color: '#96cb11'};
    static PAID_LIMITED = {type: 5, text: "CDD (rémunéré)", text_small: "CDD", emote: '😎', color: '#1ad50d'};
    static PAID_UNLIMITED = {type: 6, text: "CDI (rémunéré)", text_small: "CDI", emote: '🤯', color: '#0deaea'};

    /**
     * @return {(string)[]}
     */
    static allTexts() {
        return [
            this.FREELANCE.emote + ' ' + this.FREELANCE.text,
            this.UNPAID.emote + ' ' + this.UNPAID.text,
            this.INTERN_FREE.emote + ' ' + this.INTERN_FREE.text,
            this.INTERN_PAID.emote + ' ' + this.INTERN_PAID.text,
            this.WORK_STUDY.emote + ' ' + this.WORK_STUDY.text,
            this.PAID_LIMITED.emote + ' ' + this.PAID_LIMITED.text,
            this.PAID_UNLIMITED.emote + ' ' + this.PAID_UNLIMITED.text
        ]
    }

    static getTags(kind) {
        const tags = [];
        switch (kind) {
            case AdvertisingKinds.INTERN_FREE.type:
                tags.push("Stage");
                tags.push("Non rémunéré");
                break;
            case AdvertisingKinds.UNPAID.type:
                tags.push("Bénévolat");
                tags.push("Non rémunéré");
                break;
            case AdvertisingKinds.INTERN_PAID.type:
                tags.push("Stage");
                tags.push("Contrat rémunéré");
                break;
            case AdvertisingKinds.WORK_STUDY.type:
                tags.push("Alternance");
                tags.push("Contrat rémunéré");
                break;
            case AdvertisingKinds.PAID_LIMITED.type:
                tags.push("CDD");
                tags.push("Contrat rémunéré");
                break;
            case AdvertisingKinds.PAID_UNLIMITED.type:
                tags.push("CDI");
                tags.push("Contrat rémunéré");
                break;
            case AdvertisingKinds.FREELANCE.type:
                tags.push("Freelance");
                tags.push("Contrat rémunéré");
                break;
        }
        return tags;
    }

    static get(number) {
        switch (number) {
            case 0:
                return AdvertisingKinds.FREELANCE;
            case 1:
                return AdvertisingKinds.UNPAID;
            case 2:
                return AdvertisingKinds.INTERN_FREE;
            case 3:
                return AdvertisingKinds.INTERN_PAID;
            case 4:
                return AdvertisingKinds.WORK_STUDY;
            case 5:
                return AdvertisingKinds.PAID_LIMITED;
            case 6:
                return AdvertisingKinds.PAID_UNLIMITED;
        }
    }
}

module.exports = {AdvertisingKinds}