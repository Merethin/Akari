# NS Happenings Reference

This is a list of unique NS happening lines and the regexes to match them (in Rust syntax), which tries its best to be exhaustive (please let me know if you encounter a happening line that isn't referenced/matched here!).

Since the presence of ending periods is inconsistent between the Happenings API and SSE events (and within SSE as well - some events have them, others do not!), these regex expressions assume that any ending periods have been stripped out.

Each happening has a descriptive label (in bold), followed by an identifier/category name of up to 8 characters. You are free to use whichever identifying system you want in your own project - these identifiers are just the ones Akari uses, and which client programs should expect.

Additionally, these expressions have instructions on how to extract certain bits of commonly used data from the regex. Below is a "parsed event structure" with certain fields, and after each regex pattern there is a short list of what fields to fill with which captured data.

SSE provides additional data on the context in which an event happens, notably with the `region:` bucket(s) in the "buckets" array. This lets you know which region an event occurred in even if the region is not explicitly mentioned in the happening line. Therefore, a few events mention "from `region:` bucket" so that you can get that data from there if it is present, or apply some fallback if not (Akari uses the special "[unknown]" region name, which is not a valid NS region name and can be filtered, in this case).

# Event structure

```
pub struct Event {
   event: u64, # The ID of the event. Has to be unique.
   time: u64, # The UNIX integer timestamp of the event. Formatted as "TIMESTAMP" in Happenings.
   actor: Option<String>, # The nation triggering this event, i.e. performing an action.
   receptor: Option<String>, # If there is one, the nation receiving the event (ex. being endorsed).
   origin: Option<String>, # The region where this event was originated.
   destination: Option<String>, # If this event includes several regions, the region which receives the event (receiving an embassy request, nation moving to that region).
   category: String, # A specific event category processed from the happening line.
   data: Vec<String>, # Values specific to the event, depending on the category (RMB post id, dispatch name, issue result)
}
```

# A Note on Skipped Happenings

Several happening categories are listed as "skipped". This means that they are duplicates of an existing happening that is generated at the same time, but provide less information.

An example would be "%%region%% annexed %%other_region%%" also generating the happening "Annexed %%other_region%%" on the annexer's happenings feed. The second would be skipped, as the first already lets us know said event has happened and is more complete.

Skipped happenings are not parsed, but they do get labeled with a category = "**skipped**", with the happening line verbatim in the data array, and get broadcasted nonetheless. Most applications will just want to ignore that category, as with the "unknown" category, but it is not actually _omitted_ for the sake of completeness.

# A Note on Unmatched Happenings

A happening that does not match any of the above regex patterns will be sent as an event with the category "**unknown**", and the happening line verbatim in the data array. The happening ID and timestamp are preserved as well. Do report any events marked as "unknown" to me so I can add them to this list!

# Nations

## bucket: law

**Issue enacted (law)**

`^Following new legislation in @@([0-9a-z_-]+)@@, (.+)$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (issue result)

## bucket: change

**Nation changes classification (chclass)**

`^@@([0-9a-z_-]+)@@ was reclassified from "([A-Za-z -]+)" to "([A-Za-z -]+)"$`
- receptor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (original classification), third group (new classification)

**Nation achieves a certain census rank (chcensus)**

`^@@([0-9a-z_-]+)@@ was ranked in((?:,? (?:and )?the Top (?:1|5|10)% (?:of the world )?for(?:(?:,? (?:and )?(?:(?:[A-Z][A-Za-z-]+ ?)+))*))+)$`
- receptor: first group
- origin: from `region:` bucket or [unknown]
- data: parsed from second group (below)

Subexpressions:
- `the Top (1|5|10)% (?:of the world )?for((?:,? (?:and )?(?:(?:[A-Z][A-Za-z-]+ ?)+))*)` to parse each rank percentage (first group: rank percentage, second group: rank names, parsed below)
- `((?:[A-Z][A-Za-z-]+ ?)+)` to parse each rank name (first group: name, strip trailing whitespace)

**Nation updates its custom fields (chfield)**

`^@@([0-9a-z_-]+)@@ changed its national ([a-z ]+) to "([^"]*)"((?:,? (?:and )?its [a-z ]+ to "[^"]*")+)?$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (field type), third group (field content), parsed from fourth group (extra field types + field content)

Subexpressions:
- `,? (?:and )?its ([a-z ]+) to "([^"]+)"` to parse each custom field name and value (first group: name, second group: value)

