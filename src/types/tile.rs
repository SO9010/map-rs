use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
use std::{
    f64::consts::PI,
    ops::{AddAssign, DivAssign, MulAssign, SubAssign},
};


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldSpaceRect {
    pub top_left: Coord,
    pub bottom_right: Coord,
}

pub enum DistanceType {
    Km,
    M,
    CM
}

impl std::fmt::Debug for DistanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistanceType::Km => write!(f, "Km"),
            DistanceType::M => write!(f, "M"),
            DistanceType::CM => write!(f, "CM"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Copy,)]
#[serde(rename_all = "camelCase")]
pub struct Coord {
    pub lat: f32,
    #[serde(rename = "lon")]
    pub long: f32,
}

impl Coord {
    pub const fn new(lat: f32, long: f32) -> Self {
        Self { lat, long }
    }

    pub fn to_tuple(&self) -> (f32, f32) {
        (self.lat, self.long)
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.lat, self.long)
    }

    pub fn to_tile_coords(&self, zoom: u32) -> Tile {
        let x = ((self.long + 180.0) / 360.0 * (2_i32.pow(zoom) as f32)).floor() as i32;
        let y = ((1.0
            - (self.lat.to_radians().tan() + 1.0 / self.lat.to_radians().cos()).ln()
                / std::f32::consts::PI)
            / 2.0
            * (2_i32.pow(zoom) as f32))
            .floor() as i32;
        Tile { x, y, zoom }
    }

    pub fn to_mercator(&self) -> Vec2 {
        let lon_rad = self.long.to_radians() as f64;
        let lat_rad = self.lat.to_radians() as f64;
        let x = lon_rad * 20037508.34 / std::f64::consts::PI;
        let y = lat_rad.tan().asinh() * 20037508.34 / std::f64::consts::PI;

        Vec2::new(x as f32, y as f32)
    }
    
    // https://stackoverflow.com/questions/639695/how-to-convert-latitude-or-longitude-to-meters
    pub fn distance(&self, other: &Coord) -> (f64, DistanceType) {
        let earth_radius_in_km = 6378.137;
        let lat1 = self.lat as f64 * PI / 180.0;
        let lat2 = other.lat as f64 * PI / 180.0;
        let d_lat = lat2 - lat1;
        let d_lon = (other.long - self.long) as f64 * PI / 180.0;
        
        let a = (d_lat/2.0).sin() * (d_lat/2.0).sin() + 
                lat1.cos() * lat2.cos() * 
                (d_lon/2.0).sin() * (d_lon/2.0).sin();
        let c = 2.0 * ((a).sqrt().atan2((1.0-a).sqrt()));
        let d = earth_radius_in_km * c;
        
        if d * 1000. > 999. {
            (d, DistanceType::Km)
        } else {
            (d * 1000., DistanceType::M)
        }
    }

    pub fn to_game_coords(&self, reference: Coord, zoom: u32, tile_quality: f64) -> Vec2 {
        let mut ref_coords = Vec2 { x: 1., y: 1. };
        if reference.lat != 0. && reference.long != 0. {
            ref_coords = reference.to_mercator();
        }

        let meters_per_tile = 20037508.34 * 2.0 / (2.0_f64.powi(zoom as i32)); // At zoom level N
        let scale = (meters_per_tile / tile_quality) as f32;

        let x = self.long * 20037508.34 / 180.0;
        let y = (self.lat.to_radians().tan() + 1.0 / self.lat.to_radians().cos()).ln()
            * 20037508.34
            / std::f32::consts::PI;

        let x_offset = (x - ref_coords.x) / scale;
        let y_offset = (y - ref_coords.y) / scale;

        Vec2 {
            x: x_offset,
            y: y_offset,
        }
    }
}

impl std::ops::Div for Coord {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Coord {
            lat: self.lat / rhs.lat,
            long: self.long / rhs.long,
        }
    }
}

impl std::ops::Sub for Coord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Coord {
            lat: self.lat - rhs.lat,
            long: self.long - rhs.long,
        }
    }
}

impl std::ops::Add for Coord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Coord {
            lat: self.lat + rhs.lat,
            long: self.long + rhs.long,
        }
    }
}

impl SubAssign for Coord {
    fn sub_assign(&mut self, rhs: Self) {
        self.lat -= rhs.lat;
        self.long -= rhs.long;
    }
}

