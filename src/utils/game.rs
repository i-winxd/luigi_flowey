
use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use raylib::prelude::*;
use rustfft::{num_complex::Complex, FftPlanner};
use crate::utils::audio::WavAudio;
use crate::utils::draw_text_anchor::{draw_text_anchored, TextConfig};

struct GameState<'a> {
    sprites: Vec<MovableSprite<'a>>,
}

impl<'a> GameState<'a> {
    /// Updates the position of all sprites stored in this state.
    fn update_all(&mut self) {
        for spr in self.sprites.iter_mut() {
            spr.update();
        }
    }

    /// Draws all the sprites stored in this state.
    fn draw_all(&self, d: &mut RaylibDrawHandle) {
        for spr in self.sprites.iter() {
            spr.draw(d);
        }
    }

    fn shuffle(&mut self, rng: &mut ThreadRng) {
        self.sprites.shuffle(rng);
    }
}

struct BBox {
    anchor: (i32, i32),
    size: (i32, i32),
}

struct MovableSprite<'a> {
    texture: &'a Texture2D,
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
    scr_w: i32,
    scr_h: i32,
}

struct HealthBar {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    hp: f64, // from 0 to 1
}

impl HealthBar {
    fn draw(&self, d: &mut RaylibDrawHandle) {
        let hp_color = Color::new(254, 255, 0, 255);
        let dmg_color = Color::new(191, 0, 1, 255);
        d.draw_rectangle(self.x, self.y, self.w, self.h, dmg_color);
        d.draw_rectangle(
            self.x,
            self.y,
            ((self.w as f64 * self.hp.powf(1.4)) as i32).max(2),
            self.h,
            hp_color,
        );
    }

    fn set_center(&mut self, x: i32, y: i32) {
        self.x = x - self.w / 2;
        self.y = y - self.h / 2;
    }

    fn take_damage(&mut self, dmg: f64) {
        self.hp -= dmg;
        self.hp = self.hp.clamp(0f64, 1f64);
    }

    fn set_hp(&mut self, new_hp: f64) {
        self.hp = new_hp.clamp(0f64, 1f64);
    }
}

struct MultiSprite<'a> {
    textures: Vec<&'a Texture2D>,
    current_texture: usize,
    x: i32,
    y: i32,
    gap: i32,
    c_gap: i32,
}

impl<'a> MultiSprite<'a> {
    fn new(
        textures: Vec<&'a Texture2D>,
        current_texture: usize,
        x: i32,
        y: i32,
        gap: i32,
    ) -> MultiSprite<'a> {
        MultiSprite {
            textures,
            current_texture,
            x,
            y,
            gap,
            c_gap: 0,
        }
    }

    fn active(&self) -> bool {
        return self.current_texture < self.textures.len();
    }

    fn set_texture(&mut self, tex: usize) {
        self.current_texture = tex;
    }

    /// Returns the center.
    /// Size always based on the first frame. Returns (0, 0)
    /// if the sprite is somehow empty.
    fn center(&self) -> (i32, i32) {
        let cur_tex = self.textures.get(0);
        match cur_tex {
            None => (0, 0),
            Some(ct) => (self.x + ct.width / 2, self.y + ct.height / 2),
        }
    }

    /// Sets the position of the sprite, where x and y is
    /// where the center of this sprite will be.
    ///
    /// If current_texture is oob, it will assume the texture is the size of index 0
    fn set_position_center(&mut self, x: i32, y: i32) {
        let ct = self.textures.get_mut(self.current_texture);
        if let Some(tex) = ct {
            self.x = x - (tex.width() / 2);
            self.y = y - (tex.height()) / 2;
        } else if let Some(tex) = self.textures.get_mut(0) {
            self.x = x - (tex.width() / 2);
            self.y = y - (tex.height()) / 2;
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle) {
        let tx_current = self.textures.get(self.current_texture);
        if let Some(tx_c) = tx_current {
            d.draw_texture(tx_c, self.x, self.y, Color::WHITE);
        }
    }

    /// Advances self to the next frame, unless it is at the end already.
    fn advance(&mut self) {
        if (self.current_texture <= self.textures.len()) {
            if (self.c_gap < self.gap) {
                self.c_gap += 1;
            } else {
                self.current_texture += 1;
                self.c_gap = 0;
            }
        }
    }

    /// Makes this sprite invisible. Moving operations are not impacted.
    fn make_invisible(&mut self) {
        self.current_texture = self.textures.len();
        self.c_gap = 0;
    }

    fn reset(&mut self) {
        self.current_texture = 0;
        self.c_gap = 0;
    }
}

