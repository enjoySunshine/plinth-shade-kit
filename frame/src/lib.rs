use channel::Submission;

#[derive(Clone, Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub color: Vec<[u8; 4]>,
    pub depth: Vec<f32>,
}

impl Image {
    pub fn cleared(width: u32, height: u32, color: [u8; 4], depth: f32) -> Self {
        let n = (width as usize) * (height as usize);
        Self {
            width,
            height,
            color: vec![color; n],
            depth: vec![depth; n],
        }
    }

    pub fn write_ppm(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut text = format!("P3\n{} {}\n255\n", self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let p = self.color[(y * self.width + x) as usize];
                text.push_str(&format!("{} {} {} {}\n", p[0], p[1], p[2], p[3]));
            }
        }
        std::fs::write(path, text)
    }

    pub fn write_depth(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut text = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let z = self.depth[(y * self.width + x) as usize];
                text.push_str(&format!("{z}\n"));
            }
        }
        std::fs::write(path, text)
    }
}

pub fn read(submission: &Submission) -> Image {
    Image::cleared(
        submission.width(),
        submission.height(),
        submission.clear_color(),
        submission.clear_depth(),
    )
}