**Nation updates its flag (chflag)**

`^@@([0-9a-z_-]+)@@ altered its national flag$`
- actor: first group
- origin: from `region:` bucket or [unknown]

**Nation uploads a new national banner (nbanner)**

`^@@([0-9a-z_-]+)@@ created a custom banner$`
- actor: first group
- origin: from `region:` bucket or [unknown]

**Nation updates a national banner (chbanner)**

`^@@[0-9a-z_-]+)@@ changed a custom banner$`
- actor: first group
- origin: from `region:` bucket or [unknown]

**Nation's influence in a region changes (chinf)**

`^@@([0-9a-z_-]+)@@'s influence in %%([0-9a-z_-]+)%% (rose|fell) from "([A-Za-z -]+)" to "([A-Za-z -]+)"$`
- receptor: first group
- origin: second group
- data: third group (direction), fourth group (old influence level), fifth group (new influence level)

**Nation deletes a custom field (rvfield)**

`^@@([0-9a-z_-]+)@@ revoked its national (faith|leader|capital)$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (field type)

## bucket: dispatch

**Nation publishes a new dispatch (dispatch)**

`^@@([0-9a-z_-]+)@@ published "<a href="page=dispatch/id=([0-9]+)">([^><]+)</a>" \(([A-Za-z]+): ([A-Za-z]+)\)$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (dispatch id), third group (dispatch name), fourth group (dispatch category), fifth group (dispatch subcategory)

# Regions

## bucket: rmb

**Nation posts on the regional RMB (rmbpost)**

`^@@([0-9a-z_-]+)@@ lodged <a href="/region=(?:[0-9a-z_-]+)/page=display_region_rmb\?postid=(?:[0-9]+)#p([0-9]+)">a message</a> on the %%([0-9a-z_-]+)%% Regional Message Board$`
- actor: first group
- origin: third group
- data: second group (post id)

**Nation suppresses a post (rmbnsupp)**

`^@@([0-9a-z_-]+)@@ suppressed a post on the %%([0-9a-z_-]+)%% Regional Message Board$`
- actor: first group
- origin: second group

**Nation unsupresses a post (rmbrsupp)**

`^@@([0-9a-z_-]+)@@ unsuppressed a post on the %%([0-9a-z_-]+)%% Regional Message Board$`
- actor: first group
- origin: second group

## bucket: embassy

**Nation sends an embassy request to another region (ereq)**

`^@@([0-9a-z_-]+)@@ proposed constructing embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group
- destination: third group

**Nation accepts embassy request from another region (eaccept)**

`^@@([0-9a-z_-]+)@@ agreed to construct embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group
- destination: third group

**Nation cancels embassy closure with another region (ecancel)**

`^@@([0-9a-z_-]+)@@ cancelled the closure of embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group
- destination: third group

**Nation indicates that an embassy closure is not desired (ewish)**

`^@@([0-9a-z_-]+)@@ indicated that %%([0-9a-z_-]+)%% did not wish to close its embassy with %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group
- destination: third group

**Nation rejects an embassy request from another region (ereject)**

`^@@([0-9a-z_-]+)@@ rejected a request from %%([0-9a-z_-]+)%% for an embassy with %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: third group
- destination: second group

**Nation orders closure of embassies with another region (eclose)**

`^@@([0-9a-z_-]+)@@ ordered the closure of embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group
- destination: third group

**Nation withdraws a request for embassies with another region (epull)**

`^@@([0-9a-z_-]+)@@ withdrew a request for embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group
- destination: third group

**Nation aborts construction of embassies with another region (eabort)**

`^@@([0-9a-z_-]+)@@ aborted construction of embassies between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group
- destination: third group

**Embassy established between two regions (eufinish)**

`^Embassy established between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$`
- origin: first group
- destination: second group

**Embassy closed between two regions (euclose)**

`^Embassy cancelled between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$`
- origin: first group
- destination: second group

**Embassy construction between two regions aborted because one of them ceased to exist (euabort)**

`^Construction of embassies aborted between %%([0-9a-z_-]+)%% and %%([0-9a-z_-]+)%%$`
- origin: first group
- destination: second group

## bucket: eject

**Nation ejects other nation from a region (eject)**

`^@@([0-9a-z_-]+)@@ was ejected from %%([0-9a-z_-]+)%% by @@([0-9a-z_-]+)@@$`
- actor: third group
- receptor: first group
- origin: second group

