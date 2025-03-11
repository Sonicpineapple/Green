#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use rand::prelude::*;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Green the Board",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

enum ModalResponse {
    Confirm(Board),
    Close,
    None,
}
struct CreateZnModal {
    n: usize,
    m: usize,
    p: usize,
}
impl CreateZnModal {
    fn new() -> Self {
        Self { n: 4, m: 4, p: 3 }
    }

    fn show(&mut self, ctx: &egui::Context) -> ModalResponse {
        let modal = egui::Modal::new("Custom puzzle".into());
        modal
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let mut slider = |label, num, range| {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Slider::new(num, range).clamping(egui::SliderClamping::Never),
                            );
                            ui.label(label);
                        });
                    };
                    slider("X size", &mut self.n, 1..=16);
                    slider("Y size", &mut self.m, 1..=16);
                    slider("Modulo", &mut self.p, 1..=21);
                });
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        return ModalResponse::Confirm(Board::new(
                            self.n,
                            self.m,
                            Zn { lim: self.p },
                        ));
                    }
                    if ui.button("Cancel").clicked() {
                        return ModalResponse::Close;
                    }
                    ModalResponse::None
                })
                .inner
            })
            .inner
    }
}

struct Modals {
    create_zn: Option<CreateZnModal>,
}
impl Modals {
    fn new() -> Self {
        Self { create_zn: None }
    }
}

struct App {
    board: Board,
    show_numbers: bool,
    mmode: bool,
    show_table: bool,
    status: ErrorMsg,
    modals: Modals,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            board: Board::new(4, 4, Zn { lim: 3 }),
            show_numbers: false,
            mmode: false,
            show_table: true,
            status: ErrorMsg::Ok,
            modals: Modals::new(),
        }
    }
}

trait Magma: std::fmt::Debug {
    fn mul(&self, a: usize, b: usize) -> usize;

    fn rquot(&self, a: usize, b: usize) -> Option<usize>;

    fn lquot(&self, a: usize, b: usize) -> Option<usize>;

    fn root(&self, a: usize) -> Option<usize>;

    fn rep(&self, a: usize) -> String {
        a.to_string()
    }

    fn init(&self) -> usize {
        0
    }

    fn order(&self) -> usize;

    fn ident(&self) -> Option<usize> {
        None
    }

    fn modify(&self, a: usize, b: usize, d: isize) -> Box<dyn Magma>;

    fn row_swap(&self, a: usize, b: usize) -> Box<dyn Magma>;

    fn col_swap(&self, a: usize, b: usize) -> Box<dyn Magma>;

    fn trans_all(&self) -> Box<dyn Magma>;

    fn rquot_all(&self) -> Result<Box<dyn Magma>, FailType>;

    fn lquot_all(&self) -> Result<Box<dyn Magma>, FailType>;
}

#[derive(Debug, Clone)]
struct Zn {
    lim: usize,
}
impl Zn {
    fn table(&self) -> Vec<Vec<usize>> {
        (0..self.lim)
            .map(|n| (0..self.lim).map(|m| self.mul(n, m)).collect())
            .collect()
    }
}
impl Magma for Zn {
    fn mul(&self, a: usize, b: usize) -> usize {
        (a + b) % self.lim
    }

    fn rquot(&self, a: usize, b: usize) -> Option<usize> {
        Some((a + self.lim - b) % self.lim)
    }

    fn lquot(&self, a: usize, b: usize) -> Option<usize> {
        self.rquot(b, a)
    }

    fn root(&self, a: usize) -> Option<usize> {
        let n = a * (self.lim + 1);
        (n % 2 == 0).then_some((n / 2) % self.lim)
    }

    fn order(&self) -> usize {
        self.lim
    }

    fn ident(&self) -> Option<usize> {
        Some(0)
    }

    fn init(&self) -> usize {
        1
    }

    fn modify(&self, a: usize, b: usize, d: isize) -> Box<dyn Magma> {
        let mut table: Vec<Vec<usize>> = self.table();
        table[a][b] = ((table[a][b] + self.order()) as isize + d) as usize % self.order();
        Box::new(SQ1 { table })
    }

    fn row_swap(&self, a: usize, b: usize) -> Box<dyn Magma> {
        let mut table = self.table();
        table.swap(a, b);
        Box::new(SQ1 { table })
    }