impl<'a> MovableSprite<'a> {
    /// Construct a new sprite from a texture.
    /// The texture is immutable.
    fn new(
        texture: &'a Texture2D,
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
        scr_w: i32,
        scr_h: i32,
    ) -> MovableSprite<'a> {
        MovableSprite {
            texture,
            x,
            y,
            dx,
            dy,
            scr_w,
            scr_h,
        }
    }

    /// Returns if point collides with the sprite anywhere
    fn collides(&self, x: i32, y: i32) -> bool {
        (self.x < x && x < self.x + self.texture.width())
            && (self.y < y && y < self.y + self.texture.height())
    }

    /// Return the bounding box.
    /// Returns top left anchor and the size of the box.
    fn get_bb(&self) -> BBox {
        BBox {
            anchor: (self.x, self.y),
            size: (self.texture.width(), self.texture.height()),
        }
    }
    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_texture(&self.texture, self.x, self.y, Color::WHITE);
    }

    /// Updates my position.
    /// Called every frame.
    fn update(&mut self) {
        let rb_x = self.x + self.texture.width();
        let rb_y = self.y + self.texture.height();

        if ((self.x + self.dx) <= 0 && self.dx < 0) {
            self.dx = -self.dx;
        }
        if ((self.y + self.dy) <= 0 && self.dy < 0) {
            self.dy = -self.dy
        }

        if ((rb_x + self.dx) >= self.scr_w - 1 && self.dx > 0) {
            self.dx = -self.dx;
        }
        if ((rb_y + self.dy) >= self.scr_h - 1 && self.dy > 0) {
            self.dy = -self.dy;
        }

        self.x += self.dx;
        self.y += self.dy;
    }
}

fn rng_choice_2<T>(rng: &mut ThreadRng, v1: T, v2: T) -> T {
    let random_bool: bool = rng.gen();
    if random_bool {
        v1
    } else {
        v2
    }
}

fn get_textures_from_sprite(
    sprites: Vec<String>,
    scale: f64,
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
) -> Vec<Texture2D> {
    let mut fv = Vec::new();
    for spr in sprites.iter() {
        let mut img_loaded = Image::load_image(spr.as_str()).unwrap();
        img_loaded.resize_nn(
            ((img_loaded.width() as f64) * scale) as i32,
            ((img_loaded.width() as f64) * scale) as i32,
        );
        fv.push(rl.load_texture_from_image(thread, &img_loaded).unwrap());
    }
    fv
}

