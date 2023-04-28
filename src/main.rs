use eframe::egui;

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
            board: Board::new(4),
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
    fn new(n: usize) -> Self {
        Board {
            pieces: vec![vec![Piece::default(); n]; n],
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
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let spectrum = colorous::RED_YELLOW_GREEN;
        egui::CentralPanel::default().show(ctx, |ui| {
            let unit = ui.available_size_before_wrap() / self.board.size as f32;
            if ui.input(|input| input.pointer.primary_pressed()) {
                if let Some(mpos) = ctx.pointer_latest_pos() {
                    let pos = (mpos.to_vec2() / unit).to_pos2();
                    self.board
                        .press(pos.x.trunc() as usize, pos.y.trunc() as usize);
                    dbg!(&self.board);
                }
            }

            for (j, row) in self.board.pieces.iter().enumerate() {
                for (i, piece) in row.iter().enumerate() {
                    let col = spectrum.eval_rational((piece.state + 1) % (piece.lim), piece.lim);
                    let col = egui::Color32::from_rgb(col.r, col.g, col.b);
                    ui.painter().rect(
                        egui::Rect::from_min_size(
                            egui::pos2(i as f32 * unit.x, j as f32 * unit.y),
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
