//! SkelForm runtime for [Macroquad](https://macroquad.rs/).
//! # Usage
//! ```rust
//! use macroquad::prelude::*;
//! use rusty_skelform_macroquad::{animate, load_skelform_armature};
//!
//! #[macroquad::main("Demo")]
//! async fn main() {
//!     // Load SkelForm armature.
//!     let (armature, tex) = load_skelform_armature("path_to_export", 0);
//!
//!     // Start a timer to use for the animation.
//!     let time = std::time::Instant::now();
//!
//!     loop {
//!         // Play first animation.
//!         animate(&armature, &tex, 0, Some(time), true, true, None);
//!
//!         next_frame().await;
//!     }
//! }
//! ````
//! Note that [`animate()`] may have different parameters as of this publishing.

use macroquad::prelude::*;
use rusty_skelform::*;
use std::{collections::HashMap, io::Read, time::Instant};

/// Load a SkelForm armature.
/// The file to load is the zip that is provided by SkelForm export.
pub fn load_skelform_armature(zip_path: &str) -> (Armature, Texture2D) {
    // return an empty armature and texture if file doesn't exist
    if !std::fs::exists(zip_path).unwrap() {
        return (Armature::default(), Texture2D::empty());
    }

    let file = std::fs::File::open(zip_path).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let mut armature_json = String::new();
    zip.by_name("armature.json")
        .unwrap()
        .read_to_string(&mut armature_json)
        .unwrap();

    let root: SkelformRoot = serde_json::from_str(&armature_json).unwrap();

    let mut tex = Texture2D::empty();

    // import texture (if it makes sense to)
    if root.texture_size.x != 0. && root.texture_size.y != 0. {
        let mut img = vec![];
        zip.by_name("textures.png")
            .unwrap()
            .read_to_end(&mut img)
            .unwrap();
        tex = Texture2D::from_file_with_format(&img, Some(ImageFormat::Png));
    }

    (root.armature.clone(), tex)
}

/// Load a SkelForm armature, but pointing to armature and texture data separately.
/// Only used for debugging. The proper way to load armatures is via `load_skelform_armature`.
pub fn load_skelform_scattered(
    armature_path: &str,
    texture_path: &str,
    armature_idx: usize,
) -> (Armature, Texture2D) {
    let file = std::fs::File::open(armature_path).unwrap();
    let root: SkelformRoot = serde_json::from_reader(&file).unwrap();

    let mut tex = Texture2D::empty();

    // import texture (if it makes sense to)
    if root.texture_size.x != 0. && root.texture_size.y != 0. {
        tex =
            Texture2D::from_file_with_format(std::fs::read(texture_path).unwrap().as_slice(), None);
    }

    (root.armature.clone(), tex)
}

#[derive(PartialEq)]
pub struct AnimOptions {
    /// Animation playback speed (default 1).
    pub speed: f32,

    /// Offset (additively) all props' position by this amount.
    pub pos_offset: macroquad::prelude::Vec2,

    pub scale_factor: f32,

    pub frame: Option<i32>,

    pub last_anim_idx: usize,
    pub last_anim_frame: i32,
}

impl Default for AnimOptions {
    fn default() -> Self {
        AnimOptions {
            speed: 1.,
            pos_offset: macroquad::prelude::Vec2::new(0., 0.),
            scale_factor: 0.25,
            frame: None,
            last_anim_idx: usize::MAX,
            last_anim_frame: 0,
        }
    }
}