    fn col_swap(&self, a: usize, b: usize) -> Box<dyn Magma> {
        let mut table = self.table();
        for col in &mut table {
            col.swap(a, b);
        }
        Box::new(SQ1 { table })
    }

    fn trans_all(&self) -> Box<dyn Magma> {
        let table = (0..self.order())
            .map(|n| (0..self.order()).map(|m| self.mul(m, n)).collect())
            .collect();
        Box::new(SQ1 { table })
    }

    fn rquot_all(&self) -> Result<Box<dyn Magma>, FailType> {
        let mut table = self.table();
        for x in 0..self.order() {
            for y in 0..self.order() {
                if let Some(z) = self.rquot(x, y) {
                    table[x][y] = z;
                } else {
                    return Err(FailType::NoDiv);
                }
            }
        }
        Ok(Box::new(SQ1 { table }))
    }

    fn lquot_all(&self) -> Result<Box<dyn Magma>, FailType> {
        let mut table = self.table();
        for x in 0..self.order() {
            for y in 0..self.order() {
                if let Some(z) = self.lquot(x, y) {
                    table[x][y] = z;
                } else {
                    return Err(FailType::NoDiv);
                }
            }
        }
        Ok(Box::new(SQ1 { table }))
    }
}

#[derive(Debug, Clone)]
struct SQ1 {
    table: Vec<Vec<usize>>,
}
impl Magma for SQ1 {
    fn mul(&self, a: usize, b: usize) -> usize {
        self.table[a][b]
    }

    fn rquot(&self, a: usize, b: usize) -> Option<usize> {
        let mut q = None;
        for i in 0..self.order() {
            if self.table[i][b] == a {
                if let Some(_) = q {
                    return None;
                }
                q = Some(i);
            }
        }
        q
    }

    fn lquot(&self, a: usize, b: usize) -> Option<usize> {
        let mut q = None;
        for i in 0..self.order() {
            if self.table[a][i] == b {
                if let Some(_) = q {
                    return None;
                }
                q = Some(i);
            }
        }
        q
    }

    fn root(&self, a: usize) -> Option<usize> {
        for i in 0..self.order() {
            if self.table[i][i] == a {
                return Some(i);
            }
        }
        None
    }

    fn order(&self) -> usize {
        self.table.len()
    }

    fn rep(&self, a: usize) -> String {
        (a + 1).to_string()
    }

    fn ident(&self) -> Option<usize> {
        for i in 0..self.order() {
            if (0..self.order()).fold(true, |acc, n| {
                acc && self.mul(i, n) == n && self.mul(n, i) == n
            }) {
                return Some(i);
            }
        }
        None
    }

    fn init(&self) -> usize {
        if let Some(m) = self.ident() {
            if m == 0 {
                return 1;
            }
        }
        0
    }

    fn modify(&self, a: usize, b: usize, d: isize) -> Box<dyn Magma> {
        let mut table = self.table.clone();
        table[a][b] = ((table[a][b] + self.order()) as isize + d) as usize % self.order();
        Box::new(SQ1 { table })
    }

    fn row_swap(&self, a: usize, b: usize) -> Box<dyn Magma> {
        let mut table = self.table.clone();
        table.swap(a, b);
        Box::new(SQ1 { table })
    }

    fn col_swap(&self, a: usize, b: usize) -> Box<dyn Magma> {
        let mut table = self.table.clone();
        for col in &mut table {
            col.swap(a, b);
        }
        Box::new(SQ1 { table })
    }

    fn trans_all(&self) -> Box<dyn Magma> {
        let table = (0..self.order())
            .map(|n| (0..self.order()).map(|m| self.mul(m, n)).collect())
            .collect();
        Box::new(SQ1 { table })
    }

    fn rquot_all(&self) -> Result<Box<dyn Magma>, FailType> {
        let mut table = self.table.clone();
        for x in 0..self.order() {
            for y in 0..self.order() {
                if let Some(z) = self.rquot(x, y) {
                    table[x][y] = z;
                } else {
                    return Err(FailType::NoDiv);
                }
            }
        }
        Ok(Box::new(SQ1 { table }))
    }

