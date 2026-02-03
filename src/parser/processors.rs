use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;

use crate::events::ParsedEvent;

pub enum Field {
    Actor(usize),
    Receptor(usize),
    Origin(usize),
    Destination(usize),
    Data(Vec<usize>),
    BucketOrigin,
}

use Field::*;

pub fn generate_processor_map() -> HashMap<&'static str, Processor> {
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
    map.insert("rmapfeat", vec![Origin(1), Data(vec![2])].into());
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
    map.insert("finishfn", vec![Origin(1)].into());
    map.insert("fngovrem", vec![Receptor(1), Origin(2)].into());
    map.insert("beginst", vec![Actor(1), Origin(2)].into());
    map.insert("stopst", vec![Actor(1), Origin(2)].into());
    map.insert("finishst", vec![Origin(1)].into());
    map.insert("stgovadd", vec![Receptor(1), Origin(2)].into());
    map.insert("annexreq", vec![Actor(1), Destination(2)].into());
    map.insert("annexrcv", vec![Destination(1), Actor(2), Origin(3)].into());
    map.insert("annexrej", vec![Actor(1), Origin(2), Destination(3)].into());
    map.insert("annexacc", vec![Actor(1), Destination(2)].into());
    map.insert("annexwth", vec![Actor(1), Destination(2)].into());
    map.insert("annexfna", vec![Origin(1), Destination(2)].into());
    map.insert("annexfnb", vec![Origin(1), Destination(2)].into());
    map.insert("addxrmb", vec![Actor(1), Origin(2), Data(vec![3])].into());
    map.insert("remxrmb", vec![Actor(1), Origin(2), Data(vec![3])].into());
    map.insert("wzbanexp", vec![Origin(1)].into());
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
    map.insert("rsctg", vec![Origin(1), Data(vec![2]), Actor(3)].into());
    map.insert("nscpass", vec![BucketOrigin, Receptor(1), Data(vec![2, 3])].into());
    map.insert("rscpass", vec![Origin(1), Data(vec![2, 3])].into());

    map
}

pub type ProcessorExtFn = fn(&mut ParsedEvent, Captures<'_>, &[&str]);

pub struct Processor {
    fields: Vec<Field>,
    custom: Option<ProcessorExtFn>,
}

impl Processor {
    pub fn apply(&self, event: &mut ParsedEvent, captures: Captures<'_>, regions: &[&str]) {
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

    RE.captures_iter(data).flat_map(|m| {
        let mut vec = vec![m[1].to_owned()];
        vec.append(&mut parse_census_labels(&m[2]));
        vec
    }).collect()
}

fn chcensus_ext(event: &mut ParsedEvent, captures: Captures<'_>, _: &[&str]) {
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

fn chfield_ext(event: &mut ParsedEvent, captures: Captures<'_>, _: &[&str]) {
    if let Some(extra_fields) = captures.get(4) {
        event.data.append(
            &mut parse_fields(extra_fields.as_str())
        );
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

fn rsfloor_ext(event: &mut ParsedEvent, captures: Captures<'_>, _: &[&str]) {
    if let Some(coauthors) = captures.get(4) {
        event.data.append(&mut parse_coauthors(coauthors.as_str()));
    }
}

fn rspass_ext(event: &mut ParsedEvent, captures: Captures<'_>, _: &[&str]) {
    let votes_for = &captures[4];
    let votes_against = &captures[5];

    event.data.push(votes_for.replace(",", ""));
    event.data.push(votes_against.replace(",", ""));
}

fn rsfail_ext(event: &mut ParsedEvent, captures: Captures<'_>, _: &[&str]) {
    let votes_against = &captures[3];
    let votes_for = &captures[4];

    event.data.push(votes_against.replace(",", ""));
    event.data.push(votes_for.replace(",", ""));
}

fn parse_authority(authority: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"</i>([A-Z])([a-z])").unwrap();
    }

    RE.captures_iter(authority).map(|m| {
        if &m[1] == "E" && &m[2] == "x" { "X".to_owned() }
        else { m[1].to_owned() }
    }).collect()
}

fn roadd_ext(event: &mut ParsedEvent, captures: Captures<'_>, _: &[&str]) {
    let authority = &captures[4];
    event.data.push(parse_authority(authority));
}

fn rochange_ext(event: &mut ParsedEvent, captures: Captures<'_>, _: &[&str]) {
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

fn rdelauth_ext(event: &mut ParsedEvent, captures: Captures<'_>, _: &[&str]) {
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