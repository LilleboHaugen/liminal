//! Walks a derived value through reflection only: prints each field's path, type and state
//! hash, changes one field by name, and registers the type.
//!
//! Run with `cargo run -p liminal_reflect --example inspect`.

use std::io::Write;

use liminal_reflect::hash::reflect_state_hash;
use liminal_reflect::registry::TypeRegistry;
use liminal_reflect::value::{Reflect, ReflectRef, Typed, validate_reflect};

#[derive(Reflect)]
enum Mood {
    Calm,
    Alarmed { level: u8 },
}

#[derive(Reflect)]
struct Boid {
    position: [f32; 2],
    speed: f32,
    mood: Mood,
    tags: Vec<String>,
    #[reflect(ignore)]
    frame_scratch: u64,
}

/// Writes one line per field of `value`, recursing into structs and enums.
fn walk(out: &mut impl Write, at: &str, value: &dyn Reflect) -> std::io::Result<()> {
    let info = value.reflect_type_info();
    writeln!(
        out,
        "{at:<16} {:<28} hash {:016x}",
        info.type_path,
        reflect_state_hash(value)
    )?;
    let fields = match value.reflect_ref() {
        ReflectRef::Struct(fields) => fields,
        ReflectRef::Enum(e) => e,
        ReflectRef::List(_) | ReflectRef::Primitive(_) => return Ok(()),
    };
    for index in 0..fields.field_count() {
        if let (Some(name), Some(field)) = (fields.field_name(index), fields.field_at(index)) {
            walk(out, &format!("{at}.{name}"), field)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut boid = Boid {
        position: [1.0, -2.5],
        speed: 3.0,
        mood: Mood::Alarmed { level: 2 },
        tags: vec!["leader".to_string()],
        frame_scratch: 0,
    };
    boid.frame_scratch += 1;
    validate_reflect(&boid)?;
    let mut out = std::io::stdout().lock();
    walk(&mut out, "boid", &boid)?;

    let value: &mut dyn Reflect = &mut boid;
    *value
        .field_mut("mood")?
        .field_mut("level")?
        .downcast_mut::<u8>()? = 9;
    writeln!(out, "after the write: {:016x}", reflect_state_hash(value))?;

    let mut registry = TypeRegistry::new();
    let id = registry.register::<Boid>()?;
    writeln!(out, "registered {} as {id}", Boid::type_path())?;
    boid.mood = Mood::Calm;
    writeln!(out, "calm: {:016x}", reflect_state_hash(&boid))?;
    Ok(())
}