    fn lquot_all(&self) -> Result<Box<dyn Magma>, FailType> {
        let mut table = self.table.clone();
        for x in 0..self.order() {
            for y in 0..self.order() {
                if let Some(z) = self.lquot(x, y) {
                    table[x][y] = z;
                } else {
                    return Err(FailType::NoDiv);
                }
            }
        }
        Ok(Box::new(SQ1 { table }))
    }
}
impl SQ1 {
    fn new_lights_out(n: usize) -> Self {
        let table = (0..n)
            .map(|a| (0..n).map(|_| (a + 1) % n).collect())
            .collect();
        SQ1 { table }
    }
}

#[derive(Debug)]
struct Board {
    pieces: Vec<Vec<usize>>,
    table: Box<dyn Magma>,
    size: (usize, usize),
    undo_stack: MoveStack,
}
impl Board {
    fn new(x: usize, y: usize, table: impl Magma + 'static) -> Self {
        Self {
            pieces: vec![vec![table.init(); x]; y],
            table: Box::new(table),
            size: (x, y),
            undo_stack: MoveStack::new(),
        }
    }

    fn set_table(&mut self, table: Box<dyn Magma>) {
        self.table = table;
        self.reset();
    }

    fn mul(&self, pieces: &mut Vec<Vec<usize>>, x: usize, y: usize, val: usize, times: usize) {
        let mut r = pieces[y][x];
        for _ in 0..times {
            r = self.table.mul(r, val);
        }
        pieces[y][x] = r;
    }

    fn div(
        &self,
        pieces: &mut Vec<Vec<usize>>,
        x: usize,
        y: usize,
        val: usize,
        times: usize,
    ) -> Result<(), FailType> {
        let mut r = pieces[y][x];
        for _ in 0..times {
            if let Some(z) = self.table.rquot(r, val) {
                r = z;
            } else {
                return Err(FailType::NoDiv);
            }
        }
        pieces[y][x] = r;
        return Ok(());
    }

    fn op(
        &self,
        pieces: &mut Vec<Vec<usize>>,
        x: usize,
        y: usize,
        val: usize,
        times: isize,
    ) -> Result<(), FailType> {
        if times >= 0 {
            Ok(self.mul(pieces, x, y, val, times as usize))
        } else {
            self.div(pieces, x, y, val, -times as usize)
        }
    }

    fn press(&mut self, m: Move) -> Result<(), FailType> {
        let (sizex, sizey) = self.size;
        let Move { x, y, d } = m;
        let val = if d >= 0 {
            self.pieces[y][x]
        } else {
            match self.table.root(self.pieces[y][x]) {
                Some(r) => r,
                None => return Err(FailType::NoRoot),
            }
        };
        let mut new_pieces = self.pieces.clone();
        self.op(&mut new_pieces, x, y, val, d)?;
        if y < sizey - 1 {
            self.op(&mut new_pieces, x, y + 1, val, d)?;
        }
        if y > 0 {
            self.op(&mut new_pieces, x, y - 1, val, d)?;
        }
        if x < sizex - 1 {
            self.op(&mut new_pieces, x + 1, y, val, d)?;
        }
        if x > 0 {
            self.op(&mut new_pieces, x - 1, y, val, d)?;
        }
        self.pieces = new_pieces;
        Ok(())
    }

    fn undo(&mut self) -> Result<(), FailType> {
        if let Some(m) = self.undo_stack.undo() {
            let res = self.press(m.inv());
            match res {
                Err(FailType::NoDiv) => {
                    self.undo_stack.redo();
                }
                _ => {}
            };
            res
        } else {
            Err(FailType::UndoEmpty)
        }
    }

    fn redo(&mut self) -> Result<(), FailType> {
        if let Some(m) = self.undo_stack.redo() {
            self.press(m)
        } else {
            Err(FailType::RedoEmpty)
        }
    }

    fn apply_move(&mut self, m: Move) -> Result<(), FailType> {
        if m.d != 0 {
            let state = self.press(m);
            if state.is_err() {
                return state;
            }
            self.undo_stack.push(m);
        }
        Ok(())
    }

    fn random_move(&mut self, rng: &mut ThreadRng) {
        let x = (rng.random::<f32>() * self.size.0 as f32).floor() as usize;
        let y = (rng.random::<f32>() * self.size.1 as f32).floor() as usize;
        let _ = self.press(Move::new(x, y));
    }

    fn reset(&mut self) {
        self.pieces = vec![vec![self.table.init(); self.size.0]; self.size.1];
        self.undo_stack = MoveStack::new();
    }
}

