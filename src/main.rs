use eframe::egui;
use image::{ImageBuffer, Rgba};
use voronoice::{Point};

struct VoronoiApp {
    sites: Vec<Point>,
    img_buffer: Option<egui::ColorImage>,
    video_frames: Vec<egui::ColorImage>, // 생성 프레임 저장
    current_frame: usize,                // 현재 재생 중인 프레임
    playing: bool,                       // 재생 중 여부
    last_update: std::time::Instant,     // 마지막 프레임 시간
}

impl Default for VoronoiApp {
    fn default() -> Self {
        let mut sites = Vec::new();
        for _i in 1..=5 {
            sites.push(Point {
                x: rand::random_range(0.1..=0.9),
                y: rand::random_range(0.1..=0.9),
            })
        }
        Self {
            sites,
            img_buffer: None,
            video_frames: Vec::new(),
            current_frame: 0,
            playing: false,
            last_update: std::time::Instant::now(),
        }
    }
}

impl eframe::App for VoronoiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Voronoi Diagram Viewer");

            if ui.button("Render").clicked() {
                let mut sites = Vec::new();
                for _i in 1..=5 {
                    sites.push(Point {
                        x: rand::random_range(0.1..=0.9),
                        y: rand::random_range(0.1..=0.9),
                    })
                }
                self.sites = sites;
                self.img_buffer = Some(render_voronoi(&self.sites, 400, 400));
                self.playing = false;
            }

            if ui.button("Play Video").clicked() {
                self.video_frames = video_voronoi2(&self.sites, 400, 400);
                self.current_frame = 0;
                self.playing = true;
                self.last_update = std::time::Instant::now();
            }
        });

        if self.playing && !self.video_frames.is_empty() {
            let now = std::time::Instant::now();
            if now.duration_since(self.last_update).as_millis() >= 5 {
                // 20fps
                self.img_buffer = Some(self.video_frames[self.current_frame].clone());
                self.current_frame += 1;
                if self.current_frame >= self.video_frames.len() {
                    self.current_frame = 0;
                    self.playing = false;
                }
                self.last_update = now;
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(color_img) = &self.img_buffer {
                let texture = ui.ctx().load_texture(
                    "voronoi_tex",
                    color_img.clone(),
                    egui::TextureOptions::NEAREST,
                );
                ui.image(&texture);
            }
        });
    }
}

fn render_voronoi(sites: &[Point], width: usize, height: usize) -> egui::ColorImage {
    // 보로노이 다이어그램 계산
    // bounding box은 (0,0) ~ (1,1)로
    // let bbox = BoundingBox::new(Point { x: 0.0, y: 0.0 }, 1.0, 1.0);

    // 이미지 버퍼 생성
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width as u32, height as u32);

    // (픽셀마다 가장 가까운 사이트 색을 균등하게 배분
    for y in 0..height {
        for x in 0..width {
            let fx = x as f64 / (width as f64);
            let fy = y as f64 / (height as f64);
            let p = Point { x: fx, y: fy };

            let mut best_idx = 0;
            let mut best_dist = f64::MAX;
            for (i, site) in sites.iter().enumerate() {
                let dx = site.x - p.x;
                let dy = site.y - p.y;
                let d2 = dx * dx + dy * dy;
                if d2 < best_dist {
                    best_dist = d2;
                    best_idx = i;
                }
            }

            let color = {
                let palette = [
                    [255, 128, 128],
                    [128, 255, 128],
                    [128, 128, 255],
                    [255, 255, 128],
                    [255, 128, 255],
                    [128, 255, 255],
                ];
                let c = palette[best_idx % palette.len()];
                Rgba([c[0], c[1], c[2], 255])
            };

            img.put_pixel(x as u32, y as u32, color);
        }
    }

    // 사이트 위치 점 찍기
    for site in sites {
        let px = (site.x * width as f64) as u32;
        let py = (site.y * height as f64) as u32;
        if px < width as u32 && py < height as u32 {
            img.put_pixel(px, py, Rgba([0, 0, 0, 255]));
            img.put_pixel(px - 1, py, Rgba([0, 0, 0, 255]));
            img.put_pixel(px + 1, py, Rgba([0, 0, 0, 255]));
            img.put_pixel(px, py - 1, Rgba([0, 0, 0, 255]));
            img.put_pixel(px, py + 1, Rgba([0, 0, 0, 255]));
        }
    }

    //ImageBuffer to egui::ColorImage
    let mut pixels = Vec::with_capacity(width * height * 4);
    for (_x, _y, pixel) in img.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        pixels.push(r);
        pixels.push(g);
        pixels.push(b);
        pixels.push(a);
    }
    egui::ColorImage::from_rgba_unmultiplied([width, height], &pixels)
}

