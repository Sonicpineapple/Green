#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
    show_numbers: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            board: Board::new(4, 3),
            show_numbers: false,
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
    undo_stack: MoveStack,
}
impl Board {
    fn new(n: usize, lim: usize) -> Self {
        Board {
            pieces: vec![vec![Piece::new(lim); n]; n],
            size: n,
            undo_stack: MoveStack::new(),
        }
    }

    fn press(&mut self, m: Move) {
        let Move { x, y, d } = m;
        let d = (d % (self.pieces[y][x].lim as isize)) as usize;
        let val = d * self.pieces[y][x].state;
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

    fn inv_press(&mut self, m: Move) {
        let Move { x, y, d } = m;
        let d = (-d % (self.pieces[y][x].lim as isize)) as usize;
        let val = d * self.pieces[y][x].state;
        let val =
            self.pieces[y][x].lim - (val * (self.pieces[y][x].lim + 1) / 2) % self.pieces[y][x].lim; // TODO: verify inverse respects individual piece limits
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

    fn undo(&mut self) {
        if let Some(m) = self.undo_stack.undo() {
            self.inv_press(m.inv());
        }
    }

    fn redo(&mut self) {
        if let Some(m) = self.undo_stack.redo() {
            self.press(m);
        }
    }

    fn apply_move(&mut self, m: Move) {
        if m.d != 0 {
            if m.d > 0 {
                self.press(m);
            } else if m.d < 0 {
                self.inv_press(m);
            }
            self.undo_stack.push(m);
        }
    }

    fn random_move(&mut self, rng: &mut ThreadRng) {
        let x = (rng.gen::<f32>() * self.size as f32).floor() as usize;
        let y = (rng.gen::<f32>() * self.size as f32).floor() as usize;
        self.press(Move::new(x, y));
    }

    fn reset(&mut self) {
        for row in &mut self.pieces {
            for piece in row {
                piece.state = 1;
            }
        }
        self.undo_stack = MoveStack::new();
    }
}

#[derive(Debug, Copy, Clone)]
struct Move {
    x: usize,
    y: usize,
    d: isize,
}
impl Move {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y, d: 1 }
    }

    fn inv(&self) -> Self {
        Self {
            d: -self.d,
            ..*self
        }
    }

    fn loc(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}

#[derive(Debug, Clone)]
struct MoveStack {
    stack: Vec<Move>,
    index: usize,
}
impl MoveStack {
    fn new() -> Self {
        Self {
            stack: vec![],
            index: 0,
        }
    }

    fn push(&mut self, m: Move) {
        if self.index != 0 {
            self.stack = self.stack[..self.stack.len() - self.index].to_vec();
            self.index = 0;
        }
        self.stack.push(m);
    }

    fn undo(&mut self) -> Option<Move> {
        if self.index < self.stack.len() {
            self.index += 1;
            return Some(self.stack[self.stack.len() - self.index]);
        }
        return None;
    }

    fn redo(&mut self) -> Option<Move> {
        if self.index > 0 {
            self.index -= 1;
            return Some(self.stack[self.stack.len() - self.index - 1]);
        }
        return None;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let spectrum = colorous::RED_YELLOW_GREEN;

        let scramble_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::F);
        let scramble = |board: &mut Board| {
            board.reset();
            let mut rng = thread_rng();
            for _ in 0..100 {
                board.random_move(&mut rng);
            }
        };
        if ctx.input_mut(|input| input.consume_shortcut(&scramble_shortcut)) {
            scramble(&mut self.board);
        }

        let reset_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::R);
        let reset = |board: &mut Board| {
            board.reset();
        };
        if ctx.input_mut(|input| input.consume_shortcut(&reset_shortcut)) {
            reset(&mut self.board);
        }

