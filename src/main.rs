#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use rand::prelude::*;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Green the Board",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}

struct App {
    board: Board,
    show_numbers: bool,
    mmode: bool,
    status: ErrorMsg,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            board: Board::new(4, Zn { lim: 3 }),
            show_numbers: false,
            mmode: false,
            status: ErrorMsg::Ok,
        }
    }
}

/// Diagonal Magma: Each element has a unique square root
trait Diagma: std::fmt::Debug {
    fn mul(&self, a: usize, b: usize) -> usize;

    fn rquot(&self, a: usize, b: usize) -> Option<usize>;

    fn lquot(&self, a: usize, b: usize) -> Option<usize>;

    fn root(&self, a: usize) -> usize;

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

    fn modify(&self, a: usize, b: usize) -> Box<dyn Diagma>;

    fn row_swap(&self, a: usize, b: usize) -> Box<dyn Diagma>;

    fn col_swap(&self, a: usize, b: usize) -> Box<dyn Diagma>;

    fn trans_all(&self) -> Box<dyn Diagma>;

    fn rquot_all(&self) -> Result<Box<dyn Diagma>, FailType>;

    fn lquot_all(&self) -> Result<Box<dyn Diagma>, FailType>;
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
impl Diagma for Zn {
    fn mul(&self, a: usize, b: usize) -> usize {
        (a + b) % self.lim
    }

    fn rquot(&self, a: usize, b: usize) -> Option<usize> {
        Some((a + self.lim - b) % self.lim)
    }

    fn lquot(&self, a: usize, b: usize) -> Option<usize> {
        self.rquot(b, a)
    }