enum FailType {
    UndoEmpty,
    RedoEmpty,
    NoDiv,
    NoRoot,
}

enum ErrorMsg {
    Ok,
    NoUndo,
    NoRedo,
    NoInv,
    NoRDiv,
    NoLDiv,
    Impossible,
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let spectrum = |t: f64| {
            let col = colorous::RAINBOW.eval_continuous(0.57 * (1. - t) + 0.32 * t);
            egui::Color32::from_rgb(col.r, col.g, col.b)
        };

        let scramble_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::F);
        let scramble = |board: &mut Board| {
            board.reset();
            let mut rng = rand::rng();
            let length = board.size.0 * board.size.1 * board.table.order();
            for _ in 0..length {
                board.random_move(&mut rng);
            }
        };
        if ctx.input_mut(|input| input.consume_shortcut(&scramble_shortcut)) {
            scramble(&mut self.board);
        }

        let reset_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::R);
        let reset = |board: &mut Board| board.reset();
        if ctx.input_mut(|input| input.consume_shortcut(&reset_shortcut)) {
            reset(&mut self.board);
        }

        let redo_shortcut = egui::KeyboardShortcut::new(
            egui::Modifiers::CTRL.plus(egui::Modifiers::SHIFT),
            egui::Key::Z,
        );
        let redo = |board: &mut Board| match board.redo() {
            Ok(_) => ErrorMsg::Ok,
            Err(e) => match e {
                FailType::RedoEmpty => ErrorMsg::NoRedo,
                FailType::NoDiv => ErrorMsg::NoInv,
                _ => ErrorMsg::Impossible,
            },
        };
        if ctx.input_mut(|input| input.consume_shortcut(&redo_shortcut)) {
            self.status = redo(&mut self.board);
        }

        let undo_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Z);
        let undo = |board: &mut Board| match board.undo() {
            Ok(_) => ErrorMsg::Ok,
            Err(e) => match e {
                FailType::UndoEmpty => ErrorMsg::NoUndo,
                FailType::NoDiv => ErrorMsg::NoInv,
                _ => ErrorMsg::Impossible,
            },
        };
        if ctx.input_mut(|input| input.consume_shortcut(&undo_shortcut)) {
            self.status = undo(&mut self.board);
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
                    if ui.button("Test").clicked() {
                        self.board = Board::new(
                            4,
                            4,
                            SQ1 {
                                table: vec![vec![1, 0, 2], vec![0, 2, 1], vec![2, 1, 0]],
                            },
                        );
                        ui.close_menu();
                    }
                    for n in 2..=6 {
                        for l in 1..=3 {
                            let n_str = n.to_string();
                            let lim = 2 * l + 1;
                            let lim_str = lim.to_string();
                            if ui
                                .button("".to_string() + &n_str + "x" + &n_str + ", " + &lim_str)
                                .clicked()
                            {
                                self.board = Board::new(n, n, Zn { lim });
                                ui.close_menu();
                            }
                        }
                    }

                    if ui.button("Custom...").clicked() {
                        self.modals.create_zn = Some(CreateZnModal::new());
                        ui.close_menu();
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
        egui::SidePanel::right("Right").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("X");
                if ui.button("-").clicked() {
                    if self.board.size.0 > 1 {
                        self.board.size.0 -= 1;
                    }
                    self.board.reset();
                }
                if ui.button("+").clicked() {
                    self.board.size.0 += 1;
                    self.board.reset();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Y");
                if ui.button("-").clicked() {
                    if self.board.size.1 > 1 {
                        self.board.size.1 -= 1;
                    }
                    self.board.reset();
                }
                if ui.button("+").clicked() {
                    self.board.size.1 += 1;
                    self.board.reset();
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Magma mode").clicked() {
                    self.mmode = !self.mmode;
                };
                let label = match self.show_table {
                    true => "Hide table",
                    false => "Show table",
                };
                if ui.button(label).clicked() {
                    self.show_table = !self.show_table;
                }
            });
            if self.mmode {
                ui.label("Warning: Unsafe");
                ui.horizontal(|ui| {
                    if ui.button("-").clicked() {
                        let n = (self.board.table.order() - 1).max(2);
                        let new_table = Box::new(SQ1::new_lights_out(n));
                        self.board.table = new_table;
                        self.board.reset();
                    }
                    if ui.button("+").clicked() {
                        let n = self.board.table.order() + 1;
                        let new_table = Box::new(SQ1::new_lights_out(n));
                        self.board.table = new_table;
                        self.board.reset();
                    }
                });
            }
            if self.show_table {
                egui::Grid::new("Table_grid")
                    .min_col_width(0.)
                    .spacing((0.1, 0.1))
                    .show(ui, |ui| {
                        if self.mmode {}
                        ui.label("*");
                        ui.label(" ");
                        for i in 0..self.board.table.order() {
                            ui.label(self.board.table.rep(i));
                        }
                        ui.end_row();
                        for i in 0..self.board.table.order() {
                            ui.label(self.board.table.rep(i));
                            ui.label(" ");
                            for j in 0..self.board.table.order() {
                                let button =
                                    ui.button(self.board.table.rep(self.board.table.mul(i, j)));
                                if self.mmode {
                                    if button.clicked_by(egui::PointerButton::Primary) {
                                        let new_table = self.board.table.modify(i, j, 1);
                                        self.board.set_table(new_table);
                                    } else if button.clicked_by(egui::PointerButton::Secondary) {
                                        let new_table = self.board.table.modify(i, j, -1);
                                        self.board.set_table(new_table);
                                    };
                                }
                            }
                            if self.mmode {
                                if ui.button("⬇").clicked() {
                                    let new_table = self
                                        .board
                                        .table
                                        .row_swap(i, (i + 1) % self.board.table.order());
                                    self.board.set_table(new_table)
                                }
                                if ui.button("⬆").clicked() {
                                    let new_table = self.board.table.row_swap(
                                        i,
                                        (i + self.board.table.order() - 1)
                                            % self.board.table.order(),
                                    );
                                    self.board.set_table(new_table);
                                }
                            }
                            ui.end_row();
                        }
                        ui.label(" ");
                        ui.label(" ");
                        if self.mmode {
                            for j in 0..self.board.table.order() {
                                if ui.button("➡").clicked() {
                                    let new_table = self
                                        .board
                                        .table
                                        .col_swap(j, (j + 1) % self.board.table.order());
                                    self.board.set_table(new_table)
                                }
                            }
                            ui.end_row();
                            ui.label(" ");
                            ui.label(" ");
                            for j in 0..self.board.table.order() {
                                if ui.button("⬅").clicked() {
                                    let new_table = self.board.table.col_swap(
                                        j,
                                        (j + self.board.table.order() - 1)
                                            % self.board.table.order(),
                                    );
                                    self.board.set_table(new_table);
                                }
                            }
                            ui.end_row();
                            ui.label(" ");
                            ui.label(" ");
                            for j in 0..self.board.table.order() {
                                if ui.button("↖").clicked() {
                                    let new_table = self
                                        .board
                                        .table
                                        .col_swap(
                                            j,
                                            (j + self.board.table.order() - 1)
                                                % self.board.table.order(),
                                        )
                                        .row_swap(
                                            j,
                                            (j + self.board.table.order() - 1)
                                                % self.board.table.order(),
                                        );
                                    self.board.set_table(new_table);
                                }
                            }
                            ui.end_row();
                            ui.label(" ");
                            ui.label(" ");
                            for j in 0..self.board.table.order() {
                                if ui.button("↘").clicked() {
                                    let new_table = self
                                        .board
                                        .table
                                        .col_swap(j, (j + 1) % self.board.table.order())
                                        .row_swap(j, (j + 1) % self.board.table.order());
                                    self.board.set_table(new_table);
                                }
                            }
                            ui.end_row();
                            ui.label(" ");
                            ui.label(" ");
                            if ui.button("T").clicked() {
                                self.board.set_table(self.board.table.trans_all());
                                self.status = ErrorMsg::Ok;
                            }
                            if ui.button("/").clicked() {
                                match self.board.table.rquot_all() {
                                    Ok(table) => {
                                        self.board.set_table(table);
                                        self.status = ErrorMsg::Ok;
                                    }
                                    Err(e) => {
                                        self.status = match e {
                                            FailType::NoDiv => ErrorMsg::NoRDiv,
                                            _ => ErrorMsg::Impossible,
                                        };
                                    }
                                }
                            }
                            if ui.button("\\").clicked() {
                                match self.board.table.lquot_all() {
                                    Ok(table) => {
                                        self.board.set_table(table);
                                        self.status = ErrorMsg::Ok;
                                    }
                                    Err(e) => {
                                        self.status = match e {
                                            FailType::NoDiv => ErrorMsg::NoLDiv,
                                            _ => ErrorMsg::Ok,
                                        };
                                    }
                                }
                            }
                            if ui.button("R").clicked() {
                                let range =
                                    rand::distr::Uniform::new(0, self.board.table.order()).unwrap();
                                let mut rng = rand::rng();
                                let new_table = (0..self.board.table.order())
                                    .map(|m| {
                                        (0..self.board.table.order())
                                            .map(|n| {
                                                if m == n {
                                                    self.board.table.mul(m, n)
                                                } else {
                                                    range.sample(&mut rng)
                                                }
                                            })
                                            .collect()
                                    })
                                    .collect();
                                self.board.set_table(Box::new(SQ1 { table: new_table }));
                            }
                        }
                    });
            }
            ui.label(match self.status {
                ErrorMsg::Ok => "",
                ErrorMsg::NoUndo => "Nothing to undo",
                ErrorMsg::NoRedo => "Nothing to redo",
                ErrorMsg::NoInv => "No unique inverse exists",
                ErrorMsg::NoRDiv => "Right division is not defined",
                ErrorMsg::NoLDiv => "Left division is not defined",
                ErrorMsg::Impossible => "How did this happen?",
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(modal) = &mut self.modals.create_zn {
                match modal.show(ctx) {
                    ModalResponse::Confirm(board) => {
                        self.board = board;
                        self.modals.create_zn = None;
                    }
                    ModalResponse::Close => self.modals.create_zn = None,
                    ModalResponse::None => {}
                }
            }

            let rect = ui.available_rect_before_wrap();
            let (min, size) = (rect.left_top(), rect.size());
            let unit = egui::vec2(
                size.x / self.board.size.0 as f32,
                size.y / self.board.size.1 as f32,
            );
            let font_size = (32. as f32).min(unit.y * 2. / 3.);

            // Handling mouse input
            let r = ui.interact(rect, "Board".into(), egui::Sense::click());
            if let Some(mpos) = r.interact_pointer_pos() {
                let pos = ((mpos - min) / unit).to_pos2();
                if r.clicked() {
                    self.status = match self
                        .board
                        .apply_move(Move::new(pos.x.trunc() as usize, pos.y.trunc() as usize))
                    {
                        Ok(_) => ErrorMsg::Ok,
                        Err(e) => match e {
                            _ => ErrorMsg::Impossible,
                        },
                    };
                } else if r.secondary_clicked() {
                    self.status = match self
                        .board
                        .apply_move(Move::new(pos.x.trunc() as usize, pos.y.trunc() as usize).inv())
                    {
                        Ok(_) => ErrorMsg::Ok,
                        Err(e) => match e {
                            FailType::NoDiv | FailType::NoRoot => ErrorMsg::NoInv,
                            _ => ErrorMsg::Impossible,
                        },
                    };
                }
            }

            // Drawing the board
            ui.style_mut().interaction.selectable_labels = false;
            for (j, row) in self.board.pieces.iter().enumerate() {
                for (i, &piece) in row.iter().enumerate() {
                    let col = if let Some(i) = self.board.table.ident() {
                        if piece == i {
                            egui::Color32::WHITE
                        } else if self.board.table.order() >= 2 {
                            let p = if piece > i { piece - 1 } else { piece };
                            spectrum((p) as f64 / (self.board.table.order() - 2) as f64)
                        } else {
                            dbg!("Boo");
                            spectrum(0. as f64)
                        }
                    } else {
                        spectrum((piece) as f64 / (self.board.table.order() - 1) as f64)
                    };
                    ui.painter().rect(
                        egui::Rect::from_min_size(
                            min + egui::vec2(i as f32 * unit.x, j as f32 * unit.y),
                            unit,
                        ),
                        egui::CornerRadius::ZERO,
                        col,
                        (5.0, egui::Color32::DARK_GRAY),
                        egui::StrokeKind::Middle,
                    );
                    if self.show_numbers {
                        ui.put(
                            egui::Rect::from_min_size(
                                min + egui::vec2(i as f32 * unit.x, j as f32 * unit.y),
                                unit,
                            ),
                            egui::Label::new(
                                egui::RichText::new(self.board.table.rep(piece))
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
