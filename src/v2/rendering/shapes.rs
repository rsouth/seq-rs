// use crate::v2::rendering::render_context::RenderingConstants::DiagramPadding;
// use raqote::{Path, PathBuilder};
// use std::time::Instant;
//
// pub fn rect_path(width: i32, height: i32) -> Path {
//     let start = Instant::now();
//     let rect_x = DiagramPadding.value() as f32;
//     let rect_y = DiagramPadding.value() as f32;
//     let rect_w = width as f32 - (2. * DiagramPadding.value() as f32);
//     let rect_h = height as f32 - (2. * DiagramPadding.value() as f32);
//     let mut rect_path = PathBuilder::new();
//     rect_path.rect(rect_x, rect_y, rect_w, rect_h);
//
//     let rpath = rect_path.finish();
//     trace!(
//         "Created rectangle path in {}µs - {:?}",
//         start.elapsed().as_micros(),
//         &rpath.ops.to_vec()
//     );
//     rpath
// }
