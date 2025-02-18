use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::egui::{self, Color32};

#[derive(Resource, Clone)]
pub struct SettingsOverlay {
    // String = cat name, Category = data and children
    pub categories: BTreeMap<String, Category>,
}

/// Holds the categories and sub-categories which are the basis of making an osm request.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Category {
    pub all: bool,                      // Toggle all to be on
    pub none: bool,                     // Toggle all to be off
    pub disabled: bool,                 // Make it so they are all disabled
    pub items: BTreeMap<String, (bool, egui::Color32)>,  // Maps sub-category names to their state
}

impl Category {
    pub fn set_children(&mut self, on_or_off: bool) {
        for (_, (toggle, _)) in self.items.iter_mut() {
            *toggle = on_or_off;
        } 
    }
}

impl SettingsOverlay {
    pub fn new() -> Self {
        let mut overlay = SettingsOverlay {
            categories: BTreeMap::new(),
        };

        // Example: Adding a category dynamically
        overlay.add_category(
            "Highway",
            vec![
                // Main road types
                "motorway",
                "truck",
                "primary",
                "secondary",
                "tertiary",
                "residential",
                // Link roads
                "motorway_link",
                "trunk_link",
                "primary_link",
                "secondary_link",
                "tertiary_link",
                // Special roads
                "living_street",
                "service",
                "pedestrian",
                "track",
                "bus_guideway",
                "escape",
                "raceway",
                "road",
                "busway",
                // Paths
                "footway",
                "bridleway",
                "steps",
                "corridor",
                "path",
                "via_ferrata",
                // Lifecycle
                "proposed",
                "construction",
                // Other
                "crossing",
                "cyclist_waiting_aid",
                "elevator",
                "emergency_bay",
                "emergency_access_point",
                "give_way",
                "ladder",
                "milestone",
                "mini_roundabout",
                "motorway_junction",
                "passing_place",
                "platform",
                "rest_area",
                "services",
                "speed_camera",
                "speed_display",
                "stop",
                "street_lamp",
                "toll_gantry",
                "traffic_mirror",
                "traffic_signals",
                "trailhead",
                "turning_circle",
                "turning_loop",

            ],
        );

        overlay.add_category(
            "Building",
            vec![
                // Residential buildings
                "apartments",
                "barracks",
                "bungalow",
                "cabin",
                "detached",
                "annexe",
                "dormitory",
                "farm",
                "ger",
                "hotel",
                "house",
                "houseboat",
                "residential",
                "semidetached_house",
                "static_caravan",
                "stilt_house",
                "terrace",
                "tree_house",
                "trullo",
                // Commercial buildings
                "commercial",
                "industrial",
                "kiosk",
                "office",
                "retail",
                "supermarket",
                "warehouse",
                // Religious buildings
                "religious",
                "cathedral",
                "chapel",
                "church",
                "kingdom_hall",
                "monastery",
                "mosque",
                "presbytery",
                "shrine",
                "synagogue",
                "temple",
                // Civic/Amenity buildings
                "bakehouse",
                "bridge",
                "civic",
                "college",
                "fire_station",
                "government",
                "gatehouse",
                "hospital",
                "kindergarten",
                "museum",
                "public",
                "school",
                "toilets",
                "train_station",
                "transportation",
                "university",
                // Agricultural/Plant Production buildings
                "barn",
                "conservatory",
                "cowshed",
                "farm_auxiliary",
                "greenhouse",
                "slurry_tank",
                "stable",
                "sty",
                "livestock",
                // Sports buildings
                "grandstand",
                "pavilion",
                "riding_hall",
                "sports_hall",
                "sports_centre",
                "stadium",
                // Storage buildings
                "allotment_house",
                "boathouse",
                "hangar",
                "hut",
                "shed",
                // Cars
                "carport",
                "garage",
                "garages",
                "parking",
                // Power/Technical Buildings
                "digester",
                "service",
                "tech_cab",
                "transformer_tower",
                "water_tower",
                "storage_tank",
                "silo",
                // Other Buildings
                "beach_hut",
                "bunker",
                "castle",
                "construction",
                "container",
                "guardhouse",
                "military",
                "outbuilding",

            ],
        );

        overlay.add_category(
            "Amenity",
            vec![
                "bar",
                "biergarten",
                "cafe",
                "fast_food",
                "food_court",
                "ice_cream",
                "pub",
                "restaurant",
                // Education
                "college",
                "dancing_school",
                "driving_school",
                "first_aid_school",
                "kindergarten",
                "language_school",
                "library",
                "music_school",
                "school",
                "traffic_park",
                "university",
                "research_institute",
                "training",
                "toy_library",
                "surf_school",
                // Transportation
                "bicycle_parking",
                "bicycle_repair_station",
                "bicycle_rental",
                "bicycle_wash",
                "boat_rental",
                "boat_sharing",
                "bus_station",
                "car_rental",
                "car_sharing",
                "car_wash",
                "compressed_air",
                "vehicle_inspection",
                "charging_station",
                "driver_training",
                "ferry_terminal",
                "fuel",
                "grit_bin",
                "motorcycle_parking",
                "parking",
                "parking_entrance",
                "parking_space",
                "taxi",
                "weighbridge",
                // Financial
                "atm",
                "bank",
                "bureau_de_change",
                "money_transfer",
                "payment_centre",
                "payment_terminal",
                // Healthcare
                "baby_hatch",
                "clinic",
                "dentist",
                "doctors",
                "hospital",
                "nursing_home",
                "pharmacy",
                "social_facility",
                "veterinary",
                // Entertainment, Arts & Culture
                "arts_centre",
                "brothel",
                "casino",
                "cinema",
                "community_centre",
                "conference_centre",
                "events_venue",
                "exhibition_centre",
                "fountain",
                "gambling",
                "love_hotel",
                "music_venue",
                "nightclub",
                "planetarium",
                "public_bookcase",
                "social_centre",
                "stage",
                "stripclub",
                "studio",
                "swingerclub",
                "theatre",
                // Public Service
                "courthouse",
                "fire_station",
                "police",
                "post_box",
                "post_depot",
                "post_office",
                "prison",
                "ranger_station",
                "townhall",
                // Facilities
                "bbq",
                "bench",
                "dog_toilet",
                "dressing_room",
                "drinking_water",
                "give_box",
                "lounge",
                "mailroom",
                "parcel_locker",
                "shelter",
                "shower",
                "telephone",
                "toilets",
                "water_point",
                "watering_place",
                // Waste Management
                "sanitary_dump_station",
                "recycling",
                "waste_basket",
                "waste_disposal",
                "waste_transfer_station",
                // Others
                "animal_boarding",
                "animal_breeding",
                "animal_shelter",
                "animal_training",
                "baking_oven",
                "clock",
                "crematorium",
                "dive_centre",
                "funeral_hall",
                "grave_yard",
                "hunting_stand",
                "internet_cafe",
                "kitchen",
                "kneipp_water_cure",
                "lounger",
                "marketplace",
                "monastery",
                "mortuary",
                "photo_booth",
                "place_of_mourning",
                "place_of_worship",
                "public_bath",
                "public_building",
                "refugee_site",
                "vending_machine",
                "user_defined",
            ],
        );

        overlay.add_category(
            "Landuse",
            vec![
                // Urban and Commercial Landuse
                "commercial",
                "construction",
                "education",
                "fairground",
                "industrial",
                "residential",
                "retail",
                "institutional",

                // Rural and Agricultural Landuse
                "aquaculture",
                "allotments",
                "farmland",
                "farmyard",
                "paddy",
                "animal_keeping",
                "flowerbed",
                "forest",
                "logging",
                "greenhouse_horticulture",
                "meadow",
                "orchard",
                "plant_nursery",
                "vineyard",

                // Waterbody Landuse
                "basin",
                "reservoir",
                "salt_pond",

                // Other Landuse
                "brownfield",
                "cemetery",
                "conservation",
                "depot",
                "garages",
                "grass",
                "greenfield",
                "landfill",
                "military",
                "port",
                "quarry",
                "railway",
                "recreation_ground",
                "religious",
                "village_green",
                "greenery",
                "winter_sports",

            ],
        );

        overlay.add_category(
            "Leisure",
            vec![
                // Entertainment and Gaming
                "adult_gaming_centre",
                "amusement_arcade",

                // Outdoor and Recreational
                "beach_resort",
                "bandstand",
                "bird_hide",
                "common",
                "dance",
                "disc_golf_course",
                "dog_park",
                "escape_game",
                "firepit",
                "fishing",
                "fitness_centre",
                "fitness_station",
                "garden",
                "hackerspace",
                "horse_riding",
                "ice_rink",
                "marina",
                "miniature_golf",
                "nature_reserve",
                "park",
                "picnic_table",
                "pitch",
                "playground",
                "slipway",
                "sports_centre",
                "stadium",
                "summer_camp",
                "swimming_area",
                "swimming_pool",
                "track",
                "water_park",

            ],
        );
        overlay.add_category(
            "ManMade",
            vec![
                "adit",
                "mineshaft",
                "beacon",
                "lighthouse",
                "breakwater",
                "dyke",
                "groyne",
                "pier",
                "bridge",
                "pipeline",
                "pumping_station",
                "reservoir_covered",
                "water_tower",
                "water_well",
                "water_tap",
                "water_works",
                "bunker_silo",
                "chimney",
                "crane",
                "gasometer",
                "goods_conveyor",
                "kiln",
                "silo",
                "storage_tank",
                "tailings_pond",
                "works",
                "communications_tower",
                "mast",
                "monitoring_station",
                "street_cabinet",
                "surveillance",
                "video_wall",
                "cross",
                "dovecote",
                "obelisk",
                "stupa",
                "observatory",
                "survey_point",
                "telescope",
                "clearcut",
                "cutline",
                "snow_fence",
                "snow_net",
                "wildlife_crossing",
                "carpet_hanger",
                "column",
                "embankment",
                "flagpole",
                "guard_stone",
                "offshore_platform",
                "petroleum_well",
                "pump",
                "watermill",
                "windmill",
                "yes",
            ],
        );
        overlay.add_category(
            "Military",
            vec![
                "academy",
                "obstacle_course",
                "school",
                "training_area",
                "airfield",
                "base",
                "barracks",
                "bunker",
                "office",
                "checkpoint",
                "danger_area",
                "nuclear_explosion_site",
                "range",
                "trench",
            ],
        );
        overlay.add_category(
            "Natural",
            vec![
                "fell",
                "grassland",
                "heath",
                "moor",
                "scrub",
                "shrubbery",
                "tree",
                "tree_row",
                "tundra",
                "wood",
                "bay",
                "beach",
                "blowhole",
                "cape",
                "coastline",
                "crevasse",
                "geyser",
                "glacier",
                "hot_spring",
                "isthmus",
                "mud",
                "peninsula",
                "reef",
                "shingle",
                "shoal",
                "spring",
                "strait",
                "water",
                "wetland",
                "arch",
                "arete",
                "bare_rock",
                "blockfield",
                "cave_entrance",
                "cliff",
                "dune",
                "earth_bank",
                "fumarole",
                "hill",
                "peak",
                "ridge",
                "rock",
                "saddle",
                "sand",
                "scree",
                "sinkhole",
                "stone",
                "valley",
                "volcano",
            ],
        );
        overlay.add_category(
            "Office",
            vec![
                "accountant",
                "architect",
                "engineer",
                "financial_advisor",
                "geodesist",
                "graphic_design",
                "lawyer",
                "notary",
                "surveyor",
                "tax_advisor",
                "advertising_agency",
                "company",
                "construction_company",
                "consulting",
                "event_management",
                "financial",
                "it",
                "logistics",
                "moving_company",
                "property_management",
                "publisher",
                "security",
                "telecommunication",
                "transport",
                "administrative",
                "diplomatic",
                "government",
                "harbour_master",
                "politician",
                "quango",
                "water_utility",
                "association",
                "charity",
                "foundation",
                "ngo",
                "political_party",
                "religion",
                "union",
                "educational_institution",
                "research",
                "tutoring",
                "university",
                "airline",
                "guide",
                "travel_agent",
                "visa",
                "employment_agency",
                "energy_supplier",
                "newspaper",
                "coworking",
                "estate_agent",
                "insurance",
                "yes",
            ],
        );
        overlay.add_category(
            "Power",
            vec![
                "cable",
                "catenary_mast",
                "compensator",
                "connection",
                "converter",
                "generator",
                "heliostat",
                "insulator",
                "line",
                "minor_line",
                "plant",
                "pole",
                "portal",
                "substation",
                "switch",
                "switchgear",
                "terminal",
                "tower",
                "transformer",
            ],
        );
        overlay.add_category(
            "PublicTransport",
            vec![
                "stop_position",
                "platform",
                "station",
                "stop_area",
                "stop_area_group",
            ],
        );        
        overlay.add_category(
            "Railway",
            vec![
                "abandoned",
                "construction",
                "proposed",
                "disused",
                "funicular",
                "light_rail",
                "miniature",
                "monorail",
                "narrow_gauge",
                "preserved",
                "rail",
                "subway",
                "tram",
                "bridge",
                "cutting",
                "electrified",
                "embankment",
                "embedded_rails",
                "frequency",
                "railway_track_ref",
                "service_crossover",
                "service_siding",
                "service_spur",
                "service_yard",
                "tunnel",
                "tracks",
                "usage_main",
                "usage_branch",
                "usage_industrial",
                "usage_military",
                "usage_tourism",
                "usage_scientific",
                "usage_test",
                "voltage",
                "halt",
                "stop_position",
                "platform",
                "station",
                "stop",
                "subway_entrance",
                "tram_stop",
                "buffer_stop",
                "crossing",
                "derail",
                "level_crossing",
                "railway_crossing",
                "roundhouse",
                "signal",
                "switch",
                "tram_level_crossing",
                "traverser",
                "turntable",
                "ventilation_shaft",
                "wash",
                "water_crane",
                "user_defined",
            ],
        );
        overlay.add_category(
            "Route",
            vec![
                "bicycle",
                "bus",
                "canoe",
                "detour",
                "ferry",
                "foot",
                "hiking",
                "horse",
                "inline_skates",
                "light_rail",
                "mtb",
                "piste",
                "railway",
                "road",
                "running",
                "ski",
                "subway",
                "train",
                "tracks",
                "tram",
                "trolleybus",
            ],
        );        
        overlay.add_category(
            "Telecom",
            vec![
                "exchange",
                "connection_point",
                "distribution_point",
                "service_device",
                "data_center",
                "line",
            ],
        );
        overlay.add_category(
            "Tourism",
            vec![
                "alpine_hut",
                "apartment",
                "chalet",
                "guest_house",
                "hostel",
                "hotel",
                "motel",
                "wilderness_hut",
                "aquarium",
                "artwork",
                "attraction",
                "camp_pitch",
                "camp_site",
                "caravan_site",
                "gallery",
                "information",
                "museum",
                "picnic_site",
                "theme_park",
                "viewpoint",
                "zoo",
                "yes",
            ],
        );
        overlay.add_category(
            "Water",
            vec![
                "river",
                "oxbow",
                "canal",
                "ditch",
                "lock",
                "fish_pass",
                "lake",
                "reservoir",
                "pond",
                "basin",
                "lagoon",
                "stream_pool",
                "EnumItering_pool",
                "moat",
                "wastewater",
            ],
        );
        overlay.add_category(
            "Waterway",
            vec![
                "river",
                "riverbank",
                "stream",
                "tidal_channel",
                "canal",
                "drain",
                "ditch",
                "pressurised",
                "fairway",
                "dock",
                "boatyard",
                "dam",
                "weir",
                "waterfall",
                "lock_gate",
                "soakhole",
                "turning_point",
                "water_point",
                "fuel",
            ],
        );

        overlay
    }

