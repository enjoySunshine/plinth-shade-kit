use channel::{Blend, Channel};
use frame::read;
use std::path::PathBuf;
use std::time::Instant;

fn pack(x: f32, y: f32, z: f32, w: f32, color: [u8; 4]) -> [u8; 20] {
    let mut bytes = [0u8; 20];
    bytes[0..4].copy_from_slice(&x.to_le_bytes());
    bytes[4..8].copy_from_slice(&y.to_le_bytes());
    bytes[8..12].copy_from_slice(&z.to_le_bytes());
    bytes[12..16].copy_from_slice(&w.to_le_bytes());
    bytes[16..20].copy_from_slice(&color);
    bytes
}

fn tri(verts: [((f32, f32, f32, f32), [u8; 4]); 3]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(60);
    for ((x, y, z, w), color) in verts {
        bytes.extend_from_slice(&pack(x, y, z, w, color));
    }
    bytes
}

fn full(z: f32, color: [u8; 4]) -> (Vec<u8>, Vec<u8>) {
    let a = tri([
        ((-1.0, 1.0, z, 1.0), color),
        ((1.0, 1.0, z, 1.0), color),
        ((-1.0, -1.0, z, 1.0), color),
    ]);
    let b = tri([
        ((1.0, 1.0, z, 1.0), color),
        ((1.0, -1.0, z, 1.0), color),
        ((-1.0, -1.0, z, 1.0), color),
    ]);
    (a, b)
}

fn base() -> channel::Submission {
    let mut ch = Channel::new(8, 8);
    ch.clear([0, 0, 0, 0], 1.0);
    ch.set_viewport(0.0, 0.0, 8.0, 8.0, 0.0, 1.0);
    ch.set_scissor(0, 0, 8, 8);
    ch.set_cull_back(false);
    ch.set_depth(true, true);
    ch.set_blend(Blend::Add);
    let red = [10, 0, 0, 10];
    ch.draw(&tri([
        ((-1.0, 1.0, 0.5, 1.0), red),
        ((1.0, 1.0, 0.5, 1.0), red),
        ((-1.0, -1.0, 0.5, 1.0), red),
    ]));
    let second = tri([
        ((1.0, 1.0, 0.5, 1.0), red),
        ((1.0, -1.0, 0.5, 1.0), red),
        ((-1.0, -1.0, 0.5, 1.0), red),
    ]);
    ch.draw_indexed(&second, &[0, 1, 2]);
    ch.draw(&tri([
        ((-1.0, 1.0, 0.25, 1.0), red),
        ((-0.6875, 1.0, 0.25, 1.0), red),
        ((-1.0, 0.6875, 0.25, 1.0), red),
    ]));
    ch.submit()
}

fn colored() -> channel::Submission {
    let mut ch = Channel::new(8, 8);
    ch.clear([0, 0, 0, 0], 1.0);
    ch.set_viewport(0.0, 0.0, 8.0, 8.0, 0.0, 1.0);
    ch.set_scissor(0, 0, 8, 8);
    ch.set_cull_back(false);
    ch.set_depth(true, true);
    ch.set_blend(Blend::Replace);
    ch.draw(&tri([
        ((-0.75, 0.75, 0.5, 1.0), [200, 0, 0, 255]),
        ((0.75, 0.75, 0.5, 1.0), [0, 200, 0, 255]),
        ((0.0, -0.5, 0.5, 1.0), [0, 0, 200, 255]),
    ]));
    ch.draw(&tri([
        ((-0.5, 0.5, 0.25, 1.0), [255, 255, 255, 255]),
        ((0.5, 0.5, 0.25, 1.0), [255, 255, 255, 255]),
        ((0.0, -0.25, 0.25, 1.0), [255, 255, 255, 255]),
    ]));
    let (a, b) = full(0.75, [255, 0, 0, 255]);
    ch.draw(&a);
    ch.draw(&b);
    ch.submit()
}

fn clipped() -> channel::Submission {
    let mut ch = Channel::new(8, 8);
    ch.clear([0, 0, 0, 0], 1.0);
    ch.set_viewport(0.0, 0.0, 8.0, 8.0, 0.0, 1.0);
    ch.set_scissor(0, 0, 4, 8);
    ch.set_cull_back(false);
    ch.set_depth(true, true);
    ch.set_blend(Blend::Replace);
    let (a, b) = full(0.5, [200, 0, 0, 255]);
    ch.draw(&a);
    ch.draw(&b);
    let (c, d) = full(0.25, [0, 200, 0, 255]);
    ch.draw(&c);
    ch.draw(&d);
    ch.submit()
}

fn short() -> channel::Submission {
    let mut ch = Channel::new(8, 8);
    ch.clear([0, 0, 0, 0], 1.0);
    ch.set_viewport(0.0, 0.0, 8.0, 8.0, 0.0, 1.0);
    ch.set_scissor(0, 0, 8, 8);
    ch.set_blend(Blend::Replace);
    ch.set_depth(true, true);
    let (a, b) = full(0.5, [200, 0, 0, 255]);
    ch.draw(&a);
    ch.draw(&b);
    ch.draw(&pack(0.0, 0.0, 0.25, 1.0, [0, 200, 0, 255]));
    ch.submit()
}

fn wide() -> channel::Submission {
    let mut ch = Channel::new(1024, 1024);
    ch.clear([0, 0, 0, 0], 1.0);
    ch.set_viewport(0.0, 0.0, 1024.0, 1024.0, 0.0, 1.0);
    ch.set_scissor(0, 0, 1024, 1024);
    ch.set_blend(Blend::Replace);
    ch.set_depth(true, true);
    for i in 0..200 {
        let col = (i % 16) as f32;
        let row = (i / 16) as f32;
        let x0 = -1.0 + col * (2.0 / 16.0);
        let y0 = 1.0 - row * (2.0 / 16.0);
        let x1 = x0 + (2.0 / 16.0) * 0.8;
        let y1 = y0 - (2.0 / 16.0) * 0.8;
        let z = 0.2 + (i as f32) * 0.003;
        let color = [(i * 3) as u8, (i * 5) as u8, 40, 255];
        ch.draw(&tri([
            ((x0, y0, z, 1.0), color),
            ((x1, y0, z, 1.0), color),
            ((x0, y1, z, 1.0), color),
        ]));
    }
    ch.submit()
}

fn dump(name: &str, submission: &channel::Submission) {
    let image = read(submission);
    let root = PathBuf::from("review-out");
    image.write_ppm(&root.join(format!("{name}.ppm"))).unwrap();
    image
        .write_depth(&root.join(format!("{name}.depth")))
        .unwrap();
    let p = image.color[0];
    println!(
        "{name} pixel0 {} {} {} {} depth {}",
        p[0], p[1], p[2], p[3], image.depth[0]
    );
}

fn main() {
    dump("base", &base());
    dump("color", &colored());
    dump("scissor", &clipped());
    dump("short", &short());
    let started = Instant::now();
    let submission = wide();
    let image = read(&submission);
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    image.write_ppm(std::path::Path::new("review-out/wide.ppm")).unwrap();
    image
        .write_depth(std::path::Path::new("review-out/wide.depth"))
        .unwrap();
    println!("wide {ms:.1} ms");
}
