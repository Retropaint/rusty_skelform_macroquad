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
use std::io::Read;

/// Load a SkelForm armature.
/// The file to load is the zip that is provided by SkelForm export.
pub fn load(zip_path: &str) -> (Armature, Vec<Texture2D>) {
    let file = std::fs::File::open(zip_path).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let mut armature_json = String::new();
    zip.by_name("armature.json")
        .unwrap()
        .read_to_string(&mut armature_json)
        .unwrap();

    let armature: Armature = serde_json::from_str(&armature_json).unwrap();

    let mut texes = vec![];

    for atlas in &armature.atlases {
        let mut img = vec![];
        zip.by_name(&atlas.filename.to_string())
            .unwrap()
            .read_to_end(&mut img)
            .unwrap();
        texes.push(Texture2D::from_file_with_format(
            &img,
            Some(ImageFormat::Png),
        ));
    }

    (armature.clone(), texes)
}

#[derive(PartialEq)]
pub struct ConstructOptions {
    /// Offset (additively) all bones' position by this amount.
    pub position: macroquad::prelude::Vec2,

    pub scale: macroquad::prelude::Vec2,
    pub velocity: macroquad::prelude::Vec2,
}

impl Default for ConstructOptions {
    fn default() -> Self {
        ConstructOptions {
            position: macroquad::prelude::Vec2::new(0., 0.),
            scale: macroquad::prelude::Vec2::new(1., 1.),
            velocity: macroquad::prelude::Vec2::new(0., 0.),
        }
    }
}

/// Process bones to be used for animation(s).
pub fn animate(
    bones: &mut Vec<Bone>,
    inverse_kinematics: &mut Vec<InverseKinematics>,
    visuals: &mut Vec<Visuals>,
    animations: &Vec<&Animation>,
    frames: &Vec<u32>,
    smooth_frames: &Vec<u32>,
) {
    rusty_skelform::animate(
        bones,
        inverse_kinematics,
        visuals,
        animations,
        frames,
        smooth_frames,
    );
}

pub fn construct(armature: &mut Armature, options: &ConstructOptions) {
    rusty_skelform::construct(armature);
    for b in 0..armature.constructed_bones.len() {
        let bone = &mut armature.constructed_bones[b];
        bone.pos.y = -bone.pos.y;
        bone.rot = -bone.rot;
        let options_scale = rusty_skelform::Vec2::new(options.scale.x, options.scale.y);
        bone.scale *= options_scale;
        bone.pos *= rusty_skelform::Vec2::new(options.scale.x, options.scale.y);
        bone.pos += rusty_skelform::Vec2::new(options.position.x, options.position.y);

        // apply velocity, for physics
        if let Some(physics) = armature.physics.get_mut(bone.physics_id as usize) {
            physics.global_pos -= rusty_skelform::Vec2::new(options.velocity.x, options.velocity.y);
        }

        rusty_skelform::check_flip(bone, options_scale);

        if let Some(visual) = armature.visuals.get_mut(bone.visuals_id as usize) {
            for vert in &mut visual.vertices {
                vert.pos.y = -vert.pos.y;
                vert.pos *= rusty_skelform::Vec2::new(options.scale.x, options.scale.y);
                vert.pos += rusty_skelform::Vec2::new(options.position.x, options.position.y);
            }
        }
    }
}

/// Draw the provided bones with Macroquad.
pub fn draw(
    bones: &mut Vec<Bone>,
    visuals: &Vec<Visuals>,
    texes: &Vec<Texture2D>,
    styles: &Vec<&Style>,
) {
    // bones with higher zindex should render first
    bones.sort_by(|a, b| {
        // get A zindex
        let a_zindex = if a.visuals_id != -1 {
            visuals[a.visuals_id as usize].zindex
        } else {
            0
        };

        // get B zindex
        let b_zindex = if b.visuals_id != -1 {
            visuals[b.visuals_id as usize].zindex
        } else {
            0
        };

        a_zindex.partial_cmp(&b_zindex).unwrap()
    });

    let col = Color::from_rgba(255, 255, 255, 255);
    for bone in bones {
        if bone.visuals_id == -1 {
            continue;
        }

        // get this bone's visual data
        let visual = &visuals[bone.visuals_id as usize];

        // get this bone's texture (based on active styles)
        let tex = get_bone_texture(visual.tex.clone(), styles);
        if tex == None {
            continue;
        }
        let tex = tex.unwrap();

        // render bone as mesh
        if visual.vertices.len() > 0 {
            let atlas_idx = tex.atlas_idx as usize;
            draw_mesh(&create_mesh(&visual, &tex, &texes[atlas_idx]));
            continue;
        }

        // Macroquad's sprite origin is top-left, so this will align them to center origin
        let push_center = tex.size / 2. * bone.scale;

        // render bone as regular rect
        draw_texture_ex(
            &texes[tex.atlas_idx as usize],
            bone.pos.x - push_center.x,
            bone.pos.y - push_center.y,
            col,
            DrawTextureParams {
                source: Some(Rect {
                    x: tex.offset.x,
                    y: tex.offset.y,
                    w: tex.size.x,
                    h: tex.size.y,
                }),
                dest_size: Some(macroquad::prelude::Vec2::new(
                    tex.size.x * bone.scale.x,
                    tex.size.y * bone.scale.y,
                )),
                rotation: bone.rot,
                ..Default::default()
            },
        );
    }
}

/// Create Macroquad meshes from the given bones and texture data.
fn create_mesh(visual: &Visuals, tex: &Texture, tex2d: &Texture2D) -> Mesh {
    let mut mesh = Mesh {
        vertices: vec![],
        indices: vec![],
        texture: Some(tex2d.clone()),
    };

    mesh.indices = visual.indices.iter().map(|i| *i as u16).collect();

    let lt_tex_x = tex.offset.x / tex2d.size().x;
    let lt_tex_y = tex.offset.y / tex2d.size().y;
    let rb_tex_x = (tex.offset.x + tex.size.x) / tex2d.size().x - lt_tex_x;
    let rb_tex_y = (tex.offset.y + tex.size.y) / tex2d.size().y - lt_tex_y;

    for v in &visual.vertices {
        let uv_x = lt_tex_x + (rb_tex_x * v.uv.x);
        let uv_y = lt_tex_y + (rb_tex_y * v.uv.y);

        let white = macroquad::color::WHITE;
        mesh.vertices.push(macroquad::models::Vertex::new(
            v.pos.x, v.pos.y, 0., uv_x, uv_y, white,
        ));
    }

    mesh
}