    fn root(&self, a: usize) -> usize {
        (a * (self.lim + 1) / 2) % self.lim
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

    fn modify(&self, a: usize, b: usize) -> Box<dyn Diagma> {
        let mut table: Vec<Vec<usize>> = self.table();
        table[a][b] = (table[a][b] + 1) % self.order();
        Box::new(SQ1 { table })
    }

    fn row_swap(&self, a: usize, b: usize) -> Box<dyn Diagma> {
        let mut table = self.table();
        table.swap(a, b);
        Box::new(SQ1 { table })
    }

    fn col_swap(&self, a: usize, b: usize) -> Box<dyn Diagma> {
        let mut table = self.table();
        for col in &mut table {
            col.swap(a, b);
        }
        Box::new(SQ1 { table })
    }

    fn trans_all(&self) -> Box<dyn Diagma> {
        let table = (0..self.order())
            .map(|n| (0..self.order()).map(|m| self.mul(m, n)).collect())
            .collect();
        Box::new(SQ1 { table })
    }

    fn rquot_all(&self) -> Result<Box<dyn Diagma>, FailType> {
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

    fn lquot_all(&self) -> Result<Box<dyn Diagma>, FailType> {
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
impl Diagma for SQ1 {
    fn mul(&self, a: usize, b: usize) -> usize {
        self.table[a][b]
    }

    fn rquot(&self, a: usize, b: usize) -> Option<usize> {
        for i in 0..self.order() {
            if self.table[i][b] == a {
                return Some(i);
            }
        }
        None
    }

    fn lquot(&self, a: usize, b: usize) -> Option<usize> {
        for i in 0..self.order() {
            if self.table[a][i] == b {
                return Some(i);
            }
        }
        None
    }

    fn root(&self, a: usize) -> usize {
        for i in 0..self.order() {
            if self.table[i][i] == a {
                return i;
            }
        }
        todo!()
    }

    fn order(&self) -> usize {
        self.table.len()
    }

    fn rep(&self, a: usize) -> String {
        (a + 1).to_string()
    }

    fn modify(&self, a: usize, b: usize) -> Box<dyn Diagma> {
        let mut table = self.table.clone();
        table[a][b] = (table[a][b] + 1) % self.order();
        Box::new(SQ1 { table })
    }

    fn row_swap(&self, a: usize, b: usize) -> Box<dyn Diagma> {
        let mut table = self.table.clone();
        table.swap(a, b);
        Box::new(SQ1 { table })
    }

    fn col_swap(&self, a: usize, b: usize) -> Box<dyn Diagma> {
        let mut table = self.table.clone();
        for col in &mut table {
            col.swap(a, b);
        }
        Box::new(SQ1 { table })
    }

    fn trans_all(&self) -> Box<dyn Diagma> {
        let table = (0..self.order())
            .map(|n| (0..self.order()).map(|m| self.mul(m, n)).collect())
            .collect();
        Box::new(SQ1 { table })
    }

    fn rquot_all(&self) -> Result<Box<dyn Diagma>, FailType> {
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

    fn lquot_all(&self) -> Result<Box<dyn Diagma>, FailType> {
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
    table: Box<dyn Diagma>,
    size: usize,
    undo_stack: MoveStack,
}
impl Board {
    fn new(n: usize, table: impl Diagma + 'static) -> Self {
        Self {
            pieces: vec![vec![table.init(); n]; n],
            table: Box::new(table),
            size: n,
            undo_stack: MoveStack::new(),
        }
    }

    fn set_table(&mut self, table: Box<dyn Diagma>) {
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
        let Move { x, y, d } = m;
        let val = if d >= 0 {
            self.pieces[y][x]
        } else {
            self.table.root(self.pieces[y][x])
        };
        let mut new_pieces = self.pieces.clone();
        self.op(&mut new_pieces, x, y, val, d)?;
        if y < self.size - 1 {
            self.op(&mut new_pieces, x, y + 1, val, d)?;
        }
        if y > 0 {
            self.op(&mut new_pieces, x, y - 1, val, d)?;
        }
        if x < self.size - 1 {
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
            self.press(m.inv())
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
        let x = (rng.gen::<f32>() * self.size as f32).floor() as usize;
        let y = (rng.gen::<f32>() * self.size as f32).floor() as usize;
        let _ = self.press(Move::new(x, y));
    }

    fn reset(&mut self) {
        for row in &mut self.pieces {
            for piece in row {
                *piece = self.table.init();
            }
        }
        self.undo_stack = MoveStack::new();
    }
}

enum FailType {
    UndoEmpty,
    RedoEmpty,
    NoDiv,
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
            let mut rng = thread_rng();
            for _ in 0..100 {
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
                                self.board = Board::new(n, Zn { lim });
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
        egui::SidePanel::right("Right").show(ctx, |ui| {
            if ui.button("Magma mode").clicked() {
                self.mmode = !self.mmode;
            };
            if self.mmode {
                ui.label("Warning: Unsafe");
            }
            ui.horizontal(|ui| {
                if self.mmode {
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
                }
            });
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
                            if ui
                                .button(self.board.table.rep(self.board.table.mul(i, j)))
                                .clicked()
                                && self.mmode
                            {
                                let new_table = self.board.table.modify(i, j);
                                self.board.set_table(new_table);
                            };
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
                                    (i + self.board.table.order() - 1) % self.board.table.order(),
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
                                    (j + self.board.table.order() - 1) % self.board.table.order(),
                                );
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
                    }
                });
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
            let rect = ui.available_rect_before_wrap();
            let (min, size) = (rect.left_top(), rect.size());
            let unit = size / self.board.size as f32;
            let font_size = (32. as f32).min(unit.y * 2. / 3.);

            // Handling mouse input
            if ui.input(|input| input.pointer.primary_pressed()) {
                if ui.ui_contains_pointer() {
                    if let Some(mpos) = ctx.pointer_latest_pos() {
                        let pos = ((mpos - min) / unit).to_pos2();
                        self.status = match self
                            .board
                            .apply_move(Move::new(pos.x.trunc() as usize, pos.y.trunc() as usize))
                        {
                            Ok(_) => ErrorMsg::Ok,
                            Err(e) => match e {
                                _ => ErrorMsg::Impossible,
                            },
                        };
                    }
                }
            } else if ui.input(|input| input.pointer.secondary_pressed()) {
                if ui.ui_contains_pointer() {
                    if let Some(mpos) = ctx.pointer_latest_pos() {
                        let pos = ((mpos - min) / unit).to_pos2();
                        self.status = match self.board.apply_move(
                            Move::new(pos.x.trunc() as usize, pos.y.trunc() as usize).inv(),
                        ) {
                            Ok(_) => ErrorMsg::Ok,
                            Err(e) => match e {
                                FailType::NoDiv => ErrorMsg::NoInv,
                                _ => ErrorMsg::Impossible,
                            },
                        };
                    }
                }
            }

            // Drawing the board
            for (j, row) in self.board.pieces.iter().enumerate() {
                for (i, &piece) in row.iter().enumerate() {
                    let col = if let Some(i) = self.board.table.ident() {
                        if piece == i {
                            egui::Color32::WHITE
                        } else if self.board.table.order() >= 2 {
                            spectrum((piece - 1) as f64 / (self.board.table.order() - 2) as f64)
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
