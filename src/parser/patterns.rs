use regex::{Regex, RegexSet, Error};

pub fn generate_patterns() -> Result<(Vec<(&'static str, Regex)>, RegexSet), Error> {
    let patterns: Vec<(&'static str, Regex)> = vec![
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
        ("rmapfeat", Regex::new(r#"^%%([0-9a-z_-]+)%% became the Featured Map of the day with &&([0-9a-z_-]+)&&$"#)?),
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
        ("rgovtset", Regex::new(r#"^@@([0-9a-z_-]+)@@ named the Governor's office  <b>(.*)</b> in %%([0-9a-z_-]+)%%$"#)?),
        ("rgovtupd", Regex::new(r#"^@@([0-9a-z_-]+)@@ renamed the Governor's office from "(.*)" to  <b>(.*)</b> in %%([0-9a-z_-]+)%%$"#)?),
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
        ("skipped", Regex::new(r#"^Became a Frontier$"#)?),
        ("fngovrem", Regex::new(r#"^@@([0-9a-z_-]+)@@ stepped down as Governor of %%([0-9a-z_-]+)%% as it became a Frontier$"#)?),
        ("beginst", Regex::new(r#"^@@([0-9a-z_-]+)@@ began the process of removing %%([0-9a-z_-]+)%%'s designation as a Frontier$"#)?),
        ("stopst", Regex::new(r#"^@@([0-9a-z_-]+)@@ canceled the process of removing %%([0-9a-z_-]+)%%'s designation as a Frontier$"#)?),
        ("finishst", Regex::new(r#"^%%([0-9a-z_-]+)%% ceased to operate as a Frontier$"#)?),
        ("skipped", Regex::new(r#"^Ceased to operate as a Frontier$"#)?),
        ("stgovadd", Regex::new(r#"^@@([0-9a-z_-]+)@@ became Governor of %%([0-9a-z_-]+)%%$"#)?),
        ("annexreq", Regex::new(r#"^@@([0-9a-z_-]+)@@ sent a demand to annex %%([0-9a-z_-]+)%%$"#)?),
        ("annexrcv", Regex::new(r#"^%%([0-9a-z_-]+)%% received a demand from @@([0-9a-z_-]+)@@ to be annexed by %%([0-9a-z_-]+)%%$"#)?),
        ("annexrej", Regex::new(r#"^@@([0-9a-z_-]+)@@ rejected a demand for %%([0-9a-z_-]+)%% to be annexed into %%([0-9a-z_-]+)%%$"#)?),
        ("annexacc", Regex::new(r#"^@@([0-9a-z_-]+)@@ accepted a demand to be annexed by %%([0-9a-z_-]+)%%$"#)?),
        ("annexwth", Regex::new(r#"^@@([0-9a-z_-]+)@@ withdrew a demand to annex %%([0-9a-z_-]+)%%$"#)?),
        ("annexfna", Regex::new(r#"^%%([0-9a-z_-]+)%% was annexed by %%([0-9a-z_-]+)%%$"#)?),
        ("skipped", Regex::new(r#"^Annexed by %%([0-9a-z_-]+)%%$"#)?),
        ("annexfnb", Regex::new(r#"^%%([0-9a-z_-]+)%% annexed %%([0-9a-z_-]+)%%$"#)?),
        ("skipped", Regex::new(r#"^Annexed %%([0-9a-z_-]+)%%$"#)?),
        ("addxrmb", Regex::new(r#"^@@([0-9a-z_-]+)@@ granted posting privileges on the %%([0-9a-z_-]+)%% Regional Message Board to ([a-zA-Z ]+) in embassy regions$"#)?),
        ("remxrmb", Regex::new(r#"^@@([0-9a-z_-]+)@@ revoked posting privileges on the %%([0-9a-z_-]+)%% Regional Message Board from ([a-zA-Z ]+) in embassy regions$"#)?),
        ("wzbanexp", Regex::new(r#"^Regional bans expired in %%([0-9a-z_-]+)%%$"#)?),
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
        ("rspass", Regex::new(r#"^The (General Assembly|Security Council) resolution <strong><a href="/page=WA_past_resolution/id=([0-9]+)/council=(?:1|2)">(.+)</a></strong> was passed ([0-9,]+) votes to ([0-9,]+)(?: and implemented in all WA member nations)?$"#)?),
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
        ("wkick", Regex::new(r#"^@@([0-9a-z_-]+)@@ was ejected from the (?:WA for rule violations|World Assembly)$"#)?),
        // bucket: endo
        ("wendo", Regex::new(r#"^@@([0-9a-z_-]+)@@ endorsed @@([0-9a-z_-]+)@@$"#)?),
        ("wunendo", Regex::new(r#"^@@([0-9a-z_-]+)@@ withdrew its endorsement from @@([0-9a-z_-]+)@@$"#)?),
        // bucket: all
        ("govabd", Regex::new(r#"^Governor @@([0-9a-z_-]+)@@ abdicated$"#)?),
        ("npoll", Regex::new(r#"^@@([0-9a-z_-]+)@@ created a new poll in %%([0-9a-z_-]+)%%: "(.+)"$"#)?),
        ("modkick", Regex::new(r#"^@@([0-9a-z_-]+)@@ was removed from %%([0-9a-z_-]+)%% by moderation$"#)?),
        ("nrspass", Regex::new(r#"^@@([0-9a-z_-]+)@@'s resolution <a href="/page=WA_past_resolution/id=([0-9]+)/council=(?:1|2)">(.+)</a> was passed by the (General Assembly|Security Council)$"#)?),
        ("nscnom", Regex::new(r#"^@@([0-9a-z_-]+)@@ was nominated for a World Assembly (Commendation|Condemnation) by @@([0-9a-z_-]+)@@$"#)?),
        ("rscnom", Regex::new(r#"^%%([0-9a-z_-]+)%% was nominated for a World Assembly (Commendation|Condemnation) by @@([0-9a-z_-]+)@@$"#)?),
        ("rsctg", Regex::new(r#"^%%([0-9a-z_-]+)%% was targeted for (Liberation|Injunction) in a World Assembly proposal by @@([0-9a-z_-]+)@@$"#)?),
        ("nscpass", Regex::new(r#"^@@([0-9a-z_-]+)@@ was (commended|condemned) by <a href="/page=WA_past_resolution/id=(?:[0-9]+)/council=2">Security Council Resolution # ([0-9]+)</a>$"#)?),
        ("rscpass", Regex::new(r#"^%%([0-9a-z_-]+)%% was (commended|condemned|liberated|injuncted) by <a href="/page=WA_past_resolution/id=(?:[0-9]+)/council=2">Security Council Resolution # ([0-9]+)</a>$"#)?),
        ("skipped", Regex::new(r#"^(Commended|Condemned|Liberated|Injuncted) by <a href="/page=WA_past_resolution/id=(?:[0-9]+)/council=2">Security Council Resolution # (?:[0-9]+)</a>$"#)?)
    ];

    let regex_set = RegexSet::new(
        patterns.iter().map(|(_, pattern)| { pattern.as_str() })
    )?;

    Ok((patterns, regex_set))
}