**Nation ejects and bans other nation from a region (banject)**

`^@@([0-9a-z_-]+)@@ was ejected and banned from %%([0-9a-z_-]+)%% by @@([0-9a-z_-]+)@@$`
- actor: third group
- receptor: first group
- origin: second group

## bucket: admin

**Nation bans other nation from a region (ban)**

`^@@([0-9a-z_-]+)@@ banned @@([0-9a-z_-]+)@@ from %%([0-9a-z_-]+)%%$`
- actor: first group
- receptor: second group
- origin: third group

**Nation is banned from a region (rcvban)**

`^@@([0-9a-z_-]+)@@ was banned from %%([0-9a-z_-]+)%% by @@([0-9a-z_-]+)@@$`
- actor: third group
- receptor: first group
- origin: second group

**Nation removes other nation from a region's banlist (unban)**

`^@@([0-9a-z_-]+)@@ removed @@([0-9a-z_-]+)@@ from the regional ban list in %%([0-9a-z_-]+)%%$`
- actor: first group
- receptor: second group
- origin: third group

**Nation is removed from a region's banlist (rcvunban)**

`^@@([0-9a-z_-]+)@@ was removed from the regional ban list of %%([0-9a-z_-]+)%% by @@([0-9a-z_-]+)@@$`
- actor: third group
- receptor: first group
- origin: second group

**Nation sets a regional password (setpw)**

`^@@([0-9a-z_-]+)@@ password-protected %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation updates a regional password (changepw)**

`^@@([0-9a-z_-]+)@@ changed the regional password in %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation removes a regional password (rmpw)**

`^@@([0-9a-z_-]+)@@ removed regional password protection from %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Region updates (rupdate)**

`^%%([0-9a-z_-]+)%% updated$`
- origin: first group

**Region becomes featured (rfeature)**

`^%%([0-9a-z_-]+)%% became the Featured Region of the day$`
- origin: first group

**Region becomes the featured map (rmapfeat)**

`^%%([0-9a-z_-]+)%% became the Featured Map of the day with &&([0-9a-z_-]+)&&$`
- origin: first group
- data: second group (map ID)

**Nation founds region (rfound)**

`^@@([0-9a-z_-]+)@@ founded the region %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation initially sets a region's banner (srbanner)**

`^@@([0-9a-z_-]+)@@ set the regional banner of %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation updates a region's banner (crbanner)**

`^@@([0-9a-z_-]+)@@ changed the regional banner of %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation updates a region's flag (crflag)**

`^@@([0-9a-z_-]+)@@ altered the regional flag of %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation removes a region's flag (rrflag)**

`^@@([0-9a-z_-]+)@@ abolished the regional flag of %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation removes a regional poll (rmpoll)**

`^@@([0-9a-z_-]+)@@ deleted a regional poll in %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation adds a tag to a region (addtag)**

`^@@([0-9a-z_-]+)@@ added the tag "([^"]+)" to %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: third group
- data: second group (tag)

**Nation removes a tag from a region (rmtag)**

`^@@([0-9a-z_-]+)@@ removed the tag "([^"]+)" from %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: third group
- data: second group (tag)

**Governor/Delegate appoints other nation as a RO (roadd)**

`^@@([0-9a-z_-]+)@@ appointed @@([0-9a-z_-]+)@@ as (.+) with authority over (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?)*) in %%([0-9a-z_-]+)%%$`
- actor: first group
- receptor: second group
- origin: fifth group
- data: office name (third group), authority (parsed from fourth group below)

Subexpressions:
- `</i>([A-Z])` to parse each authority letter (letter: first group)

**Governor/Delegate renames a nation's RO position (rorename)**

`^@@([0-9a-z_-]+)@@ renamed the office held by @@([0-9a-z_-]+)@@ from "(.+)" to "(.+)" in %%([0-9a-z_-]+)%%$`
- actor: first group
- receptor: second group
- origin: fifth group
- data: old name (third group), new name (fourth group)

**Governor/Delegate changes a nation's RO authority (rochange)**

`^@@([0-9a-z_-]+)@@ (granted|removed) (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?)*) authority (?:and removed (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?)*) authority )?(?:from|to) @@([0-9a-z_-]+)@@ as (.+) in %%([0-9a-z_-]+)%%$`
- actor: first group
- receptor: fifth group
- origin: seventh group
- data: granted authority (parsed below from third group if second group is "granted"), removed authority (parsed below from third group if second group is "removed", or fourth group if it exists), office name (sixth group)

Subexpressions:
- `</i>([A-Z])` to parse each authority letter (letter: first group)

**Governor/Delegate changes a nation's RO authority and position title (rochname)**

`^@@([0-9a-z_-]+)@@ (granted|removed) (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?)*) authority (?:and removed (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+?)*) authority )?(?:from|to) @@([0-9a-z_-]+)@@ and renamed the office from "(.+)" to "(.+)" in %%([0-9a-z_-]+)%%$`
- actor: first group
- receptor: fifth group
- origin: eighth group
- data: granted authority (parsed below from third group if second group is "granted"), removed authority (parsed below from third group if second group is "removed", or fourth group if it exists), old office name (sixth group), new office name (seventh group)

Subexpressions:
- `</i>([A-Z])` to parse each authority letter (letter: first group)

**Governor/Delegate dismisses a RO (roremove)**

`^@@([0-9a-z_-]+)@@ dismissed @@([0-9a-z_-]+)@@ as (.+) of %%([0-9a-z_-]+)%%$`
- actor: first group
- receptor: second group
- origin: fourth group
- data: office name (third group)

**Nation voluntarily resigns from RO position (roresign)**

`^@@([0-9a-z_-]+)@@ resigned as (.+) of %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: third group
- data: second group (office name)