fn video_voronoi2(sites: &[Point], width: usize, height: usize) -> Vec<egui::ColorImage> {
    let mut fps: Vec<egui::ColorImage> = Vec::new();

    // 이미지 초기화
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(
        width as u32,
        height as u32,
        Rgba([255, 255, 255, 255]),
    );

    // 이미지 대각선 길이
    let max_radius = ((width * width + height * height) as f64).sqrt() / 2.0;

    // 반경 단계
    let step = 2.0;

    for r in (0..(max_radius as usize)).step_by(step as usize) {
        let r_f = r as f64 / (width.max(height) as f64); // 0~1 범위로 매핑

        for y in 0..height {
            for x in 0..width {
                let fx = x as f64 / width as f64;
                let fy = y as f64 / height as f64;
                let p = Point { x: fx, y: fy };

                // 이미 칠해졌으면 스킵
                let pixel = img.get_pixel(x as u32, y as u32);
                if pixel.0 != [255, 255, 255, 255] {
                    continue;
                }

                // 가장 가까운 사이트 찾기
                let mut best_idx = 0;
                let mut best_dist = f64::MAX;
                for (i, site) in sites.iter().enumerate() {
                    let dx = site.x - p.x;
                    let dy = site.y - p.y;
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < best_dist {
                        best_dist = d;
                        best_idx = i;
                    }
                }

                // 현재 반경 안에 있으면 칠하기
                if best_dist <= r_f {
                    let palette = [
                        [255, 128, 128],
                        [128, 255, 128],
                        [128, 128, 255],
                        [255, 255, 128],
                        [255, 128, 255],
                        [128, 255, 255],
                    ];
                    let c = palette[best_idx % palette.len()];
                    img.put_pixel(x as u32, y as u32, Rgba([c[0], c[1], c[2], 255]));
                }
            }
        }

        // 현재 상태를 전체 프레임의 1/2 프레임으로 저장
        if r % 2 == 0 {
            let mut pixels = Vec::with_capacity(width * height * 4);
            for site in sites {
                let px = (site.x * width as f64) as u32;
                let py = (site.y * height as f64) as u32;
                if px < width as u32 && py < height as u32 {
                    img.put_pixel(px, py, Rgba([0, 0, 0, 255]));
                    img.put_pixel(px - 1, py, Rgba([0, 0, 0, 255]));
                    img.put_pixel(px + 1, py, Rgba([0, 0, 0, 255]));
                    img.put_pixel(px, py - 1, Rgba([0, 0, 0, 255]));
                    img.put_pixel(px, py + 1, Rgba([0, 0, 0, 255]));
                }
            }
            for (_x, _y, pixel) in img.enumerate_pixels() {
                let [r, g, b, a] = pixel.0;
                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(a);
            }
            fps.push(egui::ColorImage::from_rgba_unmultiplied(
                [width, height],
                &pixels,
            ));
        }
    }

    fps
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Voronoi GUI",
        options,
        Box::new(|_cc| Ok(Box::new(VoronoiApp::default()))),
    )
}
