extern crate osmpbfreader;

use osmpbfreader::{groups, primitive_block_from_blob};
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

fn main() {
    // check if arguments are right
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} pbf_file", args[0]);
        return;
    }

    let mut valid_maxspeed = HashSet::new();
    valid_maxspeed.insert("none".to_string());
    valid_maxspeed.insert("signals".to_string());
    valid_maxspeed.insert("walk".to_string());

    // collect all valid speed-strings
    let country_codes = vec![
        "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX",
        "AZ", "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ",
        "BR", "BS", "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK",
        "CL", "CM", "CN", "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM",
        "DO", "DZ", "EC", "EE", "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", "FM", "FO", "FR",
        "GA", "GB", "GD", "GE", "GF", "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS",
        "GT", "GU", "GW", "GY", "HK", "HM", "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN",
        "IO", "IQ", "IR", "IS", "IT", "JE", "JM", "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN",
        "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC", "LI", "LK", "LR", "LS", "LT", "LU", "LV",
        "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK", "ML", "MM", "MN", "MO", "MP", "MQ",
        "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA", "NC", "NE", "NF", "NG", "NI",
        "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG", "PH", "PK", "PL", "PM",
        "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW", "SA", "SB", "SC",
        "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR", "ST", "SS", "SV",
        "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO", "TR",
        "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
        "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW",
    ];
    // from https://wiki.openstreetmap.org/wiki/Speed_limits
    let zone_types = vec![
        "bicycle_road",
        "living_street",
        "motorway",
        "national",
        "nsl_dual",
        "nsl_restricted",
        "nsl_single",
        "pedestrian_zone",
        "rural",
        "urban",
        "urban:primary",
        "urban:secondary",
        "walk",
        "zone:30",
        "zone30",
    ];
    for country_code in country_codes {
        for zone_type in &zone_types {
            valid_maxspeed.insert(format!("{}:{}", country_code, zone_type));
        }
    }

    // read pbf file
    let filename = std::env::args_os().nth(1).unwrap();
    let path = Path::new(&filename);
    if !path.exists() {
        println!("{} not found", filename.into_string().unwrap());
        std::process::exit(1);
    }
    let r = File::open(&path).unwrap();
    let mut pbf = osmpbfreader::OsmPbfReader::new(r);

    // first store all way-IDs that are having the "highway" tag. also store speed-limit
    for block in pbf.blobs().map(|b| primitive_block_from_blob(&b.unwrap())) {
        let block = block.unwrap();
        for group in block.get_primitivegroup().iter() {
            for way in groups::ways(&group, &block) {
                if way.tags.contains_key("highway") {
                    let _highway = way.tags.get("highway").unwrap().trim();
                    if way.tags.contains_key("maxspeed") {
                        let max_speed = way.tags.get("maxspeed").unwrap().trim();
                        if !valid_maxspeed.contains(max_speed) {
                            let tmp = &max_speed.replace(" mph", "");
                            let max_speed_slim = &tmp.replace(" knots", "");
                            let speed = max_speed_slim.parse::<usize>();
                            // get way ID
                            let osm_id = way.id;
                            // sort by mistakes and print osm ids by error
                            match speed {
                                Ok(ok) => {
                                    if ok < 1 as usize {
                                        println!(
                                            "https://www.openstreetmap.org/way/{:?} \t{:?}",
                                            osm_id.0, max_speed
                                        );
                                    }
                                }
                                Err(_err) => {
                                    println!(
                                        "https://www.openstreetmap.org/way/{:?} \t{:?}",
                                        osm_id.0, max_speed
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
