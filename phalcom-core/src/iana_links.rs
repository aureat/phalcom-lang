//! Generated from the IANA tzdb 2026a `backward` link file.
//!
//! The backend preserves the spelling supplied by the caller in
//! `TimeZone::iana_name()`, so the adapter owns this logical-name map. Keep
//! this table generated from the pinned tzdb source; it is not a rule-table
//! comparison or a backend-pointer identity substitute.

pub(crate) fn canonical_name(identifier: &str) -> Option<&'static str> {
    Some(match identifier {
        "US/Eastern" => "America/New_York",
        "US/Central" => "America/Chicago",
        "US/Mountain" => "America/Denver",
        "US/Pacific" => "America/Los_Angeles",
        "US/Arizona" => "America/Phoenix",
        "US/Alaska" => "America/Anchorage",
        "US/Hawaii" => "Pacific/Honolulu",
        "US/Aleutian" => "America/Adak",
        "Canada/Eastern" => "America/Toronto",
        "Canada/Central" => "America/Winnipeg",
        "Canada/Mountain" => "America/Edmonton",
        "Canada/Pacific" => "America/Vancouver",
        "Canada/Atlantic" => "America/Halifax",
        "Canada/Newfoundland" => "America/St_Johns",
        "Europe/Kiev" => "Europe/Kyiv",
        "Asia/Calcutta" => "Asia/Kolkata",
        "Asia/Ashkhabad" => "Asia/Ashgabat",
        "Asia/Dacca" => "Asia/Dhaka",
        "Asia/Rangoon" => "Asia/Yangon",
        "Asia/Saigon" => "Asia/Ho_Chi_Minh",
        "Asia/Tel_Aviv" => "Asia/Jerusalem",
        "Etc/UCT" | "Etc/Universal" | "Etc/Zulu" | "UCT" | "Universal" | "Zulu" => "Etc/UTC",
        "GMT" | "GMT+0" | "GMT-0" | "GMT0" | "Greenwich" => "Etc/GMT",
        _ => return None,
    })
}
