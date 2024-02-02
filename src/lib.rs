extern crate js_sys;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys;

struct Demon {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    dx: i32,
    dy: i32,
}

impl Demon {
    fn draw(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        ctx.set_font("24px Arial");
        ctx.set_fill_style(&JsValue::from_str("#D24545"));
        ctx.fill_text(&format!("鬼"), self.x as f64, self.y as f64)
            .unwrap();        
    }
}

struct Game {
    canvas_context: web_sys::CanvasRenderingContext2d,
    canvas_width: i32,
    canvas_height: i32,
    demon: Demon,
    game_loop_closure: Option<Closure<dyn FnMut()>>, // ゲームループクローザ
    game_loop_interval_handle: Option<i32>,          // ゲームループのハンドル
}

impl Game {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let canvas = document.get_element_by_id("myCanvas").unwrap();
        let canvas = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let canvas_context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let canvas_width = canvas.width() as i32;
        let canvas_height = canvas.height() as i32;

        let mut dx = random_integer(2.0);
        let dy = random_integer(2.0);
        if dx == 0 && dy == 0 {
            dx = 1;
        }
        let demon = Demon {
            x: canvas_width / 2,
            y: canvas_height - 30,
            width: 10,
            height: 10,
            dx: dx,
            dy: dy,
        };

        Self {
            canvas_context,
            canvas_width,
            canvas_height,
            demon,
            game_loop_closure: None,
            game_loop_interval_handle: None,
        }
    }    

    // ゲームループ
    fn game_loop(&mut self) {
        // 画面を初期化
        self.canvas_context.clear_rect(
            0.0,
            0.0,
            self.canvas_width as f64,
            self.canvas_height as f64,
        );

        // 鬼の描画
        self.demon.draw(&self.canvas_context);

        // 鬼の移動
        self.demon.x = self.demon.x.saturating_add(self.demon.dx);
        self.demon.y = self.demon.y.saturating_add(self.demon.dy);

        self.start_game_loop();
    }

    // メソッドではなく、関連関数なので Game::set_game_loop_and_start() として呼び出す
    // 引数には自分自身を Rc<RefCell<>> で包んだものを渡す
    pub fn set_game_loop_and_start(game: Rc<RefCell<Self>>) {
        let cloned_game = game.clone();
        let mut game_borrow = game.borrow_mut();

        game_borrow.set_game_loop(move || cloned_game.borrow_mut().game_loop());
        game_borrow.start_game_loop();
    }

    fn set_game_loop<F: 'static>(&mut self, f: F)
    where
        F: FnMut(),
    {
        self.game_loop_closure = Some(Closure::new(f));
    }

    fn start_game_loop(&mut self) {
        // クロージャの参照を取り出す
        let closure = self.game_loop_closure.as_ref().unwrap();
        let window = web_sys::window().unwrap();

        let handle = window.request_animation_frame(closure.as_ref().unchecked_ref());

        self.game_loop_interval_handle = Some(handle.unwrap());
    }      
}

fn random_integer(length: f64) -> i32 {
    let random_length = 2.0 * length * js_sys::Math::random();
    (random_length - length) as i32
}

#[wasm_bindgen]
pub fn run() {
    let game = Game::new();
    let game = Rc::new(RefCell::new(game));
    Game::set_game_loop_and_start(game.clone());
}