**Governor/Delegate names the governor's office (rgovtset)**

`^@@([0-9a-z_-]+)@@ named the Governor's office  <b>(.*)</b> in %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: third group
- data: governor name (second group)

Note: the two spaces there are intentional. NationStates puts two spaces before the "b" tag for some reason.

**Governor/Delegate changes the governor's office name (rgovtupd)**

`^@@([0-9a-z_-]+)@@ renamed the Governor's office from "(.*)" to  <b>(.*)</b> in %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: fourth group
- data: old governor name (second group), new governor name (third group)

Note: As above, the two spaces there are intentional. NationStates puts two spaces before the "b" tag for some reason.

**Governor/Delegate updates the delegate's authority (rdelauth)**

`^@@([0-9a-z_-]+)@@ (granted|removed) (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+)*) authority (?:and removed (<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+(?:,? (?:and )?<i class="[a-z0-9\-]+"></i>[a-zA-Z ]+)*) authority )?(?:from|to) the WA Delegate (?:@@([0-9a-z_-]+)@@ )?in %%([0-9a-z_-]+)%%$`
- actor: first group
- receptor: fifth group, if applicable
- origin: sixth group
- data: granted authority (parsed below from third group if second group is "granted"), removed authority (parsed below from third group if second group is "removed", or fourth group if it exists)

Subexpressions:
- `</i>([A-Z])` to parse each authority letter (letter: first group)

**Nation ascends to Governor position following an abdication or CTE (rnewgov)**

`^@@([0-9a-z_-]+)@@ succeeded @@([0-9a-z_-]+)@@ as Governor of %%([0-9a-z_-]+)%%$`
- actor: second group (person who abdicates, could possibly be inactive if CTEing but little can be done to represent this case well)
- receptor: first group
- origin: third group

**Governor increases a nation's succession priority (rsucprio)**

`^@@([0-9a-z_-]+)@@ increased @@([0-9a-z_-]+)@@'s succession priority in %%([0-9a-z_-]+)%%$`
- actor: first group
- receptor: second group
- origin: third group

**Nation creates a new welcome telegram for a region (nwelcome)**

`^@@([0-9a-z_-]+)@@ composed a new Welcome Telegram for %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation deletes a region's welcome telegram (rwelcome)**

`^@@([0-9a-z_-]+)@@ canceled the Welcome Telegram of %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group
  
**Nation updates a region's World Factbook Entry (rwfe)**

