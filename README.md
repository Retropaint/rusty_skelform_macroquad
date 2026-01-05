Library for running [SkelForm](https://skelform.org) animations in
[Macroquad](https://macroquad.rs/).

## Example

A basic character example is included in `/examples`.

in the `/examples` folder:

```
cargo run --example basic
```

## Basic Setup

```
use rusty_skelform_macroquad as skf_mq;
```

- `skf_mq::load()` - loads `.skf` file and returns armature & textures, to be
  used later
- `skf_mq::animate()` - transforms the armature's bones based on the
  animation(s)
- `skf_mq::construct()` - provides the bones from this armature that are ready
  for use
- `skf_mq::draw()` - draws the bones on-screen, with the provided style(s)

### 1. Load:

```
let (mut armature, textures) = skf_mq::load("armature.skf")
```

This should only be called once (eg; before main game loop), and `armature` and
`textures` should be kept for later use.

### 2\. Animate:

```
# use `skf_mq.time_frame()` to get the animation frame based on time (1000 = 1 second)
time: std::time::Instant = std::time::Instant::now();
let time_frame = skf_mq::time_frame(time, &armature.animations[0], false, true);

skf_mq::animate(
    &mut armature.bones,
    &vec![&armature.animations[0]],
    &vec![time_frame],
    &vec![0],
);
```

_Note: not needed if armature is statilc_

### 3\. Construct:

```
let options = skf_mq::ConstructOptions {
  position: Vec2::new(screen_width()/2, screen_height()/2),
  ..Default::default()
};

let mut final_bones = skf_mq::construct(&armature, options);
```

Modifications to the armature (eg; aiming at cursor) may be done before or after
construction.

### 4\. Draw:

```
skf_mq::draw(
    &mut final_bones,
    &textures,
    &vec![&armature.styles[0]],
);
```