    pub fn add_category(&mut self, name: &str, items: Vec<&str>) {
        let mut category = Category::default();
        for item in items {
            category.items.insert(item.to_string(), (false, Color32::from_rgb(150, 150, 150)));
        }
        self.categories.insert(name.to_string(), category);
    }

    pub fn get_true_keys_with_category(&self) -> Vec<(String, String)> {
        self.categories.iter()
            .flat_map(|(category_name, category)| {
                if category.disabled {
                    vec![]
                }
                else if category.all {
                    vec![(category_name.clone(), "*".to_string())]
                } else {
                    category.items.iter()
                    .filter_map(move |(item_name, &value)| {
                        if value.0 {
                            Some((category_name.clone(), item_name.clone()))
                        } else {
                            None
                        }
                    }).collect::<Vec<_>>()
                }
            }).collect::<Vec<_>>()
    }

    /// Returns a hashmap of the true keys with their category and key
    pub fn get_true_keys_with_category_with_individual(&self) -> Vec<(String, String)> {
        self.categories.iter()
        .flat_map(|(category_name, category)| {
            if category.disabled {
                vec![]
            }
            else {
                category.items.iter()
                .filter_map(move |(item_name, &value)| {
                    if value.0 {
                        Some((category_name.clone(), item_name.clone()))
                    } else {
                        None
                    }
                }).collect::<Vec<_>>()
            }
        }).collect::<Vec<_>>()
    }

    /*
    pub fn get_true_keys_with_category_with_individual(&self) -> HashMap<String, String> {
        let mut keys = HashMap::new();
        for (cat, key) in self.categories.iter() {
            for (k, (active,_)) in key.items.iter() {
                if *active {
                    keys.insert(cat.clone(), k.clone());
                }
            }
        }
        keys
    }
    */

    pub fn get_disabled_categories(&self) -> Vec<String> {
        self.categories.iter()
            .filter_map(|(category_name, category)| {
                if category.disabled {
                    Some(category_name.clone())
                } else {
                    None
                }
            }).collect::<Vec<_>>()
    }
}

