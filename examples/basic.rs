use macroquad::prelude::*;

use rusty_skelform_macroquad::{animate, load_skelform_armature, AnimOptions};

pub const ARMATURE_NIL: &str = "Armature not found! Please run this in the 'examples' folder.";

#[macroquad::main("SkelForm - Macroquad Basic Demo")]
async fn main() {
    // Load SkelForm armature.
    let armature_filename = "skellington.skf";
    let (mut armature, tex) = load_skelform_armature(armature_filename);

    // Start a timer to use for the animation.
    let time = std::time::Instant::now();

    if armature.bones.len() == 0 {
        println!("{}", ARMATURE_NIL.to_string());
    }

    let mut frame = 0;

    loop {
        animate(
            &mut armature,
            &tex,
            1,
            Some(time),
            true,
            true,
            Some(AnimOptions {
                speed: 1.,
                scale_factor: 0.25,
                pos_offset: Vec2::new(screen_width() / 2., screen_height() / 2.),
                //frame: Some(0),
                ..Default::default()
            }),
        );

        if armature.bones.len() == 0 {
            draw_text(
                ARMATURE_NIL,
                10.,
                25.,
                25.,
                Color::from_rgba(255, 255, 255, 255),
            );
        }
        next_frame().await;
        frame += 1;
    }
}