        let undo_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Z);
        let undo = |board: &mut Board| {
            board.undo();
        };
        if ctx.input_mut(|input| input.consume_shortcut(&undo_shortcut)) {
            undo(&mut self.board);
        }

        let redo_shortcut = egui::KeyboardShortcut::new(
            egui::Modifiers::CTRL.plus(egui::Modifiers::SHIFT),
            egui::Key::Z,
        );
        let redo = |board: &mut Board| {
            board.redo();
        };
        if ctx.input_mut(|input| input.consume_shortcut(&redo_shortcut)) {
            redo(&mut self.board);
        }

        let toggle_num_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::N);
        let toggle_num = |app: &mut App| {
            app.show_numbers = !app.show_numbers;
        };
        if ctx.input_mut(|input| input.consume_shortcut(&toggle_num_shortcut)) {
            toggle_num(self);
        }

        egui::TopBottomPanel::top("Top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Edit", |ui| {
                    let scramble_button = egui::Button::new("Scramble")
                        .shortcut_text(ctx.format_shortcut(&scramble_shortcut));
                    if ui.add(scramble_button).clicked() {
                        scramble(&mut self.board);
                        ui.close_menu();
                    }

                    let reset_button = egui::Button::new("Reset")
                        .shortcut_text(ctx.format_shortcut(&reset_shortcut));
                    if ui.add(reset_button).clicked() {
                        reset(&mut self.board);
                        ui.close_menu();
                    }

                    let undo_button = egui::Button::new("Undo")
                        .shortcut_text(ctx.format_shortcut(&undo_shortcut));
                    if ui.add(undo_button).clicked() {
                        undo(&mut self.board);
                        ui.close_menu();
                    }

                    let redo_button = egui::Button::new("Redo")
                        .shortcut_text(ctx.format_shortcut(&redo_shortcut));
                    if ui.add(redo_button).clicked() {
                        redo(&mut self.board);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Puzzle", |ui| {
                    for n in 2..=6 {
                        for l in 1..=3 {
                            let n_str = n.to_string();
                            let lim = 2 * l + 1;
                            let lim_str = lim.to_string();
                            if ui
                                .button("".to_string() + &n_str + "x" + &n_str + ", " + &lim_str)
                                .clicked()
                            {
                                self.board = Board::new(n, lim);
                                ui.close_menu();
                            }
                        }
                    }
                });
                ui.menu_button("Options", |ui| {
                    let toggle_num_button = egui::Button::new("Toggle numbers")
                        .shortcut_text(ctx.format_shortcut(&toggle_num_shortcut));
                    if ui.add(toggle_num_button).clicked() {
                        toggle_num(self);
                    }
                })
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let (min, size) = (rect.left_top(), rect.size());
            let unit = size / self.board.size as f32;
            let font_size = (32. as f32).min(unit.y * 2. / 3.);

            // Handling mouse input
            if ui.input(|input| input.pointer.primary_pressed()) {
                if ui.ui_contains_pointer() {
                    if let Some(mpos) = ctx.pointer_latest_pos() {
                        let pos = ((mpos - min) / unit).to_pos2();
                        self.board
                            .apply_move(Move::new(pos.x.trunc() as usize, pos.y.trunc() as usize));
                    }
                }
            } else if ui.input(|input| input.pointer.secondary_pressed()) {
                if ui.ui_contains_pointer() {
                    if let Some(mpos) = ctx.pointer_latest_pos() {
                        let pos = ((mpos - min) / unit).to_pos2();
                        self.board.apply_move(
                            Move::new(pos.x.trunc() as usize, pos.y.trunc() as usize).inv(),
                        );
                    }
                }
            }

            // Drawing the board
            for (j, row) in self.board.pieces.iter().enumerate() {
                for (i, piece) in row.iter().enumerate() {
                    let col: egui::Color32;
                    if piece.state == 0 {
                        col = egui::Color32::WHITE;
                    } else {
                        let scol = spectrum.eval_rational(
                            (piece.lim - piece.state - 1) % (piece.lim - 1),
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
                    );
                    if self.show_numbers {
                        ui.put(
                            egui::Rect::from_min_size(
                                min + egui::vec2(i as f32 * unit.x, j as f32 * unit.y),
                                unit,
                            ),
                            egui::Label::new(
                                egui::RichText::new(piece.state.to_string())
                                    .color(egui::Color32::DARK_GRAY)
                                    .size(font_size),
                            ),
                        );
                    }
                }
            }
        });
    }
}