`^@@([0-9a-z_-]+)@@ updated the World Factbook entry in %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation adds the most supported map to the world factbook (amapwf)**

`^@@([0-9a-z_-]+)@@ added the most supported regional map to the world factbook$`
- actor: first group
- origin: from `region:` bucket or [unknown]

**Nation removes the most supported map from world factbook (rmapwf)**

`^@@([0-9a-z_-]+)@@ removed the most supported regional map from the world factbook$`
- actor: first group
- origin: from `region:` bucket or [unknown]

**Nation becomes WA Delegate of a region (ndel)**

`^@@([0-9a-z_-]+)@@ became WA Delegate of %%([0-9a-z_-]+)%%$`
- receptor: first group
- origin: second group

**Nation becomes WA Delegate of a region, seizing the position from someone else (rdel)**

`^@@([0-9a-z_-]+)@@ seized the position of %%([0-9a-z_-]+)%% WA Delegate from @@([0-9a-z_-]+)@@$`
- receptor: first group
- origin: second group
- data: third group (old delegate)

**Nation loses the delegacy of a region (ldel)**

`^@@([0-9a-z_-]+)@@ lost WA Delegate status in %%([0-9a-z_-]+)%%$`
- receptor: first group
- origin: second group

**Nation begins the process of converting to a Frontier (beginfn)**

`^@@([0-9a-z_-]+)@@ began the process of converting %%([0-9a-z_-]+)%% to a Frontier$`
- actor: first group
- origin: second group

**Nation cancels the process of converting to a Frontier (stopfn)**

`^@@([0-9a-z_-]+)@@ canceled the process of converting %%([0-9a-z_-]+)%% to a Frontier$`
- actor: first group
- origin: second group

**Region converts to a Frontier (finishfn)**

`^%%([0-9a-z_-]+)%% became a Frontier$`
- origin: first group

**Region converts to a Frontier (skipped)**

`^Became a Frontier$`

This happening is skipped by Akari as it is generated at the same time as the above happening which describes the same event and provides more information.

**Governor removed from office when converting to a Frontier (fngovrem)**

`^@@([0-9a-z_-]+)@@ stepped down as Governor of %%([0-9a-z_-]+)%% as it became a Frontier$`
- receptor: first group
- origin: second group

**Nation begins the process of converting to a Stronghold (beginst)**

`^@@([0-9a-z_-]+)@@ began the process of removing %%([0-9a-z_-]+)%%'s designation as a Frontier$`
- actor: first group
- origin: second group

**Nation cancels the process of converting to a Stronghold (stopst)**

`^@@([0-9a-z_-]+)@@ canceled the process of removing %%([0-9a-z_-]+)%%'s designation as a Frontier$`
- actor: first group
- origin: second group

**Region converts to a Stronghold (finishst)**

`^%%([0-9a-z_-]+)%% ceased to operate as a Frontier$`
- origin: first group

**Region converts to a Stronghold (skipped)**

`^Ceased to operate as a Frontier$`

This happening is skipped by Akari as it is generated at the same time as the above happening which describes the same event and provides more information.

**Governor appointed to office when converting to a Stronghold (stgovadd)**

`^@@([0-9a-z_-]+)@@ became Governor of %%([0-9a-z_-]+)%%$`
- receptor: first group
- origin: second group

**Region sends a request to annex another region (annexreq)**

`^@@([0-9a-z_-]+)@@ sent a demand to annex %%([0-9a-z_-]+)%%$`
- actor: first group
- destination: second group

**Region gets a request to be annexed by another region (annexrcv)**

`^%%([0-9a-z_-]+)%% received a demand from @@([0-9a-z_-]+)@@ to be annexed by %%([0-9a-z_-]+)%%$`
- actor: second group
- origin: third group
- destination: first group

**Region rejects a request to be annexed by another region (annexrej)**

`^@@([0-9a-z_-]+)@@ rejected a demand for %%([0-9a-z_-]+)%% to be annexed into %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group
- destination: third group

**Region accepts a request to be annexed by another region (annexacc)**

`^@@([0-9a-z_-]+)@@ accepted a demand to be annexed by %%([0-9a-z_-]+)%%$`
- actor: first group
- destination: second group

**Region withdraws a request for annexation (annexwth)**

`^@@([0-9a-z_-]+)@@ withdrew a demand to annex %%([0-9a-z_-]+)%%$`
- actor: first group
- destination: second group

**Region is annexed into another region (annexfna)**

`^%%([0-9a-z_-]+)%% was annexed by %%([0-9a-z_-]+)%%$`
- origin: first group
- destination: second group

**Region is annexed into another region (skipped)**

`^Annexed by %%([0-9a-z_-]+)%%$`

This happening is skipped by Akari as it is generated at the same time as the above happening which describes the same event and provides more information.

**Region annexes another region (annexfnb)**

`^%%([0-9a-z_-]+)%% annexed %%([0-9a-z_-]+)%%$`
- origin: first group
- destination: second group

**Region annexes another region (skipped)**

`^Annexed %%([0-9a-z_-]+)%%$`

This happening is skipped by Akari as it is generated at the same time as the above happening which describes the same event and provides more information.

**Nation grants posting privileges to embassy regions (addxrmb)**

`^@@([0-9a-z_-]+)@@ granted posting privileges on the %%([0-9a-z_-]+)%% Regional Message Board to ([a-zA-Z ]+) in embassy regions$`
- actor: first group
- origin: second group
- data: third group (nation group allowed to post)

**Nation revokes posting privileges from embassy regions (remxrmb)**

`^@@([0-9a-z_-]+)@@ revoked posting privileges on the %%([0-9a-z_-]+)%% Regional Message Board from ([a-zA-Z ]+) in embassy regions$`
- actor: first group
- origin: second group
- data: third group (nation group previously allowed to post)

**Warzone bans expire (wzbanexp)**

`^Regional bans expired in %%([0-9a-z_-]+)%%$`
- origin: first group

## bucket: maps

**Nation creates a new map (mcreate)**

`^@@([0-9a-z_-]+)@@ created &&([0-9a-z_-]+)&&$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (map ID)

