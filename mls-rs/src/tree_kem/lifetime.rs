// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// Copyright by contributors to this project.
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::{client::MlsError, time::MlsTime};
use core::time::Duration;
use mls_rs_codec::{MlsDecode, MlsEncode, MlsSize};

#[derive(Clone, Debug, PartialEq, Eq, MlsSize, MlsEncode, MlsDecode, Default)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Lifetime {
    pub not_before: MlsTime,
    pub not_after: MlsTime,
}

impl Lifetime {
    pub fn new(not_before: MlsTime, not_after: MlsTime) -> Lifetime {
        Lifetime {
            not_before,
            not_after,
        }
    }

    pub fn seconds(s: u64, maybe_not_before: Option<MlsTime>) -> Result<Self, MlsError> {
        #[cfg(feature = "std")]
        let not_before = MlsTime::now();
        #[cfg(not(feature = "std"))]
        // There is no clock on no_std, this is here just so that we can run tests.
        let not_before = MlsTime::from(3600u64);

        let not_before = if let Some(not_before_time) = maybe_not_before {
            not_before_time
        } else {
            not_before
        };

        let not_after = MlsTime::from(
            not_before
                .seconds_since_epoch()
                .checked_add(s)
                .ok_or(MlsError::TimeOverflow)?,
        );

        Ok(Lifetime {
            // Subtract 1 hour to address time difference between machines
            not_before: not_before - Duration::from_secs(3600),
            not_after,
        })
    }

    pub fn days(d: u32, maybe_not_before: Option<MlsTime>) -> Result<Self, MlsError> {
        Self::seconds((d * 86400) as u64, maybe_not_before)
    }

    pub fn years(y: u8, maybe_not_before: Option<MlsTime>) -> Result<Self, MlsError> {
        Self::days(365 * y as u32, maybe_not_before)
    }

    pub(crate) fn within_lifetime(&self, time: MlsTime) -> bool {
        self.not_before <= time && time <= self.not_after
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;
    use assert_matches::assert_matches;

    const HOUR: Duration = Duration::from_secs(3600);
    const DAY: Duration = Duration::from_secs(24 * 3600);

    #[test]
    fn test_lifetime_overflow() {
        let res = Lifetime::seconds(u64::MAX, None);
        assert_matches!(res, Err(MlsError::TimeOverflow))
    }

    #[test]
    fn test_seconds() {
        let seconds = 10;
        let lifetime = Lifetime::seconds(seconds, None).unwrap();
        assert_eq!(
            lifetime.not_after - lifetime.not_before,
            Duration::from_secs(3610)
        );
    }

    #[test]
    fn test_days() {
        let days = 2;
        let lifetime = Lifetime::days(days, None).unwrap();

        assert_eq!(
            lifetime.not_after - lifetime.not_before,
            days * DAY + 1 * HOUR
        );
    }

    #[test]
    fn test_years() {
        let years = 2;
        let lifetime = Lifetime::years(years, None).unwrap();

        assert_eq!(
            lifetime.not_after - lifetime.not_before,
            365 * DAY * (years as u32) + 1 * HOUR
        );
    }

    #[test]
    fn test_bounds() {
        let test_lifetime = Lifetime {
            not_before: MlsTime::from(5),
            not_after: MlsTime::from(10),
        };

        assert!(!test_lifetime
            .within_lifetime(MlsTime::from_duration_since_epoch(Duration::from_secs(4))));

        assert!(!test_lifetime
            .within_lifetime(MlsTime::from_duration_since_epoch(Duration::from_secs(11))));

        assert!(test_lifetime
            .within_lifetime(MlsTime::from_duration_since_epoch(Duration::from_secs(5))));

        assert!(test_lifetime
            .within_lifetime(MlsTime::from_duration_since_epoch(Duration::from_secs(10))));

        assert!(test_lifetime
            .within_lifetime(MlsTime::from_duration_since_epoch(Duration::from_secs(6))));
    }
}
