use eframe::egui;
use rand::prelude::*;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Window Title",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}

struct App {
    board: Board,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            board: Board::new(4, 3),
        }
    }
}

#[derive(Debug, Clone)]
struct Piece {
    state: usize,
    lim: usize,
}
impl Piece {
    fn default() -> Self {
        Self { state: 1, lim: 3 }
    }

    fn new(lim: usize) -> Self {
        Self { state: 1, lim }
    }

    fn change(&mut self, n: usize) {
        self.state = (self.state + n) % self.lim;
    }
}

#[derive(Debug, Clone)]
struct Board {
    pieces: Vec<Vec<Piece>>,
    size: usize,
}
impl Board {
    fn new(n: usize, lim: usize) -> Self {
        Board {
            pieces: vec![vec![Piece::new(lim); n]; n],
            size: n,
        }
    }

    fn press(&mut self, x: usize, y: usize) {
        let val = self.pieces[y][x].state;
        self.pieces[y][x].change(val);
        if y < self.size - 1 {
            self.pieces[y + 1][x].change(val);
        };
        if y > 0 {
            self.pieces[y - 1][x].change(val);
        };
        if x < self.size - 1 {
            self.pieces[y][x + 1].change(val);
        };
        if x > 0 {
            self.pieces[y][x - 1].change(val);
        };
    }

    fn random_move(&mut self, rng: &mut ThreadRng) {
        let x = (rng.gen::<f32>() * self.size as f32).floor() as usize;
        let y = (rng.gen::<f32>() * self.size as f32).floor() as usize;
        self.press(x, y);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let spectrum = colorous::RED_YELLOW_GREEN;
        egui::TopBottomPanel::top("Top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Puzzle", |ui| {
                    if ui.button("Scramble").clicked() {
                        let mut rng = thread_rng();
                        for _ in 0..100 {
                            self.board.random_move(&mut rng);
                        }
                    }
                    for n in 3..=6 {
                        for l in 1..=3 {
                            let n_str = n.to_string();
                            let lim = 2 * l + 1;
                            let lim_str = lim.to_string();
                            if ui
                                .button("".to_string() + &n_str + "x" + &n_str + ", " + &lim_str)
                                .clicked()
                            {
                                self.board = Board::new(n, lim);
                            }
                        }
                    }
                    if ui.button("3x3,3").clicked() {
                        self.board = Board::new(3, 3);
                    }
                });
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let (min, size) = (rect.left_top(), rect.size());
            let unit = size / self.board.size as f32;
            if ui.input(|input| input.pointer.primary_pressed()) {
                if ui.ui_contains_pointer() {
                    if let Some(mpos) = ctx.pointer_latest_pos() {
                        let pos = ((mpos - min) / unit).to_pos2();
                        self.board
                            .press(pos.x.trunc() as usize, pos.y.trunc() as usize);
                    }
                }
            }

            for (j, row) in self.board.pieces.iter().enumerate() {
                for (i, piece) in row.iter().enumerate() {
                    let col: egui::Color32;
                    if piece.state == 0 {
                        col = egui::Color32::WHITE;
                    } else {
                        let scol = spectrum.eval_rational(
                            (piece.lim - 3 + piece.state) % (piece.lim - 1),
                            piece.lim - 1,
                        );
                        col = egui::Color32::from_rgb(scol.r, scol.g, scol.b);
                    }
                    ui.painter().rect(
                        egui::Rect::from_min_size(
                            min + egui::vec2(i as f32 * unit.x, j as f32 * unit.y),
                            unit,
                        ),
                        egui::Rounding::none(),
                        col,
                        (5.0, egui::Color32::DARK_GRAY),
                    )
                }
            }
        });
    }
}
