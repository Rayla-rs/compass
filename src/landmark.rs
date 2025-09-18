use geoconv::{bearing, haversine_distance, Degrees, Lle, Meters, Wgs84};

pub struct Landmark {
    pub name: &'static str,
    pub lle: Lle<Wgs84, Degrees>,
}

impl Landmark {
    pub fn bearing_from(&self, other: (Degrees, Degrees)) -> Degrees {
        bearing(other, (self.lle.latitude, self.lle.longitude))
    }
    pub fn distance_from(&self, other: (Degrees, Degrees)) -> Meters {
        haversine_distance(other, (self.lle.latitude, self.lle.longitude))
    }
}
