use raylib::prelude::*;


fn argmax_f32(values: &Vec<f32>) -> Option<usize> {
    if values.is_empty() {
        return None;
    }

    let mut max_index = 0;
    let mut max_value = values[0];

    for (i, &value) in values.iter().enumerate() {
        if value > max_value {
            max_index = i;
            max_value = value;
        }
    }
    Some(max_index as usize)
}

pub struct TextConfig {
    pub spacing: f32,
    pub tint: Color,
    pub paragraph_align: f32,
    pub anchor_x: f32,
    pub anchor_y: f32,

    // % of vertical offset per line
    pub line_spacing: f32,
}


/// Draws text that supports paragraph alignment and anchoring.
/// 
/// For all align and anchor arguments:
/// 0 if left aligned, 0.5 for center, 1 if right aligned, transposing
/// if needed
/// 
/// * `paragraph_align` - Affects each line with respect to the longest line.
/// * `anchor_x` - 0 if position is the left anchor, 1 if position is the right
///                anchor
/// * `anchor_y` - ditto, top -> bottom
/// 
pub fn draw_text_anchored<T: RaylibDraw>(d: &mut T,
                          custom_font: &Font, 
                          lines: Vec<&str>, 
                          position: impl Into<Vector2>,
                          font_size: f32,
                          config: &TextConfig
) {
    let (spacing, tint, paragraph_align, anchor_x, anchor_y,
        line_spacing) = (config.spacing, config.tint, config.paragraph_align, config.anchor_x, config.anchor_y,
        config.line_spacing);
    let pos = position.into();
    let mut boxes: Vec<Vector2> = Vec::new();

    for line in &lines {
        let measured = custom_font.measure_text(line, font_size, spacing);
        boxes.push(measured);
    }
    let bounding_box_height: f32 = boxes.iter().map(|item| { item.y }).sum::<f32>() + spacing * ((lines.len() - 1) as f32);

    let was_iter: Vec<f32> = boxes.iter().map(|item| { item.x }).collect();
    let argmax_of = argmax_f32(&was_iter);

    let bounding_box_width: f32 = match argmax_of {
        None => { 0.0 }
        Some(val) => {
            was_iter[val] + (lines[val].len() as f32) * spacing
        }
    };

    // assume anchor is @ top left for now and text is left aligned
    let positions: Vec<Vector2> = boxes.iter().enumerate().map(|(index, cur_box)| {
        let line_height = cur_box.y + spacing;
        let cur_box = &(boxes[index]);
        let para_offset: f32 = (bounding_box_width - cur_box.x) * paragraph_align;
        let horiz_box_offset: f32 = bounding_box_width * anchor_x; 
        let vert_box_offset: f32 = bounding_box_height * anchor_y;

        let line_height_float = (line_height * (index as f32)) * line_spacing;
        Vector2::new(
            pos.x + para_offset - horiz_box_offset,
             pos.y + line_height_float - vert_box_offset
            )
    }).collect();

    
    for (idx, t_pos) in positions.iter().enumerate() {

        // d.draw_rectangle(t_pos.x as i32, 
        //     t_pos.y as i32, 
        //     boxes[idx].x as i32, 
        //     boxes[idx].y as i32, 
        //     Color::GRAY);

        d.draw_text_ex(
            custom_font,
            lines[idx],
            t_pos,
            font_size,
            spacing,
            tint
        );
    }

}
