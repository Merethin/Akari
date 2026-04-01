use html_escape::decode_html_entities;

// Windows-1252 (Latin-1) to Unicode translation table for the 0x80-0x9f range
// https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/WindowsBestFit/bestfit1252.txt
const TRANSLATION_TABLE: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}',
    '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}',
    '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}',
    '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}',
    '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

// Translate a string in "NS encoding" to proper Unicode.
// NS gives us UTF-8 text, but the decoded characters follow the Windows-1252 encoding, 
// and any characters outside of that are encoded as HTML entities (&#1834;).
pub fn translate_to_unicode(data: &mut String) {
    let mut result = String::with_capacity(data.len());

    for c in data.chars() {
        if (0x80..=0x9f).contains(&(c as u32)) {
            result.push(TRANSLATION_TABLE[(c as usize) - 0x80]);
        } else {
            result.push(c);
        }
    }

    *data = decode_html_entities(&result).into_owned();
}