pub fn play_it() {
    let scr_w = 800;
    let scr_h = 600;

    let (mut rl, thread) = raylib::init()
        .size(scr_w, scr_h)
        .title("FIND LUIGI")
        .build();

    let mut audio = RaylibAudio::init_audio_device().unwrap();
    let hurt = audio.new_sound("resources/snd_hurt1.wav").unwrap();
    let dead = audio.new_sound("resources/snd_hurt1_c.wav").unwrap();
    let heart_break = audio.new_sound("resources/snd_break1.wav").unwrap();
    let mut luigi_image = Image::load_image("resources/LUIGI_WANTED.png").unwrap();
    let mut wario_image = Image::load_image("resources/WARIO_WANTED.png").unwrap();
    let mut mario_image = Image::load_image("resources/MARIO_WANTED.png").unwrap();
    let mut yoshi_image = Image::load_image("resources/YOSHI_WANTED.png").unwrap();
    let mut ut_soul_image = Image::load_image("resources/UT_SOUL.png").unwrap();
    let mut ut_soul_cracked = Image::load_image("resources/UT_SOUL_BREAK.png").unwrap();
    let mut crosshair_image = Image::load_image("resources/Crosshair_larger.png").unwrap();
    let sprite_rescale = 2;

    for img in [
        &mut luigi_image,
        &mut wario_image,
        &mut mario_image,
        &mut yoshi_image,
        &mut crosshair_image,
        &mut ut_soul_image,
        &mut ut_soul_cracked,
    ] {
        img.resize_nn(img.width() * sprite_rescale, img.height() * sprite_rescale);
    }

    let luigi_texture = rl.load_texture_from_image(&thread, &luigi_image).unwrap();
    let wario_texture = rl.load_texture_from_image(&thread, &wario_image).unwrap();
    let mario_texture = rl.load_texture_from_image(&thread, &mario_image).unwrap();
    let yoshi_texture = rl.load_texture_from_image(&thread, &yoshi_image).unwrap();
    let crosshair_texture = rl
        .load_texture_from_image(&thread, &crosshair_image)
        .unwrap();
    let ut_soul_texture = rl.load_texture_from_image(&thread, &ut_soul_image).unwrap();
    let ut_soul_cracked_texture = rl
        .load_texture_from_image(&thread, &ut_soul_cracked)
        .unwrap();
    let custom_font = rl
        .load_font_ex(&thread, "resources/LINESeedSans_Bd.ttf", 200, None)
        .unwrap();

    let text_config = TextConfig {
        spacing: 0.0,
        tint: Color::WHITE,
        paragraph_align: 1.0,
        anchor_x: 0.5,
        anchor_y: 0.0,
        line_spacing: 0.8,
    };
    let mut rng = rand::thread_rng();

    let audio = WavAudio::new("resources/wav_test_3.wav").unwrap();

    let mut total_elapsed_time: f64 = 0.0;

    // let pts: Vec<i32> = vec![0, 100, 200, 100, 200, 100, 200, 100, 200, 0];
    let luigi_count = 1;
    let yoshi_count = 20;
    let mario_count = 20;
    let wario_count = 20;
    let default_vel = 2;

    let mut game_state = GameState {
        sprites: {
            let process_pairs = [
                (luigi_count, &luigi_texture),
                (yoshi_count, &yoshi_texture),
                (mario_count, &mario_texture),
                (wario_count, &wario_texture),
            ];
            let mut sprites: Vec<MovableSprite> = Vec::new();
            for (count, tex) in process_pairs {
                for _ in 0..count {
                    sprites.push(preconstruct_sprite(
                        scr_w,
                        scr_h,
                        &thread,
                        tex,
                        &mut rng,
                        default_vel,
                    ));
                }
            }
            sprites
        },
    };
    game_state.shuffle(&mut rng);
    let mut crosshair_spr = MultiSprite::new(vec![&crosshair_texture], 0, 0, 0, 1);
    let mut heart_spr = MultiSprite::new(
        vec![&ut_soul_texture, &ut_soul_cracked_texture],
        0,
        (scr_w / 2) as i32,
        ((scr_h as f64) * 0.70) as i32,
        0,
    );

    let expl_textures = get_textures_from_sprite(
        (0..17)
            .map(|v| format!("resources/exp/EXP_F{}.png", v))
            .collect(),
        2.0,
        &mut rl,
        &thread,
    );
    let mut cur_explosion = MultiSprite::new(
        expl_textures.iter().map(|v| v).collect(),
        expl_textures.len(),
        0,
        0,
        1,
    );

    let audio_duration = 0.30;
    let expected_length = audio.get_index_from_secs(audio_duration) as usize;
    let mut planner = FftPlanner::<f64>::new();
    rl.set_target_fps(60);
    let mov_vel = 5;
    let mov_vel_f = 5f32;
    let mut health_bar = HealthBar {
        x: 0,
        y: 0,
        w: 128,
        h: 32,
        hp: 1.0,
    };
    health_bar.set_center(scr_w / 2, (scr_h as f64 * 0.92) as i32);
    let i_frames_per_hit = 60;
    let mut cur_i_frames = 0;
    let mut dead_for = 0;
    let mut playing = false;
    let mut frames_since_play = 0;
    while !rl.window_should_close() {
        if (cur_i_frames > 0) {
            cur_i_frames -= 1;
        }
        let enter_pressed = rl.is_key_down(KeyboardKey::KEY_ENTER);
        if playing && health_bar.hp > 0.0 {
            frames_since_play += 1;
        }
        if health_bar.hp > 0f64 && playing {
            let mut mov_vel = Vector2::new(0f32, 0f32);
            if rl.is_key_down(KeyboardKey::KEY_W) {
                mov_vel += Vector2::new(0f32, -1f32);
            }
            if rl.is_key_down(KeyboardKey::KEY_A) {
                mov_vel += Vector2::new(-1f32, 0f32);
            }
            if rl.is_key_down(KeyboardKey::KEY_S) {
                mov_vel += Vector2::new(0f32, 1f32);
            }
            if rl.is_key_down(KeyboardKey::KEY_D) {
                mov_vel += Vector2::new(1f32, 0f32);
            }
            mov_vel =
                (mov_vel / ((mov_vel.x * mov_vel.x) + (mov_vel.y * mov_vel.y)).sqrt()) * mov_vel_f;
            heart_spr.x += (mov_vel.x.round() as i32);
            heart_spr.y += (mov_vel.y.round() as i32);
            heart_spr.x = heart_spr.x.clamp(0, scr_w - heart_spr.textures[0].width());
            heart_spr.y = heart_spr.y.clamp(0, scr_h - heart_spr.textures[0].height());
        }
        total_elapsed_time += rl.get_frame_time() as f64;
        let pressed_state = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let mouse_position = rl.get_mouse_position();
        if rl.is_key_down(KeyboardKey::KEY_ENTER) {
            playing = true;
        }
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        if health_bar.hp > 0f64 {
            if playing {
                game_state.update_all();
            }
            if (cur_i_frames == 0 || (cur_i_frames / 6) % 2 == 0) {
                heart_spr.draw(&mut d);
            }
            if (cur_i_frames == 0) {
                let (hh, hv) = heart_spr.center();
                let collides = game_state.sprites.iter().any(|x| x.collides(hh, hv));
                if collides && playing {
                    hurt.play();
                    cur_i_frames = i_frames_per_hit;
                    health_bar.take_damage(0.08);
                }
            }

            game_state.draw_all(&mut d);
            health_bar.draw(&mut d);

            if !playing {
                draw_text_anchored(
                    &mut d,
                    &custom_font,
                    vec!["Press enter to start"],
                    Vector2::new(scr_w as f32 / 2.0f32, scr_h as f32 / 2.0f32),
                    100f32,
                    &text_config,
                );
            }
        } else {
            dead_for += 1;
            if dead_for == 60 {
                heart_spr.x -= 4;
                heart_spr.y -= 1;
                heart_spr.set_texture(1);
                heart_break.play();
            }
            heart_spr.draw(&mut d);
            if dead_for >= 60 {
                draw_text_anchored(
                    &mut d,
                    &custom_font,
                    vec![
                        format!("You survived {}s", frames_since_play / 60).as_str(),
                        "Press enter to restart",
                    ],
                    Vector2::new(scr_w as f32 / 2.0f32, scr_h as f32 / 2.0f32),
                    50f32,
                    &text_config,
                );
                if enter_pressed {
                    health_bar.set_hp(1.0);
                    dead_for = 0;
                    heart_spr.set_texture(0);
                    frames_since_play = 0;
                }
            }
        }
        // draw_explosion(&mut cur_explosion, pressed_state, mouse_position, &mut d);
        // draw_cursor(&mut crosshair_spr, mouse_position, &mut d);
    }
}

