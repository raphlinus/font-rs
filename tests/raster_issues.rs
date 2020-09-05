extern crate font_rs;

use font_rs::{raster::Raster, geom::Point};

/// Index oob panic found rasterizing "Gauntl" using Bitter-Regular.otf.
#[test]
fn draw_line_index_panic() {
    let mut r = Raster::new(6, 16);
    r.draw_line(&Point::new(5.54, 14.299999), &Point::new(3.7399998, 13.799999));
    r.draw_line(&Point::new(3.7399998, 13.799999), &Point::new(3.7399998, 0.0));
    r.draw_line(&Point::new(3.7399998, 0.0), &Point::new(0.0, 0.10000038));
}
