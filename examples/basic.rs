use std::time::Instant;

use macroquad::prelude as mqr;
use mqr::*;
use rusty_skelform as skf;
use rusty_skelform_macroquad as skf_mq;
use skf::time_frame;

pub const ARMATURE_NIL: &str = "Armature not found! Please run this in the 'examples' folder.";
pub const INSTRUCTIONS: &str =
    "A - Move Left\nD - Move Right\nSpace - Jump\n1, 2 - Change outfit\nSkellington will look at and reach for cursor";

#[macroquad::main("SkelForm - Macroquad Basic Demo")]
async fn main() {
    let armature_filename = "Untitled.skf";
    if !std::fs::exists(armature_filename).unwrap() {
        println!("\n{}\n", ARMATURE_NIL.to_string());
        return;
    }

    // load SkelForm armature
    let (mut skellington, skel_texes) = skf_mq::load(armature_filename);

    // timer for animations
    let mut time = Instant::now();

    if skellington.bones.len() == 0 {
        println!("{}", ARMATURE_NIL.to_string());
    }

    let mut dir = 1.;
    let mut pos = Vec2::new(100., -100.);
    let mut prev_pos = pos;
    let mut vel = Vec2::new(0., 0.);
    let ground_y = screen_height() / 2. + 200.;
    let mut last_anim_idx = 0;
    let mut anim_idx: usize;
    let mut grounded = false;
    let mut skel_style = 1;

    loop {
        clear_background(GRAY);

        if pos.y < ground_y {
            vel.y += 0.05;
        } else {
            vel.y = 0.;
            pos.y = ground_y
        }

        pos += vel;

        let speed = 5.;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::D) {
            anim_idx = 1;
        } else {
            anim_idx = 0;
        }
        if is_key_down(KeyCode::D) {
            pos.x += speed;
            dir = 1.;
        }
        if is_key_down(KeyCode::A) {
            pos.x -= speed;
            dir = -1.;
        }
        if is_key_pressed(KeyCode::Space) && grounded {
            vel.y = -5.;
            pos.y = ground_y - 1.;
        }
        if is_key_pressed(KeyCode::Key1) {
            skel_style = 1;
        }
        if is_key_pressed(KeyCode::Key2) {
            skel_style = 0;
        }

        if vel.y < 0. {
            anim_idx = 2;
            grounded = false;
        } else if vel.y > 0. {
            anim_idx = 3;
            grounded = false;
        } else {
            grounded = true;
        }

        if last_anim_idx != anim_idx {
            time = Instant::now();
            last_anim_idx = anim_idx;
        }

        let skel_scale = 0.125;

        draw_skellington(
            time,
            &mut skellington,
            anim_idx,
            pos,
            prev_pos,
            dir,
            &skel_texes,
            skel_scale,
            skel_style,
        );

        let white = Color::from_rgba(255, 255, 255, 255);
        if skellington.bones.len() == 0 {
            draw_text(ARMATURE_NIL, 10., 25., 25., white);
        } else {
            draw_multiline_text(INSTRUCTIONS, 10., 25., 35., None, white);
        }

        prev_pos = pos;

        next_frame().await;
    }
}

fn draw_skellington(
    time: std::time::Instant,
    armature: &mut skf::Armature,
    anim_idx: usize,
    pos: Vec2,
    prev_pos: Vec2,
    dir: f32,
    texes: &Vec<mqr::Texture2D>,
    skel_scale: f32,
    skel_style: usize,
) {
    // process animation(s)
    // let tf0 = time_frame(time, &armature.animations[anim_idx], false, true);
    let velocity = Vec2::new(pos.x - prev_pos.x, -(pos.y - prev_pos.y)) * 10.;
    let skel_options = skf_mq::ConstructOptions {
        speed: 1.,
        scale: mqr::Vec2::new(0.25, 0.25),
        position: Vec2::new(pos.x, pos.y),
        velocity,
        ..Default::default()
    };
    // skf_mq::animate(
    //     &mut armature.bones,
    //     &vec![&armature.animations[anim_idx]],
    //     &vec![tf0],
    //     &vec![20],
    // );

    // construct and draw armature
    skf_mq::construct(armature, &skel_options);

    let styles = &vec![&armature.styles[0]];
    skf_mq::draw(&mut armature.cached_bones, &texes, styles);
}