fn draw_explosion(
    cur_explosion: &mut MultiSprite,
    pressed_state: bool,
    mouse_position: Vector2,
    mut d: &mut RaylibDrawHandle,
) {
    cur_explosion.draw(&mut d);
    cur_explosion.advance();
    if pressed_state {
        cur_explosion.set_position_center(mouse_position.x as i32, mouse_position.y as i32);
        cur_explosion.reset();
    }
}

fn draw_cursor(
    crosshair_spr: &mut MultiSprite,
    mouse_position: Vector2,
    mut d: &mut RaylibDrawHandle,
) {
    crosshair_spr.set_position_center(mouse_position.x as i32, mouse_position.y as i32);
    crosshair_spr.draw(&mut d);
}

fn preconstruct_sprite<'a>(
    scr_w: i32,
    scr_h: i32,
    thread: &'a RaylibThread,
    texture: &'a Texture2D,
    mut rng: &mut ThreadRng,
    default_vel: i32,
) -> MovableSprite<'a> {
    MovableSprite::new(
        &texture,
        rng.gen_range(0..scr_w),
        rng.gen_range(0..scr_h),
        rng_choice_2(&mut rng, -default_vel, default_vel),
        rng_choice_2(&mut rng, -default_vel, default_vel),
        scr_w,
        scr_h,
    )
}
