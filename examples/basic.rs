use macroquad::prelude::*;

use rusty_skelform::time_frame;
use rusty_skelform_macroquad::{animate, draw, load_skelform_armature, AnimOptions};

pub const ARMATURE_NIL: &str = "Armature not found! Please run this in the 'examples' folder.";

#[macroquad::main("SkelForm - Macroquad Basic Demo")]
async fn main() {
    // Load SkelForm armature.
    let armature_filename = "untitled.skf";
    let (mut armature, tex) = load_skelform_armature(armature_filename);

    // Start a timer to use for the animation.
    let time = std::time::Instant::now();

    if armature.bones.len() == 0 {
        println!("{}", ARMATURE_NIL.to_string());
    }

    let mut frame = 0;

    loop {
        clear_background(GRAY);
        let tf0 = time_frame(time, &armature.animations[0], false, true);
        //let tf1 = time_frame(time, &armature.animations[1], false, true);
        let animated_bones = animate(
            &mut armature.bones,
            &mut armature.ik_families,
            &vec![&armature.animations[0]],
            &vec![tf0],
            AnimOptions {
                speed: 1.,
                scale: Vec2::new(-0.25, 0.25),
                position: Vec2::new(screen_width() / 2., screen_height() / 2.),
                blend_frames: vec![30, 30],
                //frame: Some(0),
                ..Default::default()
            },
        );
        draw(&animated_bones, &tex, &vec![&armature.styles[0]]);

        let speed = 10.;
        if is_key_down(KeyCode::Up) {
            armature.bones[0].pos.y += speed;
        }
        if is_key_down(KeyCode::Down) {
            armature.bones[0].pos.y -= speed;
        }
        if is_key_down(KeyCode::Right) {
            armature.bones[0].pos.x += speed;
        }
        if is_key_down(KeyCode::Left) {
            armature.bones[0].pos.x -= speed;
        }

        if is_key_down(KeyCode::W) {
            armature.bones[1].pos.y += speed;
        }
        if is_key_down(KeyCode::S) {
            armature.bones[1].pos.y -= speed;
        }
        if is_key_down(KeyCode::D) {
            armature.bones[1].pos.x += speed;
        }
        if is_key_down(KeyCode::A) {
            armature.bones[1].pos.x -= speed;
        }

        if armature.bones.len() == 0 {
            let white = Color::from_rgba(255, 255, 255, 255);
            draw_text(ARMATURE_NIL, 10., 25., 25., white);
        }

        next_frame().await;
        frame += 1;
    }
}
