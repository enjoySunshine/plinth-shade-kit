//! 道具通道：记下视口、裁剪、混合、深度和三角形，交给显卡的是同一份记录。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blend {
    Add,
    Replace,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scissor {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawRef {
    pub index: u32,
    pub vertex_bytes: usize,
    pub index_count: usize,
    pub viewport: Viewport,
    pub scissor: Scissor,
    pub blend: Blend,
    pub depth_less: bool,
    pub depth_write: bool,
    pub cull_back: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
    pub color: [u8; 4],
}

#[derive(Clone)]
struct DrawBytes {
    vertices: Vec<u8>,
    indices: Option<Vec<u16>>,
}

#[derive(Clone)]
enum Op {
    Viewport(Viewport),
    Scissor(Scissor),
    Blend(Blend),
    Depth { less: bool, write: bool },
    Cull(bool),
    Draw(DrawBytes),
}

/// 一次提交。审片和显卡记录都从这里取。
#[derive(Clone)]
pub struct Submission {
    width: u32,
    height: u32,
    clear_color: [u8; 4],
    clear_depth: f32,
    ops: Vec<Op>,
}

impl Submission {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn clear_color(&self) -> [u8; 4] {
        self.clear_color
    }

    pub fn clear_depth(&self) -> f32 {
        self.clear_depth
    }

    pub fn gpu(&self) -> GpuRecord {
        GpuRecord {
            bytes: encode(&self.ops),
        }
    }

    pub fn replay(&self) -> Vec<DrawRef> {
        let mut viewport = Viewport {
            x: 0.0,
            y: 0.0,
            width: self.width as f32,
            height: self.height as f32,
            near: 0.0,
            far: 1.0,
        };
        let mut scissor = Scissor {
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
        };
        let mut blend = Blend::Replace;
        let mut depth_less = true;
        let mut depth_write = true;
        let mut cull_back = false;
        let mut out = Vec::new();
        for op in &self.ops {
            match op {
                Op::Viewport(v) => viewport = *v,
                Op::Scissor(s) => scissor = *s,
                Op::Blend(b) => blend = *b,
                Op::Depth { less, write } => {
                    depth_less = *less;
                    depth_write = *write;
                }
                Op::Cull(on) => cull_back = *on,
                Op::Draw(draw) => {
                    out.push(DrawRef {
                        index: out.len() as u32,
                        vertex_bytes: draw.vertices.len(),
                        index_count: draw.indices.as_ref().map(|v| v.len()).unwrap_or(0),
                        viewport,
                        scissor,
                        blend,
                        depth_less,
                        depth_write,
                        cull_back,
                    });
                }
            }
        }
        out
    }

    pub fn decode(&self, _index: u32) -> Option<[Vertex; 3]> {
        None
    }
}

/// 交给显卡的那一份字节。
#[derive(Clone, PartialEq, Eq)]
pub struct GpuRecord {
    bytes: Vec<u8>,
}

impl GpuRecord {
    pub fn scissor_rects(&self) -> Vec<[u32; 4]> {
        let mut out = Vec::new();
        let b = &self.bytes;
        let mut i = 0;
        while i + 1 <= b.len() {
            match b[i] {
                2 => {
                    if i + 17 > b.len() {
                        break;
                    }
                    let x = u32::from_le_bytes(b[i + 1..i + 5].try_into().unwrap());
                    let y = u32::from_le_bytes(b[i + 5..i + 9].try_into().unwrap());
                    let w = u32::from_le_bytes(b[i + 9..i + 13].try_into().unwrap());
                    let h = u32::from_le_bytes(b[i + 13..i + 17].try_into().unwrap());
                    out.push([x, y, w, h]);
                    i += 17;
                }
                1 => i += 25,
                3 => i += 2,
                4 => i += 3,
                5 => i += 2,
                6 => {
                    if i + 9 > b.len() {
                        break;
                    }
                    let n = u32::from_le_bytes(b[i + 1..i + 5].try_into().unwrap()) as usize;
                    let m = u32::from_le_bytes(b[i + 5..i + 9].try_into().unwrap()) as usize;
                    i += 9 + n + m * 2;
                }
                _ => break,
            }
        }
        out
    }
}

fn encode(ops: &[Op]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for op in ops {
        match op {
            Op::Viewport(v) => {
                bytes.push(1);
                for n in [v.x, v.y, v.width, v.height, v.near, v.far] {
                    bytes.extend_from_slice(&n.to_le_bytes());
                }
            }
            Op::Scissor(s) => {
                bytes.push(2);
                for n in [s.x, s.y, s.width, s.height] {
                    bytes.extend_from_slice(&n.to_le_bytes());
                }
            }
            Op::Blend(b) => {
                bytes.push(3);
                bytes.push(match b {
                    Blend::Add => 1,
                    Blend::Replace => 2,
                });
            }
            Op::Depth { less, write } => {
                bytes.push(4);
                bytes.push(u8::from(*less));
                bytes.push(u8::from(*write));
            }
            Op::Cull(on) => {
                bytes.push(5);
                bytes.push(u8::from(*on));
            }
            Op::Draw(draw) => {
                bytes.push(6);
                bytes.extend_from_slice(&(draw.vertices.len() as u32).to_le_bytes());
                let ic = draw.indices.as_ref().map(|v| v.len()).unwrap_or(0) as u32;
                bytes.extend_from_slice(&ic.to_le_bytes());
                bytes.extend_from_slice(&draw.vertices);
                if let Some(idx) = &draw.indices {
                    for n in idx {
                        bytes.extend_from_slice(&n.to_le_bytes());
                    }
                }
            }
        }
    }
    bytes
}

#[derive(Clone)]
pub struct Channel {
    width: u32,
    height: u32,
    clear_color: [u8; 4],
    clear_depth: f32,
    ops: Vec<Op>,
}

impl Channel {
    pub fn new(width: u32, height: u32) -> Self {
        let mut channel = Self {
            width,
            height,
            clear_color: [0, 0, 0, 0],
            clear_depth: 1.0,
            ops: Vec::new(),
        };
        channel.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
        channel.set_scissor(0, 0, width, height);
        channel.set_blend(Blend::Replace);
        channel.set_depth(true, true);
        channel.set_cull_back(false);
        channel
    }

    pub fn clear(&mut self, color: [u8; 4], depth: f32) {
        self.clear_color = color;
        self.clear_depth = depth;
    }

    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32, near: f32, far: f32) {
        self.ops.push(Op::Viewport(Viewport {
            x,
            y,
            width,
            height,
            near,
            far,
        }));
    }

    pub fn set_scissor(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.ops.push(Op::Scissor(Scissor {
            x,
            y,
            width,
            height,
        }));
    }

    pub fn set_blend(&mut self, blend: Blend) {
        self.ops.push(Op::Blend(blend));
    }

    pub fn set_depth(&mut self, less: bool, write: bool) {
        self.ops.push(Op::Depth { less, write });
    }

    pub fn set_cull_back(&mut self, on: bool) {
        self.ops.push(Op::Cull(on));
    }

    pub fn draw(&mut self, vertices: &[u8]) {
        self.ops.push(Op::Draw(DrawBytes {
            vertices: vertices.to_vec(),
            indices: None,
        }));
    }

    pub fn draw_indexed(&mut self, vertices: &[u8], indices: &[u16]) {
        self.ops.push(Op::Draw(DrawBytes {
            vertices: vertices.to_vec(),
            indices: Some(indices.to_vec()),
        }));
    }

    pub fn submit(&self) -> Submission {
        Submission {
            width: self.width,
            height: self.height,
            clear_color: self.clear_color,
            clear_depth: self.clear_depth,
            ops: self.ops.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(x: f32, y: f32, z: f32, c: [u8; 4]) -> [u8; 20] {
        let mut b = [0u8; 20];
        b[0..4].copy_from_slice(&x.to_le_bytes());
        b[4..8].copy_from_slice(&y.to_le_bytes());
        b[8..12].copy_from_slice(&z.to_le_bytes());
        b[12..16].copy_from_slice(&1f32.to_le_bytes());
        b[16..20].copy_from_slice(&c);
        b
    }

    #[test]
    fn scissor_stays_on_the_gpu_record_until_rewritten() {
        let mut ch = Channel::new(8, 8);
        ch.set_scissor(0, 0, 4, 8);
        ch.draw(&solid(-1.0, 1.0, 0.5, [200, 0, 0, 255]));
        ch.draw(&solid(1.0, 1.0, 0.5, [0, 200, 0, 255]));
        let sub = ch.submit();
        let rects = sub.gpu().scissor_rects();
        assert_eq!(rects.last().copied(), Some([0, 0, 4, 8]));
        let draws = sub.replay();
        assert_eq!(draws.len(), 2);
        assert_eq!(draws[0].scissor, draws[1].scissor);
        assert_eq!(draws[1].blend, Blend::Replace);
    }

    #[test]
    fn blend_from_the_previous_draw_still_applies() {
        let mut ch = Channel::new(8, 8);
        ch.set_blend(Blend::Add);
        ch.set_depth(true, true);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&solid(-1.0, 1.0, 0.5, [10, 0, 0, 10]));
        bytes.extend_from_slice(&solid(1.0, 1.0, 0.5, [10, 0, 0, 10]));
        bytes.extend_from_slice(&solid(-1.0, -1.0, 0.5, [10, 0, 0, 10]));
        ch.draw(&bytes);
        ch.draw_indexed(&bytes, &[0, 1, 2]);
        let draws = ch.submit().replay();
        assert_eq!(draws[0].blend, Blend::Add);
        assert_eq!(draws[1].blend, Blend::Add);
        assert!(draws[1].depth_less && draws[1].depth_write);
        assert_eq!(draws[0].vertex_bytes, 60);
        assert_eq!(draws[1].index_count, 3);
    }

    #[test]
    fn short_draw_keeps_the_bytes_that_were_written() {
        let mut ch = Channel::new(8, 8);
        ch.draw(&solid(0.0, 0.0, 0.5, [0, 200, 0, 255]));
        let draws = ch.submit().replay();
        assert_eq!(draws[0].vertex_bytes, 20);
        assert_eq!(draws[0].index_count, 0);
    }

    #[test]
    fn changing_scissor_changes_the_gpu_bytes() {
        let mut wide = Channel::new(8, 8);
        wide.set_scissor(0, 0, 8, 8);
        wide.draw(&[0u8; 60]);
        let mut narrow = Channel::new(8, 8);
        narrow.set_scissor(0, 0, 4, 8);
        narrow.draw(&[0u8; 60]);
        assert_ne!(
            wide.submit().gpu().scissor_rects(),
            narrow.submit().gpu().scissor_rects()
        );
    }
}
