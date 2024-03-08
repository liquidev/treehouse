use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn on(month: u32, day: u32) -> Option<Season> {
        let md = (month, day);
        Some(match () {
            _ if ((1, 1)..=(3, 20)).contains(&md) => Season::Winter,
            _ if ((3, 21)..=(6, 21)).contains(&md) => Season::Spring,
            _ if ((6, 22)..=(9, 22)).contains(&md) => Season::Summer,
            _ if ((9, 23)..=(12, 21)).contains(&md) => Season::Autumn,
            _ if ((12, 22)..=(12, 31)).contains(&md) => Season::Winter,
            // Just in case something really darn weird happens to the calendar.
            _ => return None,
        })
    }

    pub fn current() -> Option<Season> {
        let now = Utc::now();
        Self::on(now.month(), now.day())
    }
}

#[cfg(test)]
mod tests {
    use crate::fun::seasons::Season;

    #[test]
    fn all_the_seasons() {
        assert_eq!(Season::on(0, 0), None);
        assert_eq!(Season::on(1, 1), Some(Season::Winter));
        assert_eq!(Season::on(1, 15), Some(Season::Winter));
        assert_eq!(Season::on(1, 31), Some(Season::Winter));
        assert_eq!(Season::on(2, 1), Some(Season::Winter));
        assert_eq!(Season::on(2, 28), Some(Season::Winter));
        assert_eq!(Season::on(2, 29), Some(Season::Winter));
        assert_eq!(Season::on(3, 1), Some(Season::Winter));
        assert_eq!(Season::on(3, 20), Some(Season::Winter));
        assert_eq!(Season::on(3, 21), Some(Season::Spring));
        assert_eq!(Season::on(3, 22), Some(Season::Spring));
        assert_eq!(Season::on(4, 1), Some(Season::Spring));
        assert_eq!(Season::on(4, 30), Some(Season::Spring));
        assert_eq!(Season::on(5, 1), Some(Season::Spring));
        assert_eq!(Season::on(5, 31), Some(Season::Spring));
        assert_eq!(Season::on(6, 1), Some(Season::Spring));
        assert_eq!(Season::on(6, 21), Some(Season::Spring));
        assert_eq!(Season::on(6, 22), Some(Season::Summer));
        assert_eq!(Season::on(6, 30), Some(Season::Summer));
        assert_eq!(Season::on(7, 1), Some(Season::Summer));
        assert_eq!(Season::on(7, 31), Some(Season::Summer));
        assert_eq!(Season::on(8, 1), Some(Season::Summer));
        assert_eq!(Season::on(8, 31), Some(Season::Summer));
        assert_eq!(Season::on(9, 1), Some(Season::Summer));
        assert_eq!(Season::on(9, 22), Some(Season::Summer));
        assert_eq!(Season::on(9, 23), Some(Season::Autumn));
        assert_eq!(Season::on(9, 30), Some(Season::Autumn));
        assert_eq!(Season::on(10, 1), Some(Season::Autumn));
        assert_eq!(Season::on(10, 31), Some(Season::Autumn));
        assert_eq!(Season::on(11, 1), Some(Season::Autumn));
        assert_eq!(Season::on(11, 30), Some(Season::Autumn));
        assert_eq!(Season::on(12, 1), Some(Season::Autumn));
        assert_eq!(Season::on(12, 21), Some(Season::Autumn));
        assert_eq!(Season::on(12, 22), Some(Season::Winter));
        assert_eq!(Season::on(12, 22), Some(Season::Winter));
        assert_eq!(Season::on(12, 31), Some(Season::Winter));
        assert_eq!(Season::on(12, 32), None);
        assert_eq!(Season::on(21, 37), None);
    }
}