/// Run an animation and return the props to be (optionally) used.
///
/// `should_render` - Render the animation immediately with the most sensible stock settings (affected by AnimOptions).
/// `should_loop` - Simulate looping. If the animation is 10 frames and the supplied frame is 11, the resulting frame is 1.
///
/// Notable options:
/// `scale_factor` - Multiply scales by a factor of this.
/// `frame` - Render only this particular frame.
/// `last_anim_idx` - Index of the last animation that was played. Used for blending.
/// `last_anim_frame` - The frame of the last animation to blend from. Set to -1 for last frame.
///
/// Note: edits to the armature (head following cursor, etc) should be made *before* calling `animate()`, unless processing the props manually.
pub fn animate(
    armature: &mut Armature,
    texture: &Texture2D,
    animation_index: usize,
    time: Option<Instant>,
    should_loop: bool,
    should_render: bool,
    mut options: Option<AnimOptions>,
) -> (Vec<Bone>, i32) {
    if options == None {
        options = Some(AnimOptions::default());
    }

    // default to first frame, if neither it nor time were provided
    if time == None && options.as_ref().unwrap().frame == None {
        options.as_mut().unwrap().frame = Some(0);
    }

    let mut new_armature = armature.clone();

    // fix Macroquad-specific quirks
    {
        for bone in &mut new_armature.bones {
            bone.pos.y = -bone.pos.y;
            bone.rot = -bone.rot;
        }

        // reverse rotations
        for anim in &mut new_armature.animations {
            for kf in &mut anim.keyframes {
                if kf.element == AnimElement::Rotation {
                    kf.value = -kf.value;
                }
            }
        }
    }

    let mut props = new_armature.bones.clone();
    let mut frame = 0;

    if armature.animations.len() != 0 && animation_index < armature.animations.len() - 1 {
        let anim = &mut new_armature.animations[animation_index];
        if options.as_ref().unwrap().frame == None {
            frame = get_frame_by_time(anim, time.unwrap(), options.as_ref().unwrap().speed);
        } else if options.as_ref().unwrap().frame != None {
            frame = options.as_ref().unwrap().frame.unwrap();
        }

        props = rusty_skelform::animate(&mut new_armature, animation_index, frame, should_loop);
    }

    let mut og_props = props.clone();
    rusty_skelform::inheritance(&mut og_props, HashMap::new());
    let ik_rots = rusty_skelform::inverse_kinematics(&og_props, &armature.ik_families);
    rusty_skelform::inheritance(&mut props, ik_rots);

    for prop in &mut props {
        prop.scale *= options.as_ref().unwrap().scale_factor;
        prop.pos *= options.as_ref().unwrap().scale_factor;
        prop.pos.x += options.as_ref().unwrap().pos_offset.x;
        prop.pos.y += options.as_ref().unwrap().pos_offset.y;
    }

    if should_render {
        draw_props(&mut props, &new_armature, texture);
    }

    (props, frame)
}

/// Draw the provided props with Macroquad.
pub fn draw_props(props: &mut Vec<Bone>, armature: &Armature, tex: &Texture2D) {
    let col = Color::from_rgba(255, 255, 255, 255);
    for p in 0..props.len() {
        if props[p].style_idxs.len() == 0 {
            continue;
        }

        let prop_tex = &armature.styles[0].textures[props[p].tex_idx as usize];

        // render bone as mesh
        if props[p].vertices.len() > 0 {
            draw_mesh(&create_mesh(&props[p], prop_tex, tex));
            continue;
        }

        let push_center = prop_tex.size / 2. * props[p].scale;

        // render bone as regular rect
        draw_texture_ex(
            &tex,
            props[p].pos.x - push_center.x,
            props[p].pos.y - push_center.y,
            col,
            DrawTextureParams {
                source: Some(Rect {
                    x: prop_tex.offset.x,
                    y: prop_tex.offset.y,
                    w: prop_tex.size.x,
                    h: prop_tex.size.y,
                }),
                dest_size: Some(macroquad::prelude::Vec2::new(
                    prop_tex.size.x * props[p].scale.x,
                    prop_tex.size.y * props[p].scale.y,
                )),
                rotation: props[p].rot,
                ..Default::default()
            },
        );
    }
}

/// Create Macroquad meshes from the given bones and texture data.
pub fn create_mesh(bone: &Bone, bone_tex: &Texture, tex2d: &Texture2D) -> Mesh {
    let mut mesh = Mesh {
        vertices: vec![],
        indices: vec![],
        texture: Some(tex2d.clone()),
    };

    for i in &bone.indices {
        mesh.indices.push(*i as u16);
    }

    for v in &bone.vertices {
        let lt_tex_x = bone_tex.offset.x / tex2d.size().x;
        let lt_tex_y = bone_tex.offset.y / tex2d.size().y;
        let rb_tex_x = (bone_tex.offset.x + bone_tex.size.x) / tex2d.size().x;
        let rb_tex_y = (bone_tex.offset.y + bone_tex.size.y) / tex2d.size().y;
        mesh.vertices.push(macroquad::models::Vertex::new(
            bone.pos.x + ((v.pos.x - bone_tex.size.x / 2.) * bone.scale.x / 2.),
            bone.pos.y + ((-v.pos.y - bone_tex.size.y / 2.) * bone.scale.y / 2.),
            0.,
            v.uv.x,
            v.uv.y,
            macroquad::color::WHITE,
        ));
    }

    mesh
}