impl AddAssign for Coord {
    fn add_assign(&mut self, rhs: Self) {
        self.lat += rhs.lat;
        self.long += rhs.long;
    }
}

impl MulAssign for Coord {
    fn mul_assign(&mut self, rhs: Self) {
        self.lat *= rhs.lat;
        self.long *= rhs.long;
    }
}

impl DivAssign for Coord {
    fn div_assign(&mut self, rhs: Self) {
        self.lat /= rhs.lat;
        self.long /= rhs.long;
    }
}

pub fn tile_to_coords(x: i32, y: i32, zoom: u32) -> Coord {
    let n = 2_i32.pow(zoom) as f32;
    let lon = x as f32 / n * 360.0 - 180.0;
    let lat_rad = (std::f32::consts::PI * (1.0 - 2.0 * y as f32 / n))
        .sinh()
        .atan();
    let lat = lat_rad.to_degrees();
    Coord::new(lat, lon)
}

pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub zoom: u32,
}

impl Tile {
    pub const fn new(x: i32, y: i32, zoom: u32) -> Self {
        Self { x, y, zoom }
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    pub fn to_lat_long(&self) -> Coord {
        let n = 2.0f64.powi(self.zoom as i32);
        let lon_deg = self.x as f64 / n * 360.0 - 180.0;
        let lat_deg = (PI * (1.0 - 2.0 * self.y as f64 / n))
            .sinh()
            .atan()
            .to_degrees();
        Coord::new(lat_deg as f32, normalize_longitude(lon_deg) as f32)
    }

    pub fn to_game_coords(&self, offset: Coord, zoom: u32, tile_quality: f64) -> Vec2 {
        self.to_lat_long()
            .to_game_coords(offset, zoom, tile_quality)
    }

    pub fn to_mercator(&self) -> Vec2 {
        self.to_lat_long().to_mercator()
    }
}

pub fn level_to_tile_width(level: u32) -> f32 {
    360.0 / (2_i32.pow(level) as f32)
}

pub fn world_mercator_to_lat_lon(
    x_offset: f64,
    y_offset: f64,
    reference: Coord,
    zoom: u32,
    quality: f32,
) -> Coord {
    // Convert reference point to Web Mercator
    let refrence = reference.to_mercator();

    // Calculate meters per pixel (adjust for your tile setup)
    let meters_per_tile = 20037508.34 * 2.0 / (2.0_f64.powi(zoom as i32)); // At zoom level N
    let scale = meters_per_tile / quality as f64;

    // Apply offsets with corrected scale
    let global_x = refrence.x as f64 + (x_offset * scale);
    let global_y = refrence.y as f64 + (y_offset * scale);

    // Inverse Mercator to convert back to lat/lon
    let lon = (global_x / 20037508.34) * 180.0;
    let lat = (global_y / 20037508.34 * 180.0).to_radians();
    let lat = 2.0 * lat.exp().atan() - std::f64::consts::FRAC_PI_2;
    let lat = lat.to_degrees();

    Coord::new(lat as f32, normalize_longitude(lon) as f32)
}

pub fn lat_lon_to_world_mercator_with_offset(
    lat: f64,
    lon: f64,
    reference: Coord,
    zoom: u32,
    quality: u32,
) -> (f64, f64) {
    // Convert reference point to Web Mercator

    let refrence = reference.to_mercator();

    // Calculate meters per pixel (adjust for your tile setup)
    let meters_per_tile = 20037508.34 * 2.0 / (2.0_f64.powi(zoom as i32)); // At zoom level N
    let scale = meters_per_tile / quality as f64;

    // Convert lat/lon to world mercator coordinates
    let x = lon * 20037508.34 / 180.0;
    let y = (lat.to_radians().tan() + 1.0 / lat.to_radians().cos()).ln() * 20037508.34
        / std::f64::consts::PI;

    // Apply offsets with corrected scale
    let x_offset = (x - refrence.x as f64) / scale;
    let y_offset = (y - refrence.y as f64) / scale;

    (x_offset, y_offset)
}

fn normalize_longitude(lon: f64) -> f64 {
    let mut lon = lon;
    while lon > 180.0 {
        lon -= 360.0;
    }
    while lon < -180.0 {
        lon += 360.0;
    }
    lon
}