**Nation creates a new map version (mvcreate)**

`^@@([0-9a-z_-]+)@@ created \*\*([0-9a-z_-]+)\*\*$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (map version ID)

**Nation updates a map to a map version (mupdate)**

`^@@([0-9a-z_-]+)@@ updated &&([0-9a-z_-]+)&& to \*\*([0-9a-z_-]+)\*\*$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (map ID), third group (map version ID)

**Nation endorses a map (mnendo)**

`^@@([0-9a-z_-]+)@@ endorsed &&([0-9a-z_-]+)&&$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (map ID)

**Nation endorses a new map, removing its previous endorsement (mrendo)**

`^@@([0-9a-z_-]+)@@ endorsed &&([0-9a-z_-]+)&& instead of &&([0-9a-z_-]+)&&$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (new map ID), third group (old map ID)

**Map loses the endorsement of a nation (mlendo)**

`^&&([0-9a-z_-]+)&& lost the endorsement of @@([0-9a-z_-]+)@@$`
- actor: second group
- origin: from `region:` bucket or [unknown]
- data: first group (map ID)

**Nation removes its endorsement from a map (munendo)**

`^@@([0-9a-z_-]+)@@ removed its endorsement from &&([0-9a-z_-]+)&&$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (map ID)

# Movement

## bucket: move

**Nation relocates to a new region (move)**

`^@@([0-9a-z_-]+)@@ relocated from %%([0-9a-z_-]+)%% to %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group
- destination: third group

## bucket: founding

**New nation is founded (nfound)**

`^@@([0-9a-z_-]+)@@ was founded in %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

**Nation is refounded (nrefound)**

`^@@([0-9a-z_-]+)@@ was refounded in %%([0-9a-z_-]+)%%$`
- actor: first group
- origin: second group

## bucket: cte

**Nation ceases to exist (ncte)**

`^@@([0-9a-z_-]+)@@ ceased to exist in %%([0-9a-z_-]+)%%$`
- receptor: first group
- origin: second group

**Governor ceases to exist (rgcte)**

`^Governor @@([0-9a-z_-]+)@@ ceased to exist$`
- receptor: first group
- origin: from `region:` bucket or [unknown]

**Founder ceases to exist (rfcte)**

`^Regional Founder @@([0-9a-z_-]+)@@ ceased to exist$`
- receptor: first group
- origin: from `region:` bucket or [unknown]

# World Assembly

## bucket: vote

**Nation votes on a WA resolution (wavote)**

`^@@([0-9a-z_-]+)@@ voted (for|against) the World Assembly Resolution "(.+)"$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (vote), third group (resolution name)

**Nation removes its vote from a WA resolution (wrvote)**

`^@@([0-9a-z_-]+)@@ withdrew its vote on the World Assembly Resolution "(.+)"$`
- actor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (resolution name)

## bucket: resolution

**WA proposal enters the voting floor (rsfloor)**

`^The (General Assembly|Security Council) proposal "(.+)" \(by @@([0-9a-z_-]+)@@((?:,?( and)? @@([0-9a-z_-]+)@@)*)?\) entered the resolution voting floor$`
- receptor: third group (author)
- data: chamber (first group), proposal name (second group), coauthors (parsed from fourth group below)

Subexpressions:
- `@@([0-9a-z_-]+)@@` to parse coauthors (name: first group)

**WA resolution is passed (rspass)**

