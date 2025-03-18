use std::{fs, io::{BufReader, Read}, path::Path};

use bevy::{asset::RenderAssetUsages, ecs::system::Resource, image::Image, render::render_resource::{Extent3d, TextureDimension, TextureFormat}};
use mvt_reader::Reader;
use raqote::{AntialiasMode, DrawOptions, DrawTarget, PathBuilder, SolidSource, Source, StrokeStyle};
use rstar::{RTree, RTreeObject, AABB};

use crate::types::{level_to_tile_width, Coord};

#[derive(Resource, Clone)]
pub struct OfmTiles {
    pub tiles: RTree<Tile>,
    pub tiles_to_render: Vec<Tile>,
}

#[derive(Clone)]
pub struct Tile {
    pub name: String,
    pub image: Image,
    pub tile_location: Coord,
    pub zoom: u32,
} 

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.zoom == other.zoom && self.tile_location.lat == other.tile_location.lat && self.tile_location.long == other.tile_location.long
    }
}
impl RTreeObject for Tile {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.tile_location.long as f64, self.tile_location.lat as f64],
            [
                self.tile_location.long as f64 + level_to_tile_width(self.zoom) as f64,
                self.tile_location.lat as f64 + level_to_tile_width(self.zoom) as f64,
            ],
        )
    }
}
impl Tile {
    pub fn new(name: String, image: Image, tile_location: Coord, zoom: u32) -> Self {
        Self {
            name,
            image,
            tile_location,
            zoom,
        }
    }
}

pub fn tile_width_meters(zoom: u32) -> f64 {
    let earth_circumference_meters = 40075016.686;
    let num_tiles = 2_u32.pow(zoom) as f64;
    earth_circumference_meters / num_tiles
}

pub fn get_ofm_image(x: u64, y: u64, zoom: u64, tile_size: u32, _url: String) -> Image {
    let data = send_vector_request(x, y, zoom, "https://tiles.openfreemap.org/planet/20250122_001001_pt".to_string());
    buffer_to_bevy_image(ofm_to_data_image(data, tile_size, zoom as u32), tile_size)
}

pub fn get_rasta_data(x: u64, y: u64, zoom: u64, url: String) -> Vec<u8> {
    send_image_tile_request(x, y, zoom, url)
}

pub fn get_mvt_data(x: u64, y: u64, zoom: u64, tile_size: u32, _url: String) -> Vec<u8> {
    let data = send_vector_request(x, y, zoom, "https://tiles.openfreemap.org/planet/20250122_001001_pt".to_string());
    ofm_to_data_image(data, tile_size, zoom as u32)
}

