use std::time::Instant;

use macroquad::prelude as mqr;
use mqr::*;
use rusty_skelform as skf;
use rusty_skelform_macroquad as skf_mq;
use skf::time_frame;

pub const ARMATURE_NIL: &str = "Armature not found! Please run this in the 'examples' folder.";
pub const INSTRUCTIONS: &str =
    "A - Move Left\nD - Move Right\nSpace - Jump\nSkellington will look at and reach for cursor";

#[macroquad::main("SkelForm - Macroquad Basic Demo")]
async fn main() {
    // load SkelForm armature
    let armature_filename = "skellington.skf";
    let (mut skellington, skel_texes) = skf_mq::load(armature_filename);
    let (mut skellina, skela_texes) = skf_mq::load("skellina.skf");

    // timer for animations
    let mut time = Instant::now();
    let skela_time = Instant::now();

    if skellington.bones.len() == 0 {
        println!("{}", ARMATURE_NIL.to_string());
    }

    let mut dir = 1.;
    let mut pos = Vec2::new(100., -100.);
    let mut vel = Vec2::new(0., 0.);
    let ground_y = screen_height() / 2. + 100.;
    let mut last_anim_idx = 0;
    let mut anim_idx: usize;
    let mut grounded = false;

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
            dir,
            &skel_texes,
            skel_scale,
        );

        draw_skellina(
            skela_time,
            &mut skellina,
            skel_scale,
            &skela_texes,
            ground_y,
        );

        let white = Color::from_rgba(255, 255, 255, 255);
        if skellington.bones.len() == 0 {
            draw_text(ARMATURE_NIL, 10., 25., 25., white);
        } else {
            draw_multiline_text(INSTRUCTIONS, 10., 25., 35., None, white);
        }

        next_frame().await;
    }
}

fn draw_skellington(
    time: std::time::Instant,
    armature: &mut skf::Armature,
    anim_idx: usize,
    pos: Vec2,
    dir: f32,
    texes: &Vec<mqr::Texture2D>,
    skel_scale: f32,
) {
    // process animation(s)
    let tf0 = time_frame(time, &armature.animations[anim_idx], false, true);
    let skel_options = skf_mq::ConstructOptions {
        speed: 1.,
        scale: mqr::Vec2::new(skel_scale * dir, skel_scale),
        position: Vec2::new(pos.x, pos.y),
        ..Default::default()
    };
    skf_mq::animate(
        &mut armature.bones,
        &vec![&armature.animations[anim_idx]],
        &vec![tf0],
        &vec![20],
    );

    // these will be used later for immutable edits before construction
    let mut armature_c = armature.clone();
    let bones = &mut armature_c.bones;

    // move shoulder and head targets to mouse
    let mouse = skf::Vec2::new(
        mouse_position().0 / skel_scale * dir,
        -mouse_position().1 / skel_scale,
    );
    bones[0].pos = skf::Vec2::new(-pos.x / skel_scale * dir, pos.y / skel_scale) + mouse;
    bones[4].pos = skf::Vec2::new(-pos.x / skel_scale * dir, pos.y / skel_scale) + mouse;

    // flip skull and hat if looking the other way
    if (dir == 1. && mouse_position().0 < pos.x) || (dir != 1. && mouse_position().0 > pos.x) {
        let skull = bones.iter_mut().find(|b| b.name == "Skull").unwrap();
        skull.scale.y = -skull.scale.y;
        let hat = bones.iter_mut().find(|b| b.name == "Hat").unwrap();
        hat.rot = -hat.rot;
        let shoulder = bones.iter_mut().find(|b| b.name == "LSIK").unwrap();
        shoulder.ik_constraint = 1;
    }

    // construct and draw armature
    let mut constructed_bones = skf_mq::construct(&armature_c, skel_options);
    skf_mq::draw(&mut constructed_bones, &texes, &vec![&armature_c.styles[0]]);
}

fn draw_skellina(
    time: std::time::Instant,
    skellina: &mut skf::Armature,
    scale: f32,
    texes: &Vec<mqr::Texture2D>,
    ground_y: f32,
) {
    let tf0 = time_frame(time, &skellina.animations[0], false, true);
    skf_mq::animate(
        &mut skellina.bones,
        &vec![&skellina.animations[0]],
        &vec![tf0],
        &vec![0],
    );
    let mut bones = skf_mq::construct(
        &skellina,
        skf_mq::ConstructOptions {
            scale: mqr::Vec2::new(-scale, scale),
            position: mqr::Vec2::new(screen_width() - 100., ground_y + 50.),
            ..Default::default()
        },
    );
    skf_mq::draw(&mut bones, &texes, &vec![&skellina.styles[0]]);
}
