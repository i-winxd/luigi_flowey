use raylib::{color::Color, prelude::RaylibDraw};

pub fn render_graph<T: RaylibDraw>(d: &mut T, points: &Vec<i32>,
    y: i32,
    left_point: i32, right_point: i32,
    max_height: i32,
    bars: i32,
    gap: i32
) {
    let as_abs: Vec<i32> = points.iter().map(|v| v.abs()).collect();
    let width = right_point - left_point + gap;
    let bar_width = width / bars;  // INCLUDING THE GAP
    
    for i in 0..bars {
        d.draw_rectangle(left_point + bar_width * i, y, bar_width - gap, 
            interpolate_vec(&as_abs, (i as f64/bars as f64) * as_abs.len() as f64) as i32,
        Color::BLACK);
    }

}

fn interpolate_vec(vec: &Vec<i32>, idx: f64) -> f64 {
    let vec_len = vec.len();
    if vec_len == 0 {
        return 0.0
    }
    if idx > (vec_len - 1) as f64 {
        return (vec[vec_len - 1]) as f64;
    }
    if idx < 0.0 {
        return (vec[0]) as f64;
    }
    let flo = idx.floor() as usize;
    let cei = idx.ceil() as usize;
    let progression = idx % 1.0;
    (vec[cei] as f64) * progression + (vec[flo] as f64) * (1.0 - progression)

}