pub fn buffer_to_bevy_image(data: Vec<u8>, tile_size: u32) -> Image {
    Image::new(
        Extent3d {
            width: tile_size,
            height: tile_size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Rather than getting a vector trile which can be tricky to work with, we get a buffer of an image 
/// https://wiki.openstreetmap.org/wiki/Raster_tile_providers
fn send_image_tile_request(x: u64, y: u64, zoom: u64, url: String) -> Vec<u8> {
    let cache_dir = format!("cache/{}", url);
    let cache_file = format!("{}/{}_{}_{}.png", cache_dir, zoom, x, y);
    
    // Check if the file exists in the cache
    if Path::new(&cache_file).exists() {
        return png_to_image(fs::read(&cache_file).expect("Failed to read cache file"));
    }
    
    let mut req = format!("{}/{}/{}/{}.png", url, zoom, x, y);
    if url.contains("google") {
        // can change the layers y for both roads and satalite, m for just roads and s for just satalite
        req = format!("{}&x={x}&y={y}&z={zoom}", url);
    }

    // If not in cache, fetch from the network
    let mut status = 429;
    while status == 429 {
        if let Ok(mut response) = ureq::get(&req).call() {
            // info!("{}", format!("{}/{}/{}/{}.png", url, zoom, x, y));
            if response.status() == 200 {
                let mut reader: BufReader<Box<dyn Read + Send + Sync>> = BufReader::new(Box::new(response.body_mut().as_reader()));
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).expect("Failed to read bytes from response");

                // Save to cache
                fs::create_dir_all(cache_dir).expect("Failed to create cache directory");
                fs::write(&cache_file, &bytes).expect("Failed to write cache file");

                return png_to_image(bytes);
            } else if response.status() == 429 {
                std::thread::sleep(std::time::Duration::from_secs(5));
            } else {
                status = 0;
            }
        }
    }
    vec![]
}

// Helper convert png to uncompressed image
fn png_to_image(data: Vec<u8>) -> Vec<u8> {
    let img = image::load_from_memory(&data).expect("Failed to decode PNG data");
    let rgba = img.to_rgba8();
    rgba.to_vec()
}

fn send_vector_request(x: u64, y: u64, zoom: u64, url: String) -> Vec<u8> {
    let cache_dir = "cache";
    let cache_file = format!("{}/{}_{}_{}.pbf", cache_dir, zoom, x, y);

    // Check if the file exists in the cache
    if Path::new(&cache_file).exists() {
        return fs::read(&cache_file).expect("Failed to read cache file");
    }

    // If not in cache, fetch from the network
    let mut status = 429;
    while status == 429 {
        if let Ok(mut response) = ureq::get(format!("{}/{}/{}/{}.pbf", url, zoom, x, y).as_str()).call() {
            if response.status() == 200 {
                let mut reader: BufReader<Box<dyn Read + Send + Sync>> = BufReader::new(Box::new(response.body_mut().as_reader()));
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).expect("Failed to read bytes from response");

                // Save to cache
                fs::create_dir_all(cache_dir).expect("Failed to create cache directory");
                fs::write(&cache_file, &bytes).expect("Failed to write cache file");

                return bytes;
            } else if response.status() == 429 {
                std::thread::sleep(std::time::Duration::from_secs(5));
            } else {
                status = 0;
            }
        }
    }
    vec![]
}

/// This converts it to an image which is as many meters as the tile width This would be AAAMAAZZZING to multithread
/// It would also be good to add a settings struct to control the colors, perhaps add background images and select what specificlly is rendered.
// What would be good is if we slipt tile tiles into 4 when we start getting a zoom over the amount which cant go in anymore like over zoom = 16
fn ofm_to_data_image(data: Vec<u8>, size: u32, zoom: u32) -> Vec<u8> {
    let tile = Reader::new(data).unwrap();
    //let size_multiplyer = TILE_QUALITY as u32 / size ;
    let mut dt = DrawTarget::new(size as i32 , size as i32);

    if cfg!(debug_assertions) {
        let mut pb: PathBuilder = PathBuilder::new();
        pb.move_to(0.0, 0.0);
        pb.line_to(size as f32, 0.0);
        pb.line_to(size as f32, size as f32);
        pb.line_to(0.0, size as f32);
        pb.line_to(0.0, 0.0);
        let path = pb.finish();
    
        let stroke_style = StrokeStyle {
            cap: raqote::LineCap::Round,
            join: raqote::LineJoin::Round,
            width: 1.0,
            miter_limit: 10.0,
            dash_array: vec![5.0, 10.0], // 5 units of dash followed by 3 units of gap
            dash_offset: 0.0, // Start at the beginning of the dash pattern
        };
        dt.stroke(
            &path,
        &Source::Solid(SolidSource {
                r: 0xff,
                g: 0xff,
                b: 0xff,
                a: 0xff,
            }),        
            
            &stroke_style,
            &DrawOptions {
                antialias: AntialiasMode::Gray,
                ..Default::default()
            },
        );
    }
    
    let scale = (size as f32 / tile_width_meters(14.try_into().unwrap()).round() as f32) * 0.597_014_9;
    dt.set_transform(&raqote::Transform::scale(scale, scale));

    // Iterate over layers and features]
    let layer_names = tile.get_layer_names().unwrap();
    for (i, title) in layer_names.into_iter().enumerate() {
        for features in tile.get_features(i).iter() {
            for feature in features {
                let mut pb: PathBuilder = PathBuilder::new();
                match &feature.geometry {
                    geo::Geometry::Point(point) 
                        => {
                            if zoom >= 15 {
                                pb.move_to(point.x(), point.y());
                                pb.line_to(point.x() + 1.0, point.y() + 1.0);
                                pb.line_to(point.x() + 1.0, point.y());
                                pb.line_to(point.x(), point.y() + 1.0)
                            }
                        },
                    geo::Geometry::Line(line) 
                        => {
                            pb.move_to(line.start.x, line.start.y);
                            pb.line_to(line.end.x, line.end.y);
                        },
                    geo::Geometry::LineString(line_string) 
                        => {
                            for (j, line) in line_string.lines().enumerate() {
                                if j == 0 {
                                    pb.move_to(line.start.x, line.start.y);
                                    pb.line_to(line.end.x, line.end.y);
                                } else {
                                    pb.line_to(line.start.x, line.start.y);
                                    pb.line_to(line.end.x, line.end.y);
                                }
                            }
                        },
                    geo::Geometry::Polygon(polygon) 
                        => {
                                for (j, line) in polygon.exterior().0.iter().enumerate() {
                                    if j == 0 {
                                        pb.move_to(line.x, line.y);
                                        pb.line_to(line.x, line.y);
                                    } else {
                                        pb.line_to(line.x, line.y);
                                        pb.line_to(line.x, line.y);
                                    }
                                }
                        },
                    geo::Geometry::MultiPolygon(multi_polygon)
                        => {
                                for polygon in multi_polygon {
                                    for (j, line) in polygon.exterior().0.iter().enumerate() {
                                        if j == 0 {
                                            pb.move_to(line.x, line.y);
                                            pb.line_to(line.x, line.y);
                                        } else {
                                            pb.line_to(line.x, line.y);
                                            pb.line_to(line.x, line.y);
                                        }
                                    }
                                }
                        },
                    geo::Geometry::MultiPoint(multi_point) 
                        => {
                            if zoom >= 15 {
                                for point in multi_point {
                                    pb.move_to(point.x(), point.y());
                                    pb.line_to(point.x() + 1.0, point.y() + 1.0);
                                    pb.line_to(point.x() + 1.0, point.y());
                                    pb.line_to(point.x(), point.y() + 1.0)
                                }
                            }
                        },
                    geo::Geometry::MultiLineString(multi_line_string) 
                        => {
                            for line_string in multi_line_string {
                                for (j, line) in line_string.lines().enumerate() {
                                    if j == 0 {
                                        pb.move_to(line.start.x, line.start.y);
                                        pb.line_to(line.end.x, line.end.y);
                                    } else {
                                        pb.line_to(line.start.x, line.start.y);
                                        pb.line_to(line.end.x, line.end.y);
                                    }
                                }
                            }
                        },
                    geo::Geometry::GeometryCollection(geometry_collection) => {
                        println!("GeometryCollection: {:?}", geometry_collection);
                    },
                    geo::Geometry::Rect(rect) => {
                        println!("Rect: {:?}", rect);
                    },
                    geo::Geometry::Triangle(triangle) => {
                        println!("Triangle: {:?}", triangle);
                    },
                }

                if title == "building" {
                    let path = pb.finish();
                    dt.fill(
                        &path,
                    &Source::Solid(SolidSource {
                            r: 0xff,
                            g: 0xff,
                            b: 0xff,
                            a: 0xff,
                        }),        
                        
                        &DrawOptions {
                            antialias: AntialiasMode::Gray,
                            blend_mode: raqote::BlendMode::SrcOver,
                            alpha: 0.5,
                        },
                    );
                } else if title == "park" {
                    let path = pb.finish();
                    dt.fill(
                        &path,
                    &Source::Solid(SolidSource {
                            r: 0x00,
                            g: 0xff,
                            b: 0x00,
                            a: 0xff,
                        }),        
                        
                        &DrawOptions {
                            antialias: AntialiasMode::Gray,
                            blend_mode: raqote::BlendMode::SrcOver,
                            alpha: 0.5,
                        },
                    );
                } else if title == ("water") {
                    let path = pb.finish();
                    dt.fill(
                        &path,
                        // For some reason red and blue are swapped
                    &Source::Solid(SolidSource {
                            b: 0x00,
                            g: 0x00,
                            r: 0xff,
                            a: 0xff,
                        }),        
                        
                        &DrawOptions {
                            antialias: AntialiasMode::Gray,
                            blend_mode: raqote::BlendMode::SrcOver,
                            alpha: 0.5,
                        },
                    );
                } else if title.contains("water") || title.contains("mountian") || title.contains("land") {

                }
                else {
                    let path = pb.finish();

                    let stroke_style = StrokeStyle {
                        cap: raqote::LineCap::Round,
                        join: raqote::LineJoin::Round,
                        width: 10.,
                        miter_limit: 10.0,
                        dash_array: vec![],
                        dash_offset: 0.0,
                    };
                
                    dt.stroke(
                        &path,
                    &Source::Solid(SolidSource {
                            r: 0xff,
                            g: 0xff,
                            b: 0xff,
                            a: 0xff,
                        }),        
                        
                        &stroke_style,
                        &DrawOptions {
                            antialias: AntialiasMode::Gray,
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    dt.get_data_u8().to_vec()
}