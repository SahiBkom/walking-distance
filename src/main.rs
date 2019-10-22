use geo::prelude::*;
use geo_types::Point;
use gpx::{read, Gpx, Track};
use log::*;
use std::fs::File;
use std::io::BufReader;

struct PointTime {
    point: Point<f64>,
    time: chrono::DateTime<chrono::Utc>,
    error: bool,
}

impl PointTime {
    fn new(point: Point<f64>, time: chrono::DateTime<chrono::Utc>) -> PointTime {
        PointTime {
            point,
            time,
            error: false,
        }
    }
}

fn main() {
    env_logger::init();

    let file = File::open("/home/johan/proj/walking-distance/data/Spoor_JOTI 2019.gpx").unwrap();
    let reader = BufReader::new(file);

    let gpx: Gpx = read(reader).unwrap();

    let track: &Track = &gpx.tracks[0];
    info!("track.name {:?}", track.name);

    let mut last_ok: Option<PointTime> = None;

    for segment in &track.segments {
        println!("Segment");

        let mut distance_total: f64 = 0.0f64;
        let iter = &mut segment.points.iter().peekable();
        while let Some(i) = iter.next() {
            if let Some(p) = iter.peek() {
                let distance = i.point().vincenty_distance(&p.point());
                if let (Some(it), Some(pt)) = (i.time, p.time) {
                    let time: f64 = pt.signed_duration_since(it).num_seconds() as f64;
                    if let Ok(distance) = distance {
                        debug!(
                            "distance:{:.3?}M time:{:.3}sec speed:{:.3}m/s",
                            distance,
                            time,
                            (distance / time)
                        );
                        if (distance / time) < 3f64 {
                            distance_total += distance;
                            print!("-");

                            if let Some(y) = &last_ok {
                                let time = pt.signed_duration_since(y.time).num_seconds() as f64;;
                                if y.error && (time < 120f64) {
                                    if let Ok(distance) = y.point.vincenty_distance(&i.point()) {
                                        if (distance / time) < 3.0f64 {
                                            distance_total += distance;
                                            debug!("=({:.2} {}) ", distance, time);
                                        } else {
                                            print!("?({:.2}M {}sec) ", distance, time);
                                        }
                                    }
                                } else {
                                    if y.error {
                                        print!("({:.1}min)", time / 60.0f64);
                                    }
                                }
                            }

                            last_ok = Some(PointTime::new(p.point(), pt));
                        } else {
                            print!("_");

                            if let Some(y) = &mut last_ok {
                                y.error = true;
                            }
                        }
                    }
                }
            }
        }
        println!("");
        println!("Afstand {}", distance_total);
    }
}
