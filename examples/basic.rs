use std::time::Instant;

use macroquad::prelude as mqr;
use mqr::*;
use rusty_skelform as skf;
use rusty_skelform_macroquad as skf_mq;
use skf::time_frame;

pub const ARMATURE_NIL: &str = "Armature not found! Please run this in the 'examples' folder.";
pub const INSTRUCTIONS: &str = "This is running in a game!";

#[macroquad::main("SkelForm - Macroquad Basic Demo")]
async fn main() {
    let armature_filename = "_skellina.skf";
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
    let mut anim_idx: usize = 2;
    let mut grounded = false;
    let mut coat = 0;
    let mut pants = 0;

    loop {
        clear_background(GRAY);

        if pos.y < ground_y {
            //vel.y += 0.05;
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
            anim_idx = 0;
        }
        if is_key_down(KeyCode::S) {
            pos.y += speed;
            dir = 1.;
        }
        if is_key_down(KeyCode::W) {
            pos.y -= speed;
            dir = -1.;
        }

        if is_key_pressed(KeyCode::Space) && grounded {
            vel.y = -5.;
            pos.y = ground_y - 1.;
        }
        if is_key_pressed(KeyCode::Key1) {
            coat = if coat == 0 { 1 } else { 0 };
        }
        if is_key_pressed(KeyCode::Key2) {
            pants = if pants == 0 { 2 } else { 0 };
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

        #[rustfmt::skip]
        draw_skellington(time, &mut skellington, anim_idx, pos, prev_pos, dir, &skel_texes, skel_scale, coat);

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
    coat: usize,
) {
    // process animation(s)
    let tf0 = time_frame(time, &armature.animations[0], false, true);

    skf_mq::animate(
        &mut armature.bones,
        &mut armature.inverse_kinematics,
        &mut armature.visuals,
        &vec![&armature.animations[anim_idx]],
        &vec![tf0],
        &vec![20],
    );

    // these will be used later for immutable edits before construction
    let bones = &mut armature.bones;
    let constructed_bones = &mut armature.constructed_bones;

    // move shoulder and head targets to mouse
    let mouse = skf::Vec2::new(
        mouse_position().0 / skel_scale * dir,
        -mouse_position().1 / skel_scale,
    );
    //bones[0].pos = skf::Vec2::new(-pos.x / skel_scale * dir, pos.y / skel_scale) + mouse;
    //bones[4].pos = skf::Vec2::new(-pos.x / skel_scale * dir, pos.y / skel_scale) + mouse;

    //if skel_style == 0 {
    //    bones.iter_mut().find(|b| b.name == "Hat").unwrap().pos += skf::Vec2::new(20., -60.);
    //    bones.iter_mut().find(|b| b.name == "Collar").unwrap().pos += skf::Vec2::new(7., -23.);
    //}
    //let skull = bones.iter_mut().find(|b| b.name == "Skull").unwrap();
    //skull.scale.y = 1.;

    //// flip skull and hat if looking the other way
    //let looking_back_left = dir == 1. && mouse_position().0 < pos.x;
    //let looking_back_right = dir != 1. && mouse_position().0 > pos.x;
    //if looking_back_left || looking_back_right {
    //    skull.scale.y = -skull.scale.y;
    //    let hat = bones.iter_mut().find(|b| b.name == "Hat").unwrap();
    //    hat.rot = -0.1;
    //}

    //// revert left shoulder constraint if looking back
    //if constructed_bones.len() != 0 {
    //    let shoulder = constructed_bones
    //        .iter_mut()
    //        .find(|b| b.name == "LSIK")
    //        .unwrap();
    //    if looking_back_left || looking_back_right {
    //        shoulder.ik_constraint = "Clockwise".to_string();
    //    } else {
    //        shoulder.ik_constraint = "CounterClockwise".to_string();
    //    }
    //}

    // construct and draw armature
    let velocity = Vec2::new((pos.x - prev_pos.x) * dir, -(pos.y - prev_pos.y)) * 10.;
    let skel_options = skf_mq::ConstructOptions {
        scale: mqr::Vec2::new(0.35, 0.35),
        position: Vec2::new(pos.x, pos.y),
        velocity,
        ..Default::default()
    };
    skf_mq::construct(armature, &skel_options);

    let styles = &vec![
        &armature.styles[2],
    ];
    skf_mq::draw(
        &mut armature.constructed_bones,
        &armature.visuals,
        &texes,
        styles,
    );
}
