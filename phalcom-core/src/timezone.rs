//! VM-owned timezone database state.
//!
//! This module deliberately exposes only owned Phalcom-facing facts. Jiff
//! database and zone values never cross this boundary and the adapter stores no
//! guest heap handles, so it is safe to classify as non-GC state on `VM`.

use jiff::tz::{Offset, TimeZone, TimeZoneDatabase};
use std::collections::HashMap;

use crate::iana_links;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ZoneIdentity {
    Utc,
    Fixed(i32),
    Named(String),
}

#[derive(Clone, Debug)]
pub(crate) struct ZoneHandle {
    pub(crate) identity: ZoneIdentity,
    zone: TimeZone,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransitionFact {
    pub(crate) timestamp: jiff::Timestamp,
    pub(crate) offset_before: i32,
    pub(crate) offset_after: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LocalFact {
    Unique { timestamp: jiff::Timestamp },
    Fold { earlier: jiff::Timestamp, later: jiff::Timestamp },
    Gap { before: jiff::Timestamp, after: jiff::Timestamp },
}

#[derive(Debug)]
pub(crate) struct TimeZoneAdapter {
    database: Option<TimeZoneDatabase>,
    generation: u64,
    named: HashMap<String, TimeZone>,
    handles: HashMap<i64, ZoneHandle>,
}

impl TimeZoneAdapter {
    pub(crate) fn identity_hash(&self, zone: &ZoneHandle) -> i64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.generation.hash(&mut hasher);
        zone.identity.hash(&mut hasher);
        hasher.finish() as i64
    }

    pub(crate) fn register_handle(&mut self, zone: ZoneHandle) -> i64 {
        let id = self.identity_hash(&zone);
        self.handles.insert(id, zone);
        id
    }

    pub(crate) fn handle(&self, id: i64) -> Option<&ZoneHandle> {
        self.handles.get(&id)
    }

    pub(crate) fn display_name(&self, id: i64) -> Option<String> {
        let zone = self.handles.get(&id)?;
        Some(match &zone.identity {
            ZoneIdentity::Utc => "UTC".to_owned(),
            ZoneIdentity::Fixed(seconds) => {
                let sign = if *seconds < 0 { '-' } else { '+' };
                let absolute = i64::from(*seconds).abs();
                format!("{sign}{:02}:{:02}:{:02}", absolute / 3_600, (absolute % 3_600) / 60, absolute % 60)
            }
            ZoneIdentity::Named(name) => name.clone(),
        })
    }

    pub(crate) fn new() -> Self {
        Self {
            database: None,
            generation: 1,
            named: HashMap::new(),
            handles: HashMap::new(),
        }
    }

    pub(crate) fn utc(&self) -> ZoneHandle {
        ZoneHandle {
            identity: ZoneIdentity::Utc,
            zone: TimeZone::UTC,
        }
    }

    pub(crate) fn fixed(&self, offset_seconds: i32) -> ZoneHandle {
        ZoneHandle {
            identity: if offset_seconds == 0 {
                ZoneIdentity::Utc
            } else {
                ZoneIdentity::Fixed(offset_seconds)
            },
            zone: TimeZone::fixed(Offset::from_seconds(offset_seconds).expect("P1 UtcOffset bound exceeds Jiff offset range")),
        }
    }

    pub(crate) fn named(&mut self, identifier: &str) -> Result<ZoneHandle, jiff::Error> {
        let canonical = iana_links::canonical_name(identifier).unwrap_or(identifier);
        let zone = if let Some(zone) = self.named.get(canonical) {
            zone.clone()
        } else {
            let database = self.database.get_or_insert_with(TimeZoneDatabase::bundled);
            let zone = database.get(canonical)?;
            self.named.insert(canonical.to_owned(), zone.clone());
            zone
        };
        Ok(ZoneHandle {
            identity: ZoneIdentity::Named(canonical.to_owned()),
            zone,
        })
    }

    pub(crate) fn system(&mut self) -> Result<ZoneHandle, jiff::Error> {
        let system = TimeZone::try_system()?;
        if let Some(identifier) = system.iana_name() {
            return self.named(identifier);
        }
        let _ = self.database.get_or_insert_with(TimeZoneDatabase::bundled);
        let offset = system.to_offset(jiff::Timestamp::now()).seconds();
        Ok(self.fixed(offset))
    }

    pub(crate) fn offset_at(&self, zone: &ZoneHandle, timestamp: jiff::Timestamp) -> i32 {
        zone.zone.to_offset(timestamp).seconds()
    }

    pub(crate) fn local_at(&self, zone: &ZoneHandle, timestamp: jiff::Timestamp) -> jiff::civil::DateTime {
        zone.zone.to_datetime(timestamp)
    }

    pub(crate) fn resolve_local(&self, zone: &ZoneHandle, datetime: jiff::civil::DateTime) -> Result<LocalFact, jiff::Error> {
        self.local_fact(zone, datetime)
    }

    pub(crate) fn local_fact(&self, zone: &ZoneHandle, datetime: jiff::civil::DateTime) -> Result<LocalFact, jiff::Error> {
        let ambiguous = zone.zone.to_ambiguous_zoned(datetime);
        match ambiguous.offset() {
            jiff::tz::AmbiguousOffset::Unambiguous { .. } => Ok(LocalFact::Unique {
                timestamp: ambiguous.unambiguous()?.timestamp(),
            }),
            jiff::tz::AmbiguousOffset::Fold { .. } => Ok(LocalFact::Fold {
                earlier: ambiguous.clone().earlier()?.timestamp(),
                later: ambiguous.later()?.timestamp(),
            }),
            jiff::tz::AmbiguousOffset::Gap { .. } => Ok(LocalFact::Gap {
                before: ambiguous.clone().earlier()?.timestamp(),
                after: ambiguous.later()?.timestamp(),
            }),
        }
    }

    pub(crate) fn transition_before(&self, zone: &ZoneHandle, timestamp: jiff::Timestamp) -> Option<TransitionFact> {
        let transition = zone.zone.preceding(timestamp).next()?;
        let before = zone
            .zone
            .to_offset(transition.timestamp().checked_sub(jiff::SignedDuration::from_secs(1)).ok()?)
            .seconds();
        Some(TransitionFact {
            timestamp: transition.timestamp(),
            offset_before: before,
            offset_after: transition.offset().seconds(),
        })
    }

    pub(crate) fn transition_after(&self, zone: &ZoneHandle, timestamp: jiff::Timestamp) -> Option<TransitionFact> {
        let transition = zone.zone.following(timestamp).next()?;
        let before = zone
            .zone
            .to_offset(transition.timestamp().checked_sub(jiff::SignedDuration::from_secs(1)).ok()?)
            .seconds();
        Some(TransitionFact {
            timestamp: transition.timestamp(),
            offset_before: before,
            offset_after: transition.offset().seconds(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_database_is_lazy_until_named_lookup() {
        let adapter = TimeZoneAdapter::new();
        assert_eq!(adapter.generation, 1);
        assert!(adapter.named.is_empty());
        assert!(adapter.database.is_none());
        assert_eq!(adapter.utc().identity, ZoneIdentity::Utc);
        assert_eq!(adapter.fixed(0).identity, ZoneIdentity::Utc);
        assert!(adapter.named.is_empty());
        assert!(adapter.database.is_none());
    }

    #[test]
    fn named_aliases_share_canonical_identity_and_cached_rules() {
        let mut adapter = TimeZoneAdapter::new();
        let canonical = adapter.named("America/New_York").expect("canonical zone");
        let alias = adapter.named("US/Eastern").expect("linked zone");
        assert_eq!(canonical.identity, alias.identity);
        assert_eq!(adapter.named.len(), 1);
    }

    #[test]
    fn local_resolution_preserves_fold_and_gap_facts() {
        let mut adapter = TimeZoneAdapter::new();
        let zone = adapter.named("America/New_York").expect("zone");
        let fold = jiff::civil::DateTime::constant(2024, 11, 3, 1, 30, 0, 0);
        let gap = jiff::civil::DateTime::constant(2024, 3, 10, 2, 30, 0, 0);
        assert!(matches!(adapter.local_fact(&zone, fold), Ok(LocalFact::Fold { .. })));
        assert!(matches!(adapter.local_fact(&zone, gap), Ok(LocalFact::Gap { .. })));
    }

    #[test]
    fn local_resolution_preserves_non_hour_transition_facts() {
        let mut adapter = TimeZoneAdapter::new();
        let zone = adapter.named("Australia/Lord_Howe").expect("zone");
        let gap = jiff::civil::DateTime::constant(2024, 10, 6, 2, 15, 0, 0);
        let LocalFact::Gap { before, after } = adapter.local_fact(&zone, gap).expect("gap") else {
            panic!("expected Lord Howe gap")
        };
        assert_eq!(after.as_second() - before.as_second(), 1_800);
    }

    #[test]
    fn historical_transition_facts_retain_second_precision() {
        let mut adapter = TimeZoneAdapter::new();
        let zone = adapter.named("Europe/Paris").expect("zone");
        let timestamp = jiff::Timestamp::new(-1_862_659_200, 0).expect("timestamp");
        let transition = adapter.transition_after(&zone, timestamp).expect("historical transition");
        assert_ne!(transition.offset_before, transition.offset_after);
    }

    #[test]
    fn transition_facts_copy_owned_offsets() {
        let mut adapter = TimeZoneAdapter::new();
        let zone = adapter.named("America/New_York").expect("zone");
        let timestamp = jiff::Timestamp::new(1_704_067_200, 0).expect("timestamp");
        let transition = adapter.transition_after(&zone, timestamp).expect("transition");
        assert_ne!(transition.offset_before, transition.offset_after);
    }
}