`^The (General Assembly|Security Council) resolution <strong><a href="/page=WA_past_resolution/id=([0-9]+)/council=(?:1|2)">(.+)</a></strong> was passed ([0-9,]+) votes to ([0-9,]+)(?: and implemented in all WA member nations)?$`
- data: chamber (first group), resolution id (second group), proposal name (third group), votes for (fourth group), votes against (fifth group)

**Nation's WA resolution is passed (nrspass)**

`^@@([0-9a-z_-]+)@@'s resolution <a href="/page=WA_past_resolution/id=([0-9]+)/council=(?:1|2)">(.+)</a> was passed by the (General Assembly|Security Council)$`
- receptor: first group
- data: chamber (fourth group), resolution id (second group), proposal name (third group)

**WA resolution fails at vote (rsfail)**

`^The (General Assembly|Security Council) resolution "<strong>(.+)</strong>" was defeated ([0-9,]+) votes to ([0-9,]+)$`
- data: chamber (first group), proposal name (second group), votes for (third group), votes against (fourth group)

**WA resolution is discarded at vote (rdiscard)**

`^The (General Assembly|Security Council) resolution "<strong>(.+)</strong>" was discarded by the WA for rule violations after garnering ([0-9,]+) votes in favor and ([0-9,]+) votes against$`
- data: chamber (first group), proposal name (second group), votes for (third group), votes against (fourth group)

**Delegate approves WA proposal (rsapp)**

`^@@([0-9a-z_-]+)@@ approved the World Assembly proposal "(.+)"$`
- actor: first group
- data: proposal name (second group)

**Delegate withdraws their approval of a proposal (rsremapp)**

`^@@([0-9a-z_-]+)@@ withdrew its approval for the World Assembly proposal "(.+)"$`
- actor: first group
- data: proposal name (second group)

**WA member submits a proposal (rssubmit)**

`^@@([0-9a-z_-]+)@@ submitted a proposal to the (General Assembly|Security Council) (.+) Board entitled "(.+)"$`
- actor: first group
- data: chamber (second group), board (third group), proposal name (fourth group)

**Author withdraws a submitted proposal (rsremsub)**

`^@@([0-9a-z_-]+)@@ withdrew a proposal from the WA (General Assembly|Security Council) titled "(.+)"$`
- actor: first group
- data: chamber (second group), proposal name (third group)

**Proposal fails to achieve quorum (rsquorum)**

`^The (General Assembly|Security Council) proposal "(.+)" \[@@([0-9a-z_-]+)@@\] failed to achieve quorum$`
- receptor: third group
- data: chamber (first group), proposal name (second group)

## bucket: member

**Nation joins the WA (wadmit)**

`^@@([0-9a-z_-]+)@@ was admitted to the World Assembly$`
- actor: first group
- origin: from `region:` bucket or [unknown]

**Nation applies to join the WA (wapply)**

`^@@([0-9a-z_-]+)@@ applied to join the World Assembly$`
- actor: first group
- origin: from `region:` bucket or [unknown]

**Nation resigns from the WA (wresign)**

`^@@([0-9a-z_-]+)@@ resigned from the World Assembly$`
- actor: first group
- origin: from `region:` bucket or [unknown]

**Nation is ejected from the WA for rule violations (wkick)**

`^@@([0-9a-z_-]+)@@ was ejected from the (?:WA for rule violations|World Assembly)$`
- receptor: first group
- origin: from `region:` bucket or [unknown]

#### Duplicated from bucket: admin (also show up in bucket: member)

**Nation becomes WA Delegate of a region (ndel)**

`^@@([0-9a-z_-]+)@@ became WA Delegate of %%([0-9a-z_-]+)%%$`
- receptor: first group
- origin: second group

**Nation becomes WA Delegate of a region, seizing the position from someone else (rdel)**

`^@@([0-9a-z_-]+)@@ seized the position of %%([0-9a-z_-]+)%% WA Delegate from @@([0-9a-z_-]+)@@$`
- receptor: first group
- origin: second group
- data: third group (old delegate)

**Nation loses the delegacy of a region (ldel)**

`^@@([0-9a-z_-]+)@@ lost WA Delegate status in %%([0-9a-z_-]+)%%$`
- receptor: first group
- origin: second group

## bucket: endo

**Nation endorses a regionmate (wendo)**

`^@@([0-9a-z_-]+)@@ endorsed @@([0-9a-z_-]+)@@$`
- actor: first group
- receptor: second group
- origin: from `region:` bucket or [unknown]

