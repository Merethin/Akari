use lazy_static::lazy_static;
use regex::{Captures, Error, Regex, RegexSet};
use std::collections::HashMap;

use crate::events::Event;

pub type ProcessorExtFn = fn(&mut Event, Captures<'_>, &[&str]);

pub enum Field {
    Actor(usize),
    Receptor(usize),
    Origin(usize),
    Destination(usize),
    Data(Vec<usize>),
    BucketOrigin,
}

use Field::*;

pub struct Processor {
    fields: Vec<Field>,
    custom: Option<ProcessorExtFn>,
}

impl Processor {
    pub fn apply(&self, event: &mut Event, captures: Captures<'_>, regions: &[&str]) {
        for field in &self.fields {
            match field {
                Actor(i) => event.actor = Some(captures[*i].to_owned()),
                Receptor(i) => event.receptor = Some(captures[*i].to_owned()),
                Origin(i) => event.origin = Some(captures[*i].to_owned()),
                Destination(i) => event.destination = Some(captures[*i].to_owned()),
                Data(indexes) => {
                    for i in indexes {
                        if let Some(m) = captures.get(*i) {
                            event.data.push(m.as_str().to_owned());
                        }
                    }
                },
                BucketOrigin => event.origin = Some(regions.first().unwrap_or(&"[unknown]").to_string())
            }
        }

        if let Some(func) = self.custom {
            func(event, captures, regions);
        }
    }

    pub fn init(fields: Vec<Field>, custom: ProcessorExtFn) -> Self {
        Processor { fields, custom: Some(custom) }
    }
}

impl From<Vec<Field>> for Processor {
    fn from(fields: Vec<Field>) -> Self {
        Self { fields, custom: None }
    }
}

pub struct Happenings {
    pub regexes: Vec<(&'static str, Regex)>,
    pub set: RegexSet,
    pub map: HashMap<&'static str, Processor>,
}

pub fn generate_happenings() -> Result<Happenings, Box<Error>> {
    let regexes: Vec<(&'static str, Regex)> = vec![
        // bucket: law
        ("law", Regex::new(r#"^Following new legislation in @@([0-9a-z_-]+)@@, (.+)$"#)?),
        // bucket: change
        ("chclass", Regex::new(r#"^@@([0-9a-z_-]+)@@ was reclassified from "([A-Za-z -]+)" to "([A-Za-z -]+)"$"#)?),
        ("chcensus", Regex::new(r#"^@@([0-9a-z_-]+)@@ was ranked in((?:,? (?:and )?the Top (?:1|5|10)% (?:of the world )?for(?:(?:,? (?:and )?(?:(?:[A-Z][A-Za-z-]+ ?)+))*))+)$"#)?),
        ("chfield", Regex::new(r#"^@@([0-9a-z_-]+)@@ changed its national ([a-z ]+) to "([^"]*)"((?:,? (?:and )?its [a-z ]+ to "[^"]*")+)?$"#)?),
        ("chflag", Regex::new(r#"^@@([0-9a-z_-]+)@@ altered its national flag$"#)?),
        ("nbanner", Regex::new(r#"^@@([0-9a-z_-]+)@@ created a custom banner$"#)?),
        ("chbanner", Regex::new(r#"^@@([0-9a-z_-]+)@@ changed a custom banner$"#)?),
        ("chinf", Regex::new(r#"^@@([0-9a-z_-]+)@@'s influence in %%([0-9a-z_-]+)%% (rose|fell) from "([A-Za-z -]+)" to "([A-Za-z -]+)"$"#)?),
        ("rvfield", Regex::new(r#"^@@([0-9a-z_-]+)@@ revoked its national (faith|leader|capital)$"#)?),
        // bucket: dispatch
        ("dispatch", Regex::new(r#"^@@([0-9a-z_-]+)@@ published "<a href="page=dispatch/id=([0-9]+)">([^><]+)</a>" \(([A-Za-z]+): ([A-Za-z]+)\)$"#)?),
        // bucket: rmb
        ("rmbpost", Regex::new(r#"^@@([0-9a-z_-]+)@@ lodged <a href="/region=(?:[0-9a-z_-]+)/page=display_region_rmb\?postid=(?:[0-9]+)#p([0-9]+)">a message</a> on the %%([0-9a-z_-]+)%% Regional Message Board$"#)?),
        ("rmbnsupp", Regex::new(r#"^@@([0-9a-z_-]+)@@ suppressed a post on the %%([0-9a-z_-]+)%% Regional Message Board$"#)?),
        ("rmbrsupp", Regex::new(r#"^@@([0-9a-z_-]+)@@ unsuppressed a post on the %%([0-9a-z_-]+)%% Regional Message Board$"#)?),
        // bucket: embassy
        ("ereq", Regex::new(r#"^@@([0-9a-z_-]+)@@ proposed constructing embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$"#)?),
        ("eaccept", Regex::new(r#"^@@([0-9a-z_-]+)@@ agreed to construct embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$"#)?),
        ("ecancel", Regex::new(r#"^@@([0-9a-z_-]+)@@ cancelled the closure of embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$"#)?),
        ("ewish", Regex::new(r#"^@@([0-9a-z_-]+)@@ indicated that %%([0-9a-z_-]+)%% did not wish to close its embassy with %%([0-9a-z_-]+)%%$"#)?),
        ("ereject", Regex::new(r#"^@@([0-9a-z_-]+)@@ rejected a request from %%([0-9a-z_-]+)%% for an embassy with %%([0-9a-z_-]+)%%$"#)?),
        ("eclose", Regex::new(r#"^@@([0-9a-z_-]+)@@ ordered the closure of embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$"#)?),
        ("epull", Regex::new(r#"^@@([0-9a-z_-]+)@@ withdrew a request for embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$"#)?),
        ("eabort", Regex::new(r#"^@@([0-9a-z_-]+)@@ aborted construction of embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$"#)?),
        ("eufinish", Regex::new(r#"^Embassy established between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$"#)?),
        ("euclose", Regex::new(r#"^Embassy cancelled between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$"#)?),
        ("euabort", Regex::new(r#"^Construction of embassies aborted between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$"#)?),
        // bucket: eject
        ("eject", Regex::new(r#"^@@([0-9a-z_-]+)@@ was ejected from %%([0-9a-z_-]+)%% by @@([0-9a-z_-]+)@@$"#)?),
        ("banject", Regex::new(r#"^@@([0-9a-z_-]+)@@ was ejected and banned from %%([0-9a-z_-]+)%% by @@([0-9a-z_-]+)@@$"#)?),
        // bucket: admin
        ("ban", Regex::new(r#"^@@([0-9a-z_-]+)@@ banned @@([0-9a-z_-]+)@@ from %%([0-9a-z_-]+)%%$"#)?),
        ("rcvban", Regex::new(r#"^@@([0-9a-z_-]+)@@ was banned from %%([0-9a-z_-]+)%% by @@([0-9a-z_-]+)@@$"#)?),
        ("unban", Regex::new(r#"^@@([0-9a-z_-]+)@@ removed @@([0-9a-z_-]+)@@ from the regional ban list in %%([0-9a-z_-]+)%%$"#)?),
        ("rcvunban", Regex::new(r#"^@@([0-9a-z_-]+)@@ was removed from the regional ban list of %%([0-9a-z_-]+)%% by @@([0-9a-z_-]+)@@$"#)?),
        ("setpw", Regex::new(r#"^@@([0-9a-z_-]+)@@ password-protected %%([0-9a-z_-]+)%%$"#)?),
        ("changepw", Regex::new(r#"^@@([0-9a-z_-]+)@@ changed the regional password in %%([0-9a-z_-]+)%%$"#)?),
        ("rmpw", Regex::new(r#"^@@([0-9a-z_-]+)@@ removed regional password protection from %%([0-9a-z_-]+)%%$"#)?),
        ("rupdate", Regex::new(r#"^%%([0-9a-z_-]+)%% updated$"#)?),
        ("rfeature", Regex::new(r#"^%%([0-9a-z_-]+)%% became the Featured Region of the day$"#)?),
        ("rfound", Regex::new(r#"^@@([0-9a-z_-]+)@@ founded the region %%([0-9a-z_-]+)%%$"#)?),
        ("srbanner", Regex::new(r#"^@@([0-9a-z_-]+)@@ set the regional banner of %%([0-9a-z_-]+)%%$"#)?),
        ("crbanner", Regex::new(r#"^@@([0-9a-z_-]+)@@ changed the regional banner of %%([0-9a-z_-]+)%%$"#)?),
        ("crflag", Regex::new(r#"^@@([0-9a-z_-]+)@@ altered the regional flag of %%([0-9a-z_-]+)%%$"#)?),
        ("rrflag", Regex::new(r#"^@@([0-9a-z_-]+)@@ abolished the regional flag of %%([0-9a-z_-]+)%%$"#)?),
        ("rmpoll", Regex::new(r#"^@@([0-9a-z_-]+)@@ deleted a regional poll in %%([0-9a-z_-]+)%%$"#)?),
        ("addtag", Regex::new(r#"^@@([0-9a-z_-]+)@@ added the tag "([^"]+)" to %%([0-9a-z_-]+)%%$"#)?),
        ("rmtag", Regex::new(r#"^@@([0-9a-z_-]+)@@ removed the tag "([^"]+)" from %%([0-9a-z_-]+)%%$"#)?),
        ("roadd", Regex::new(r#"^@@([0-9a-z_-]+)@@ appointed @@([0-9a-z_-]+)@@ as (.+) with authority over (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?)*) in %%([0-9a-z_-]+)%%$"#)?),
        ("rorename", Regex::new(r#"^@@([0-9a-z_-]+)@@ renamed the office held by @@([0-9a-z_-]+)@@ from "(.+)" to "(.+)" in %%([0-9a-z_-]+)%%$"#)?),
        ("rochange", Regex::new(r#"^@@([0-9a-z_-]+)@@ (granted|removed) (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?)*) authority (?:and removed (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?)*) authority )?(?:from|to) @@([0-9a-z_-]+)@@ as (.+) in %%([0-9a-z_-]+)%%$"#)?),
        ("rochname", Regex::new(r#"^@@([0-9a-z_-]+)@@ (granted|removed) (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?)*) authority (?:and removed (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?)*) authority )?(?:from|to) @@([0-9a-z_-]+)@@ and renamed the office from "(.+)" to "(.+)" in %%([0-9a-z_-]+)%%$"#)?),
        ("roremove", Regex::new(r#"^@@([0-9a-z_-]+)@@ dismissed @@([0-9a-z_-]+)@@ as (.+) of %%([0-9a-z_-]+)%%$"#)?),
        ("roresign", Regex::new(r#"^@@([0-9a-z_-]+)@@ resigned as (.+) of %%([0-9a-z_-]+)%%$"#)?),
        ("rgovtset", Regex::new(r#"^@@([0-9a-z_-]+)@@ named the Governor's office  <b>(.+)</b> in %%([0-9a-z_-]+)%%$"#)?),
        ("rgovtupd", Regex::new(r#"^@@([0-9a-z_-]+)@@ renamed the Governor's office from "(.+)" to  <b>(.+)</b> in %%([0-9a-z_-]+)%%$"#)?),
        ("rdelauth", Regex::new(r#"^@@([0-9a-z_-]+)@@ (granted|removed) (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+)*) authority (?:and removed (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+)*) authority )?(?:from|to) the WA Delegate (?:@@([0-9a-z_-]+)@@ )?in %%([0-9a-z_-]+)%%$"#)?),
        ("rnewgov", Regex::new(r#"^@@([0-9a-z_-]+)@@ succeeded @@([0-9a-z_-]+)@@ as Governor of %%([0-9a-z_-]+)%%$"#)?),
        ("rsucprio", Regex::new(r#"^@@([0-9a-z_-]+)@@ increased @@([0-9a-z_-]+)@@'s succession priority in %%([0-9a-z_-]+)%%$"#)?),
        ("nwelcome", Regex::new(r#"^@@([0-9a-z_-]+)@@ composed a new Welcome Telegram for %%([0-9a-z_-]+)%%$"#)?),
        ("rwelcome", Regex::new(r#"^@@([0-9a-z_-]+)@@ canceled the Welcome Telegram of %%([0-9a-z_-]+)%%$"#)?),
        ("rwfe", Regex::new(r#"^@@([0-9a-z_-]+)@@ updated the World Factbook entry in %%([0-9a-z_-]+)%%$"#)?),
        ("amapwf", Regex::new(r#"^@@([0-9a-z_-]+)@@ added the most supported regional map to the world factbook$"#)?),
        ("rmapwf", Regex::new(r#"^@@([0-9a-z_-]+)@@ removed the most supported regional map from the world factbook$"#)?),
        ("ndel", Regex::new(r#"^@@([0-9a-z_-]+)@@ became WA Delegate of %%([0-9a-z_-]+)%%$"#)?),
        ("rdel", Regex::new(r#"^@@([0-9a-z_-]+)@@ seized the position of %%([0-9a-z_-]+)%% WA Delegate from @@([0-9a-z_-]+)@@$"#)?),
        ("ldel", Regex::new(r#"^@@([0-9a-z_-]+)@@ lost WA Delegate status in %%([0-9a-z_-]+)%%$"#)?),
        ("beginfn", Regex::new(r#"^@@([0-9a-z_-]+)@@ began the process of converting %%([0-9a-z_-]+)%% to a Frontier$"#)?),
        ("stopfn", Regex::new(r#"^@@([0-9a-z_-]+)@@ canceled the process of converting %%([0-9a-z_-]+)%% to a Frontier$"#)?),
        ("finishfn", Regex::new(r#"^%%([0-9a-z_-]+)%% became a Frontier$"#)?),
        ("finishfn", Regex::new(r#"^Became a Frontier$"#)?),
        ("fngovrem", Regex::new(r#"^@@([0-9a-z_-]+)@@ stepped down as Governor of %%([0-9a-z_-]+)%% as it became a Frontier$"#)?),
        ("beginst", Regex::new(r#"^@@([0-9a-z_-]+)@@ began the process of removing %%([0-9a-z_-]+)%%'s designation as a Frontier$"#)?),
        ("stopst", Regex::new(r#"^@@([0-9a-z_-]+)@@ canceled the process of removing %%([0-9a-z_-]+)%%'s designation as a Frontier$"#)?),
        ("finishst", Regex::new(r#"^%%([0-9a-z_-]+)%% ceased to operate as a Frontier$"#)?),
        ("finishst", Regex::new(r#"^Ceased to operate as a Frontier$"#)?),
        ("stgovadd", Regex::new(r#"^@@([0-9a-z_-]+)@@ became Governor of %%([0-9a-z_-]+)%%$"#)?),
        ("annexreq", Regex::new(r#"^@@([0-9a-z_-]+)@@ sent a demand to annex %%([0-9a-z_-]+)%%$"#)?),
        ("annexrcv", Regex::new(r#"^%%([0-9a-z_-]+)%% received a demand from @@([0-9a-z_-]+)@@ to be annexed by %%([0-9a-z_-]+)%%$"#)?),
        ("annexrej", Regex::new(r#"^@@([0-9a-z_-]+)@@ rejected a demand for %%([0-9a-z_-]+)%% to be annexed into %%([0-9a-z_-]+)%%$"#)?),
        ("annexacc", Regex::new(r#"^@@([0-9a-z_-]+)@@ accepted a demand to be annexed by %%([0-9a-z_-]+)%%$"#)?),
        ("addxrmb", Regex::new(r#"^@@([0-9a-z_-]+)@@ granted posting privileges on the %%([0-9a-z_-]+)%% Regional Message Board to ([a-zA-Z ]+) in embassy regions$"#)?),
        ("remxrmb", Regex::new(r#"^@@([0-9a-z_-]+)@@ revoked posting privileges on the %%([0-9a-z_-]+)%% Regional Message Board from ([a-zA-Z ]+) in embassy regions$"#)?),
        // bucket: maps
        ("mcreate", Regex::new(r#"^@@([0-9a-z_-]+)@@ created &&([0-9a-z_-]+)&&$"#)?),
        ("mvcreate", Regex::new(r#"^@@([0-9a-z_-]+)@@ created \*\*([0-9a-z_-]+)\*\*$"#)?),
        ("mupdate", Regex::new(r#"^@@([0-9a-z_-]+)@@ updated &&([0-9a-z_-]+)&& to \*\*([0-9a-z_-]+)\*\*$"#)?),
        ("mendo", Regex::new(r#"^@@([0-9a-z_-]+)@@ endorsed &&([0-9a-z_-]+)&&$"#)?),
        ("mrendo", Regex::new(r#"^@@([0-9a-z_-]+)@@ endorsed &&([0-9a-z_-]+)&& instead of &&([0-9a-z_-]+)&&$"#)?),
        ("mlendo", Regex::new(r#"^&&([0-9a-z_-]+)&& lost the endorsement of @@([0-9a-z_-]+)@@$"#)?),
        ("munendo", Regex::new(r#"^@@([0-9a-z_-]+)@@ removed its endorsement from &&([0-9a-z_-]+)&&$"#)?),
        // bucket: move
        ("move", Regex::new(r#"^@@([0-9a-z_-]+)@@ relocated from %%([0-9a-z_-]+)%% to %%([0-9a-z_-]+)%%$"#)?),
        // bucket: found
        ("nfound", Regex::new(r#"^@@([0-9a-z_-]+)@@ was founded in %%([0-9a-z_-]+)%%$"#)?),
        ("nrefound", Regex::new(r#"^@@([0-9a-z_-]+)@@ was refounded in %%([0-9a-z_-]+)%%$"#)?),
        // bucket: cte
        ("ncte", Regex::new(r#"^@@([0-9a-z_-]+)@@ ceased to exist in %%([0-9a-z_-]+)%%$"#)?),
        ("rgcte", Regex::new(r#"^Governor @@([0-9a-z_-]+)@@ ceased to exist$"#)?),
        ("rfcte", Regex::new(r#"^Regional Founder @@([0-9a-z_-]+)@@ ceased to exist$"#)?),
        // bucket: vote
        ("wavote", Regex::new(r#"^@@([0-9a-z_-]+)@@ voted (for|against) the World Assembly Resolution "(.+)"$"#)?),
        ("wrvote", Regex::new(r#"^@@([0-9a-z_-]+)@@ withdrew its vote on the World Assembly Resolution "(.+)"$"#)?),
        // bucket: resolution
        ("rsfloor", Regex::new(r#"^The (General Assembly|Security Council) proposal "(.+)" \(by @@([0-9a-z_-]+)@@((?:,?( and)? @@([0-9a-z_-]+)@@)*)?\) entered the resolution voting floor$"#)?),
        ("rspass", Regex::new(r#"^The (General Assembly|Security Council) resolution <strong><a href="/page=WA_past_resolution/id=([0-9]+)/council=(?:1|2)">(.+)</a></strong> was passed ([0-9,]+) votes to ([0-9,]+)$"#)?),
        ("rsfail", Regex::new(r#"^The (General Assembly|Security Council) resolution "<strong>(.+)</strong>" was defeated ([0-9,]+) votes to ([0-9,]+)$"#)?),
        ("rdiscard", Regex::new(r#"^The (General Assembly|Security Council) resolution "<strong>(.+)</strong>" was discarded by the WA for rule violations after garnering ([0-9,]+) votes in favor and ([0-9,]+) votes against$"#)?),
        ("rsapp", Regex::new(r#"^@@([0-9a-z_-]+)@@ approved the World Assembly proposal "(.+)"$"#)?),
        ("rsremapp", Regex::new(r#"^@@([0-9a-z_-]+)@@ withdrew its approval for the World Assembly proposal "(.+)"$"#)?),
        ("rssubmit", Regex::new(r#"^@@([0-9a-z_-]+)@@ submitted a proposal to the (General Assembly|Security Council) (.+) Board entitled "(.+)"$"#)?),
        ("rsremsub", Regex::new(r#"^@@([0-9a-z_-]+)@@ withdrew a proposal from the WA (General Assembly|Security Council) titled "(.+)"$"#)?),
        ("rsquorum", Regex::new(r#"^The (General Assembly|Security Council) proposal "(.+)" \[@@([0-9a-z_-]+)@@\] failed to achieve quorum$"#)?),
        // bucket: member
        ("wadmit", Regex::new(r#"^@@([0-9a-z_-]+)@@ was admitted to the World Assembly$"#)?),
        ("wapply", Regex::new(r#"^@@([0-9a-z_-]+)@@ applied to join the World Assembly$"#)?),
        ("wresign", Regex::new(r#"^@@([0-9a-z_-]+)@@ resigned from the World Assembly$"#)?),
        ("wkick", Regex::new(r#"^@@([0-9a-z_-]+)@@ was ejected from the WA for rule violations$"#)?),
        // bucket: endo
        ("wendo", Regex::new(r#"^@@([0-9a-z_-]+)@@ endorsed @@([0-9a-z_-]+)@@$"#)?),
        ("wunendo", Regex::new(r#"^@@([0-9a-z_-]+)@@ withdrew its endorsement from @@([0-9a-z_-]+)@@$"#)?),
        // bucket: all
        ("govabd", Regex::new(r#"^Governor @@([0-9a-z_-]+)@@ abdicated$"#)?),
        ("npoll", Regex::new(r#"^@@([0-9a-z_-]+)@@ created a new poll in %%([0-9a-z_-]+)%%: "(.+)"$"#)?),
        ("modkick", Regex::new(r#"^@@([0-9a-z_-]+)@@ was removed from %%([0-9a-z_-]+)%% by moderation$"#)?),
        ("nrspass", Regex::new(r#"^@@([0-9a-z_-]+)@@'s resolution <a href="/page=WA_past_resolution/id=([0-9]+)/council=(?:1|2)">(.+)</a> was passed by the (General Assembly|Security Council)$"#)?),
        ("nscnom", Regex::new(r#"^@@([0-9a-z_-]+)@@ was nominated for a World Assembly (Commendation|Condemnation) by @@([0-9a-z_-]+)@@$"#)?),
        ("rscnom", Regex::new(r#"^%%([0-9a-z_-]+)%% was nominated for a World Assembly (Commendation|Condemnation|Liberation|Injunction) by @@([0-9a-z_-]+)@@$"#)?),
        ("nscpass", Regex::new(r#"^@@([0-9a-z_-]+)@@ was (commended|condemned) by <a href="/page=WA_past_resolution/id=(?:[0-9]+)/council=2">Security Council Resolution # ([0-9]+)</a>$"#)?),
        ("rscpass", Regex::new(r#"^%%([0-9a-z_-]+)%% was (commended|condemned|liberated|injuncted) by <a href="/page=WA_past_resolution/id=(?:[0-9]+)/council=2">Security Council Resolution # ([0-9]+)</a>$"#)?),
    ];

    let set = RegexSet::new(
        regexes.iter().map(|(_, pattern)| { pattern.as_str() })
    )?;

    let map = generate_processor_map();

    Ok(Happenings {
        regexes,
        set,
        map
    })
}

fn generate_processor_map() -> HashMap<&'static str, Processor> {
    let mut map: HashMap<&'static str, Processor> = HashMap::new();

    // bucket: law
    map.insert("law", vec![BucketOrigin, Actor(1), Data(vec![2])].into());
    // bucket: change
    map.insert("chclass", vec![BucketOrigin, Receptor(1), Data(vec![2,3])].into());
    map.insert("chcensus", Processor::init(vec![BucketOrigin, Receptor(1)], chcensus_ext));
    map.insert("chfield", Processor::init(vec![BucketOrigin, Actor(1), Data(vec![2,3])], chfield_ext));
    map.insert("chflag", vec![BucketOrigin, Actor(1)].into());
    map.insert("nbanner", vec![BucketOrigin, Actor(1)].into());
    map.insert("chbanner", vec![BucketOrigin, Actor(1)].into());
    map.insert("chinf", vec![Receptor(1), Origin(2), Data((3..6).collect())].into());
    map.insert("rvfield", vec![BucketOrigin, Actor(1), Data(vec![2])].into());
    // bucket: dispatch
    map.insert("dispatch", vec![BucketOrigin, Actor(1), Data((2..6).collect())].into());
    // bucket: rmb
    map.insert("rmbpost", vec![Actor(1), Data(vec![2]), Origin(3)].into());
    map.insert("rmbnsupp", vec![Actor(1), Origin(2)].into());
    map.insert("rmbrsupp", vec![Actor(1), Origin(2)].into());
    // bucket: embassy
    map.insert("ereq", vec![Actor(1), Origin(2), Destination(3)].into());
    map.insert("eaccept", vec![Actor(1), Origin(2), Destination(3)].into());
    map.insert("ecancel", vec![Actor(1), Origin(2), Destination(3)].into());
    map.insert("ewish", vec![Actor(1), Origin(2), Destination(3)].into());
    map.insert("ereject", vec![Actor(1), Destination(2), Origin(3)].into());
    map.insert("eclose", vec![Actor(1), Origin(2), Destination(3)].into());
    map.insert("epull", vec![Actor(1), Origin(2), Destination(3)].into());
    map.insert("eabort", vec![Actor(1), Origin(2), Destination(3)].into());
    map.insert("eufinish", vec![Origin(1), Destination(2)].into());
    map.insert("euclose", vec![Origin(1), Destination(2)].into());
    map.insert("euabort", vec![Origin(1), Destination(2)].into());
    // bucket: eject
    map.insert("eject", vec![Receptor(1), Origin(2), Actor(3)].into());
    map.insert("banject", vec![Receptor(1), Origin(2), Actor(3)].into());
    // bucket: admin
    map.insert("ban", vec![Actor(1), Receptor(2), Origin(3)].into());
    map.insert("rcvban", vec![Receptor(1), Origin(2), Actor(3)].into());
    map.insert("unban", vec![Actor(1), Receptor(2), Origin(3)].into());
    map.insert("rcvunban", vec![Receptor(1), Origin(2), Actor(3)].into());
    map.insert("setpw", vec![Actor(1), Origin(2)].into());
    map.insert("changepw", vec![Actor(1), Origin(2)].into());
    map.insert("rmpw", vec![Actor(1), Origin(2)].into());
    map.insert("rupdate", vec![Origin(1)].into());
    map.insert("rfeature", vec![Origin(1)].into());
    map.insert("rfound", vec![Actor(1), Origin(2)].into());
    map.insert("srbanner", vec![Actor(1), Origin(2)].into());
    map.insert("crbanner", vec![Actor(1), Origin(2)].into());
    map.insert("crflag", vec![Actor(1), Origin(2)].into());
    map.insert("rrflag", vec![Actor(1), Origin(2)].into());
    map.insert("rmpoll", vec![Actor(1), Origin(2)].into());
    map.insert("addtag", vec![Actor(1), Data(vec![2]), Origin(3)].into());
    map.insert("rmtag", vec![Actor(1), Data(vec![2]), Origin(3)].into());
    map.insert("roadd", Processor::init(vec![Actor(1), Receptor(2), Data(vec![3]), Origin(5)], roadd_ext));
    map.insert("rorename", vec![Actor(1), Receptor(2), Data(vec![3,4]), Origin(5)].into());
    map.insert("rochange", Processor::init(vec![Actor(1), Receptor(5), Data(vec![6]), Origin(7)], rochange_ext));
    map.insert("rochname", Processor::init(vec![Actor(1), Receptor(5), Data(vec![6,7]), Origin(8)], rochange_ext));
    map.insert("roremove", vec![Actor(1), Receptor(2), Data(vec![3]), Origin(4)].into());
    map.insert("roresign", vec![Actor(1), Data(vec![2]), Origin(3)].into());
    map.insert("rgovtset", vec![Actor(1), Data(vec![2]), Origin(3)].into());
    map.insert("rgovtupd", vec![Actor(1), Data(vec![2,3]), Origin(4)].into());
    map.insert("rdelauth", Processor::init(vec![Actor(1), Origin(6)], rdelauth_ext));
    map.insert("rnewgov", vec![Receptor(1), Actor(2), Origin(3)].into());
    map.insert("rsucprio", vec![Actor(1), Receptor(2), Origin(3)].into());
    map.insert("nwelcome", vec![Actor(1), Origin(2)].into());
    map.insert("rwelcome", vec![Actor(1), Origin(2)].into());
    map.insert("rwfe", vec![Actor(1), Origin(2)].into());
    map.insert("amapwf", vec![BucketOrigin, Actor(1)].into());
    map.insert("rmapwf", vec![BucketOrigin, Actor(1)].into());
    map.insert("ndel", vec![Receptor(1), Origin(2)].into());
    map.insert("rdel", vec![Receptor(1), Origin(2), Data(vec![3])].into());
    map.insert("ldel", vec![Receptor(1), Origin(2)].into());
    map.insert("beginfn", vec![Actor(1), Origin(2)].into());
    map.insert("stopfn", vec![Actor(1), Origin(2)].into());
    map.insert("finishfn", Processor::init(vec![], finishfs_ext));
    map.insert("fngovrem", vec![Receptor(1), Origin(2)].into());
    map.insert("beginst", vec![Actor(1), Origin(2)].into());
    map.insert("stopst", vec![Actor(1), Origin(2)].into());
    map.insert("finishst", Processor::init(vec![], finishfs_ext));
    map.insert("stgovadd", vec![Receptor(1), Origin(2)].into());
    map.insert("annexreq", Processor::init(vec![Actor(1), Destination(2)], annexreq_ext));
    map.insert("annexrcv", vec![Destination(1), Actor(2), Origin(3)].into());
    map.insert("annexrej", vec![Actor(1), Origin(2), Destination(3)].into());
    map.insert("annexacc", Processor::init(vec![Actor(1), Destination(2)], annexacc_ext));
    map.insert("addxrmb", vec![Actor(1), Origin(2), Data(vec![3])].into());
    map.insert("remxrmb", vec![Actor(1), Origin(2), Data(vec![3])].into());
    // bucket: maps
    map.insert("mcreate", vec![BucketOrigin, Actor(1), Data(vec![2])].into());
    map.insert("mvcreate", vec![BucketOrigin, Actor(1), Data(vec![2])].into());
    map.insert("mupdate", vec![BucketOrigin, Actor(1), Data(vec![2,3])].into());
    map.insert("mendo", vec![BucketOrigin, Actor(1), Data(vec![2])].into());
    map.insert("mrendo", vec![BucketOrigin, Actor(1), Data(vec![2,3])].into());
    map.insert("mlendo", vec![BucketOrigin, Data(vec![1]), Actor(2)].into());
    map.insert("munendo", vec![BucketOrigin, Actor(1), Data(vec![2])].into());
    // bucket: move
    map.insert("move", vec![Actor(1), Origin(2), Destination(3)].into());
    // bucket: found
    map.insert("nfound", vec![Actor(1), Origin(2)].into());
    map.insert("nrefound", vec![Actor(1), Origin(2)].into());
    // bucket: cte
    map.insert("ncte", vec![Receptor(1), Origin(2)].into());
    map.insert("rgcte", vec![BucketOrigin, Receptor(1)].into());
    map.insert("rfcte", vec![BucketOrigin, Receptor(1)].into());
    // bucket: vote
    map.insert("wavote", vec![BucketOrigin, Actor(1), Data(vec![2,3])].into());
    map.insert("wrvote", vec![BucketOrigin, Actor(1), Data(vec![2])].into());
    // bucket: resolution
    map.insert("rsfloor", Processor::init(vec![Data(vec![1,2]), Receptor(3)], rsfloor_ext));
    map.insert("rspass", Processor::init(vec![Data(vec![1,2,3])], rspass_ext));
    map.insert("rsfail", Processor::init(vec![Data(vec![1,2])], rsfail_ext));
    map.insert("rdiscard", Processor::init(vec![Data(vec![1,2])], rsfail_ext));
    map.insert("rsapp", vec![Actor(1), Data(vec![2])].into());
    map.insert("rsremapp", vec![Actor(1), Data(vec![2])].into());
    map.insert("rssubmit", vec![Actor(1), Data(vec![2,3,4])].into());
    map.insert("rsremsub", vec![Actor(1), Data(vec![2,3])].into());
    map.insert("rsquorum", vec![Data(vec![1,2]), Receptor(3)].into());
    // bucket: member
    map.insert("wadmit", vec![BucketOrigin, Actor(1)].into());
    map.insert("wapply", vec![BucketOrigin, Actor(1)].into());
    map.insert("wresign", vec![BucketOrigin, Actor(1)].into());
    map.insert("wkick", vec![BucketOrigin, Actor(1)].into());
    // bucket: endo
    map.insert("wendo", vec![BucketOrigin, Actor(1), Receptor(2)].into());
    map.insert("wunendo", vec![BucketOrigin, Actor(1), Receptor(2)].into());
    // bucket: all
    map.insert("govabd", vec![BucketOrigin, Actor(1)].into());
    map.insert("npoll", vec![Actor(1), Origin(2), Data(vec![3])].into());
    map.insert("modkick", vec![Receptor(1), Origin(2)].into());
    map.insert("nrspass", vec![Receptor(1), Data(vec![4,2,3])].into());
    map.insert("nscnom", vec![BucketOrigin, Receptor(1), Data(vec![2]), Actor(3)].into());
    map.insert("rscnom", vec![Origin(1), Data(vec![2]), Actor(3)].into());
    map.insert("nscpass", vec![BucketOrigin, Receptor(1), Data(vec![2, 3])].into());
    map.insert("rscpass", vec![Origin(1), Data(vec![2, 3])].into());

    map
}

fn parse_census_labels(data: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"((?:[A-Z][A-Za-z-]+ ?)+)"#).unwrap();
    }

    RE.captures_iter(data).map(|m| {
        m[1].trim_end().to_owned()
    }).collect()
}

fn parse_census_percentages(data: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"the Top (1|5|10)% (?:of the world )?for((?:,? (?:and )?(?:(?:[A-Z][A-Za-z-]+ ?)+))*)"#).unwrap();
    }

    RE.captures_iter(data).map(|m| {
        let mut vec = vec![m[1].to_owned()];
        vec.append(&mut parse_census_labels(&m[2]));
        vec
    }).flatten().collect()
}

fn chcensus_ext(event: &mut Event, captures: Captures<'_>, _: &[&str]) {
    if let Some(census) = captures.get(2) {
        event.data.append(
            &mut parse_census_percentages(census.as_str())
        );
    }
}

fn parse_fields(fields: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#",? (?:and )?its ([a-z ]+) to "([^"]+)""#).unwrap();
    }

    RE.captures_iter(fields).map(|m| {
        (m[1].to_owned(), m[2].to_owned())
    }).flat_map(|(a, b)| [a, b]).collect()
}

fn chfield_ext(event: &mut Event, captures: Captures<'_>, _: &[&str]) {
    if let Some(extra_fields) = captures.get(4) {
        event.data.append(
            &mut parse_fields(extra_fields.as_str())
        );
    }
}

fn finishfs_ext(event: &mut Event, captures: Captures<'_>, regions: &[&str]) {
    if let Some(matched) = captures.get(1) {
        event.origin = Some(matched.as_str().to_owned());
    } else {
        event.origin = Some(regions.first().unwrap_or(&"[unknown]").to_string());
    }
}

fn parse_coauthors(coauthors: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"@@([0-9a-z_-]+)@@").unwrap();
    }

    RE.captures_iter(coauthors).map(|m| {
        m[1].to_owned()
    }).collect()
}

fn rsfloor_ext(event: &mut Event, captures: Captures<'_>, _: &[&str]) {
    if let Some(coauthors) = captures.get(4) {
        event.data.append(&mut parse_coauthors(coauthors.as_str()));
    }
}

fn rspass_ext(event: &mut Event, captures: Captures<'_>, _: &[&str]) {
    let votes_for = &captures[4];
    let votes_against = &captures[5];

    event.data.push(votes_for.replace(",", ""));
    event.data.push(votes_against.replace(",", ""));
}

fn rsfail_ext(event: &mut Event, captures: Captures<'_>, _: &[&str]) {
    let votes_for = &captures[3];
    let votes_against = &captures[4];

    event.data.push(votes_for.replace(",", ""));
    event.data.push(votes_against.replace(",", ""));
}

fn annexreq_ext(event: &mut Event, _: Captures<'_>, regions: &[&str]) {
    event.origin = Some(regions.iter().filter(
        |&region| region != &event.destination.clone().unwrap().as_str()
    ).collect::<Vec<&&str>>().first().unwrap_or(&&"[unknown]").to_string());
}

fn annexacc_ext(event: &mut Event, _: Captures<'_>, regions: &[&str]) {
    event.origin = Some(regions.iter().filter(
        |&region| region != &event.destination.clone().unwrap().as_str()
    ).collect::<Vec<&&str>>().first().unwrap_or(&&"[unknown]").to_string());
}

fn parse_authority(authority: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"</i>([A-Z])").unwrap();
    }

    RE.captures_iter(authority).map(|m| {
        m[1].to_owned()
    }).collect()
}

fn roadd_ext(event: &mut Event, captures: Captures<'_>, _: &[&str]) {
    let authority = &captures[4];
    event.data.push(parse_authority(authority));
}

fn rochange_ext(event: &mut Event, captures: Captures<'_>, _: &[&str]) {
    let mode = &captures[2];

    if mode == "granted" {
        event.data.push("+".to_owned() + &parse_authority(&captures[3]));

        if let Some(value) = captures.get(4) {
            event.data.push("-".to_owned() + &parse_authority(value.as_str()));
        }
    } else {
        event.data.push("-".to_owned() + &parse_authority(&captures[3]));
    }
}

fn rdelauth_ext(event: &mut Event, captures: Captures<'_>, _: &[&str]) {
    let mode = &captures[2];

    if mode == "granted" {
        event.data.push("+".to_owned() + &parse_authority(&captures[3]));

        if let Some(value) = captures.get(4) {
            event.data.push("-".to_owned() + &parse_authority(value.as_str()));
        }
    } else {
        event.data.push("-".to_owned() + &parse_authority(&captures[3]));
    }

    if let Some(value) = captures.get(5) {
        event.receptor = Some(value.as_str().to_owned());
    }
}