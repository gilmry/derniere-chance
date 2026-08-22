/// Great-circle distance between two lat/lon points, in kilometers
/// (haversine formula). Used to show "à Xm/Xkm" and sort the feed by
/// proximity - see VISION.md §8, géolocalisation.
pub fn distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_KM * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_point_is_zero() {
        assert_eq!(distance_km(48.8566, 2.3522, 48.8566, 2.3522), 0.0);
    }

    #[test]
    fn paris_to_lyon_is_about_390km() {
        // Notre-Dame de Paris -> Place Bellecour, Lyon
        let d = distance_km(48.8530, 2.3499, 45.7578, 4.8320);
        assert!((385.0..400.0).contains(&d), "got {d}");
    }

    #[test]
    fn is_symmetric() {
        let a = distance_km(50.8503, 4.3517, 51.2194, 4.4025); // Bruxelles -> Anvers
        let b = distance_km(51.2194, 4.4025, 50.8503, 4.3517);
        assert!((a - b).abs() < 1e-9);
    }
}