**Nation removes its endorsement from a regionmate (wunendo)**

`^@@([0-9a-z_-]+)@@ withdrew its endorsement from @@([0-9a-z_-]+)@@$`
- actor: first group
- receptor: second group
- origin: from `region:` bucket or [unknown]

# Generic

## no bucket (bucket: all)

These are weird. Don't show up in region feeds, but show up in nation feeds (Only when All is selected, don't match any other category). The Happenings API replicates the same behavior. SSE: not tested.

**Governor of a region abdicates (govabd)**

`^Governor @@([0-9a-z_-]+)@@ abdicated$`
- actor: first group
- origin: from `region:` bucket?

**Nation creates a new poll in a region (npoll)**

`^@@([0-9a-z_-]+)@@ created a new poll in %%([0-9a-z_-]+)%%: "(.+)"$`
- actor: first group
- origin: second group
- data: third group (poll title)

**Nation is kicked from a region by moderation (modkick)**

`^@@([0-9a-z_-]+)@@ was removed from %%([0-9a-z_-]+)%% by moderation$`
- receptor: first group
- origin: second group

**Nation is nominated in a Security Council proposal (nscnom)**

`^@@([0-9a-z_-]+)@@ was nominated for a World Assembly (Commendation|Condemnation) by @@([0-9a-z_-]+)@@$`
- actor: third group
- receptor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (proposal type)

**Region is nominated in a Security Council proposal (rscnom)**

`^%%([0-9a-z_-]+)%% was nominated for a World Assembly (Commendation|Condemnation) by @@([0-9a-z_-]+)@@$`
- actor: third group
- origin: first group
- data: second group (proposal type)

**Region is targeted in a Security Council proposal (rsctg)**

`^%%([0-9a-z_-]+)%% was targeted for (Liberation|Injunction) in a World Assembly proposal by @@([0-9a-z_-]+)@@$`
- actor: third group
- origin: first group
- data: second group (proposal type)

**A SC proposal nominating a nation passes (nscpass)**

`^@@([0-9a-z_-]+)@@ was (commended|condemned) by <a href="/page=WA_past_resolution/id=(?:[0-9]+)/council=2">Security Council Resolution # ([0-9]+)</a>$`
- receptor: first group
- origin: from `region:` bucket or [unknown]
- data: second group (resolution type), third group (resolution id)

**A SC proposal nominating a region passes (rscpass)**

`^%%([0-9a-z_-]+)%% was (commended|condemned|liberated|injuncted) by <a href="/page=WA_past_resolution/id=(?:[0-9]+)/council=2">Security Council Resolution # ([0-9]+)</a>$`
- origin: first group
- data: second group (resolution type), third group (resolution id)

**A SC proposal nominating a nation or region passes (skipped)**

`^(Commended|Condemned|Liberated|Injuncted) by <a href="/page=WA_past_resolution/id=(?:[0-9]+)/council=2">Security Council Resolution # (?:[0-9]+)</a>$`

This happening is skipped by Akari as it is generated at the same time as the two happenings above which describe the same event and provide more information.

# System events

These are not emitted by NationStates but by Akari itself.

This section has no use if you're simply looking at this document to use the regex patterns for your own project, but will be useful if you're building something that depends on Akari.

These events always have an event ID of -1.

**Connection to NationStates established / reestablished (conninit)**

**Connection to NationStates lost (conndrop)**

- data: first group (last event id received before disconnection)

**Missed events from NationStates (connmiss)**

- data: first group (number of events missed), second group (last event ID received before the missed events), third group (first event ID received after the missed events)

Typically emitted just after a `conninit` event when the connection has been successfully reestablished after being lost for a period of time.

## Utility

In most cases, when a `conndrop` event occurs, the connection will only be down for a minute or so - a sporadic SSE failure will lead to Akari dropping the connection, attempting to reconnect after 60 seconds and successfully doing so.

However, in some cases (if the connection limit is reached, or the SSE server / API is down for an extended period of time, or even NS itself), the disconnection period may persist for longer. In those cases, applications may want to switch to an alternative method of fetching events when the `conndrop` event is received (for example, a recruiting program using Akari for nation founds temporarily switching to the `newnationdetails` API) until the connection is resumed (which will send a `conninit` event).

If an application wants to process every single happening of a given kind, it may find it useful to catch the `connmiss` event in order to fetch the missing events fron the happenings API directly (In the future, Akari may optionally do this itself for certain output sources).