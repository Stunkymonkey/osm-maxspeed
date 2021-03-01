extern crate osmpbfreader;

use clap::{App, Arg};
use osmpbfreader::{groups, primitive_block_from_blob};
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

fn main() {
    // check arguments
    let matches = App::new("osm-maxspeed")
        .version("0.1")
        .author("Felix Bühler")
        .about("little validator for osm data")
        .arg(
            Arg::with_name("osm-file")
                .short("f")
                .long("file")
                .help("Set osm-file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("disable-maxspeed")
                .short("m")
                .long("disable-maxspeed")
                .help("Disable the maxspeed-validator (default enabled)"),
        )
        .arg(
            Arg::with_name("disable-type")
                .short("t")
                .long("disable-type")
                .help("Disable the maxspeed:type-validator (default enabled)"),
        )
        .arg(
            Arg::with_name("disable-source")
                .short("s")
                .long("disable-source")
                .help("Disable the source-validator (default enabled)"),
        )
        .get_matches();

    let show_speed = !matches.is_present("disable-maxspeed");
    let show_type = !matches.is_present("disable-type");
    let show_source = !matches.is_present("disable-source");

    let mut valid_maxspeed = HashSet::new();
    valid_maxspeed.insert("none".to_string());
    // only a proposal: https://wiki.openstreetmap.org/wiki/Key:maxspeed
    valid_maxspeed.insert("walk".to_string());

    let mut valid_maxspeed_type = HashSet::new();
    valid_maxspeed_type.insert("sign".to_string());

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
        "zone",
        "zone:20",
        "zone20",
        "zone:30",
        "zone30",
        "zone:50",
        "zone50",
    ];
    for country_code in country_codes {
        for zone_type in &zone_types {
            valid_maxspeed_type.insert(format!("{}:{}", country_code, zone_type));
        }
    }

    let mut valid_source_maxspeed = HashSet::new();
    valid_source_maxspeed.insert("markings".to_string());
    for codes in &valid_maxspeed_type {
        valid_source_maxspeed.insert(codes.to_string());
    }

    // read pbf file
    let path = if let Some(filename) = matches.value_of("osm-file") {
        let file_path = Path::new(filename);
        if !file_path.exists() {
            println!("{} not found", filename);
            std::process::exit(1);
        }
        file_path
    } else {
        println!("no file provided. exiting");
        std::process::exit(1);
    };

    let r = File::open(&path).unwrap();
    let mut pbf = osmpbfreader::OsmPbfReader::new(r);

    // first store all way-IDs that are having the "highway" tag. also store speed-limit
    for block in pbf.blobs().map(|b| primitive_block_from_blob(&b.unwrap())) {
        let block = block.unwrap();
        for group in block.get_primitivegroup().iter() {
            for way in groups::ways(&group, &block) {
                if way.tags.contains_key("highway") {
                    let _highway = way.tags.get("highway").unwrap().trim();
                    if show_speed && way.tags.contains_key("maxspeed") {
                        let max_speed = way.tags.get("maxspeed").unwrap().trim();
                        if !valid_maxspeed.contains(max_speed) {
                            let tmp = &max_speed.replace(" mph", "");
                            let max_speed_slim = &tmp.replace(" knots", "");
                            let speed = max_speed_slim.parse::<usize>();
                            match speed {
                                Ok(ok) => {
                                    if ok < 1 as usize {
                                        println!(
                                            "maxspeed\t\t\thttps://www.openstreetmap.org/way/{:?} \t{:?}",
                                            way.id.0, max_speed
                                        );
                                    }
                                }
                                Err(_err) => {
                                    println!(
                                        "maxspeed\t\t\thttps://www.openstreetmap.org/way/{:?} \t{:?}",
                                        way.id.0, max_speed
                                    );
                                }
                            }
                        }
                    }
                    if show_type && way.tags.contains_key("maxspeed:type") {
                        let max_speed_type = way.tags.get("maxspeed:type").unwrap().trim();
                        if !valid_maxspeed_type.contains(max_speed_type) {
                            println!(
                                "maxspeed:type\t\thttps://www.openstreetmap.org/way/{:?} \t{:?}",
                                way.id.0, max_speed_type
                            );
                        }
                    }
                    if show_source && way.tags.contains_key("source:maxspeed") {
                        let source_max_speed = way.tags.get("source:maxspeed").unwrap().trim();
                        if !valid_source_maxspeed.contains(source_max_speed) {
                            println!(
                                "source:maxspeed\t\thttps://www.openstreetmap.org/way/{:?} \t{:?}",
                                way.id.0, source_max_speed
                            );
                        }
                    }
                }
            }
        }
    }
}
