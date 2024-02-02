extern crate js_sys;

use anyhow::{anyhow, Result};
use futures::channel::{
    mpsc::{unbounded, UnboundedReceiver},
    oneshot::channel,
};
use std::{cell::RefCell, sync::Mutex};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{
    closure::WasmClosure, closure::WasmClosureFnOnce, prelude::Closure, JsCast, JsValue,
};
use web_sys::{self, HtmlImageElement};

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

macro_rules! error {
    ( $( $t:tt )* ) => {
        web_sys::console::error_1(&format!( $( $t )* ).into());
    }
}

const DEFAULT_COUNTER: i32 = 1000;

struct Demon {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    dx: i32,
    dy: i32,
}

impl Demon {
    pub fn new(canvas_width: i32, canvas_height: i32) -> Self {
        Self {
            x: random_integer((canvas_width as f64) / 2.0) + (canvas_width / 2),
            y: random_integer((canvas_height as f64) / 2.0) + (canvas_height / 2),
            width: 10,
            height: 10,
            dx: random_integer(2.0),
            dy: random_integer(2.0),
        }
    }

    fn draw(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        ctx.set_font("24px Arial");
        ctx.set_fill_style(&JsValue::from_str("#D24545"));
        ctx.fill_text(&format!("鬼"), self.x as f64, self.y as f64)
            .unwrap();        
    }

    fn stopped(&self) -> bool {
        (self.dx == 0) && (self.dy == 0)
    }

    fn panic(&mut self) {
        self.dx = random_integer(2.0);
        self.dy = random_integer(2.0);
    }
}

struct Demons {
    inner: Vec<Demon>,
}

impl Demons {
    pub fn new(num: i32, canvas_width: i32, canvas_height: i32) -> Self {
        let mut demons: Vec<Demon> = vec![];

        for n in 0..num {
            let demon = Demon::new(canvas_width, canvas_height);
            demons.push(demon);
        }

        Self { inner: demons }
    }

    fn draw(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        self.inner.iter().for_each(|d| d.draw(ctx));
    }    
}

struct Bean {
    x: i32,
    y: i32,
    radius: i32,
}

impl Bean {
    fn draw(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        ctx.begin_path();
        let _ = ctx.arc(
            self.x as f64,
            self.y as f64,
            self.radius as f64,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.set_fill_style(&JsValue::from_str("#E1D3A9"));
        ctx.fill();
        ctx.close_path();
    }

    fn diameter(&self) -> i32 {
        self.radius * 2
    }
}

struct Circle {
    x: i32,
    y: i32,
    radius: i32,    
}

impl Circle {
    fn draw(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        ctx.begin_path();
        let _ = ctx.arc(
            self.x as f64,
            self.y as f64,
            self.radius as f64,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.set_fill_style(&JsValue::from_str("#D1E6E8"));
        ctx.fill();
        ctx.close_path();
    }    
}

struct UserInput {
    mouse_x: i32,
    mouse_y: i32,
}

impl UserInput {
    fn set_mouse_position(&mut self, x: i32, y: i32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }    
}

struct Game {
    canvas_context: web_sys::CanvasRenderingContext2d,
    canvas_width: i32,
    canvas_height: i32,
    demon_image: Result<HtmlImageElement>,
    bean_image: Result<HtmlImageElement>,
    circle: Circle,
    demons: Demons,
    bean: Bean,
    score: u16,
    counter: i32,
    user_input: UserInput,    
    game_loop_closure: Option<Closure<dyn FnMut()>>, // ゲームループクローザ
    game_loop_interval_handle: Option<i32>,          // ゲームループのハンドル
}

impl Game {
    pub async fn new() -> Self {
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

        let demon_image = load_image("demon.png").await;
        // log!("{:#?}", demon_image);
        let bean_image = load_image("bean.png").await;

        let circle = Circle {
            x: canvas_width / 2,
            y: canvas_height / 2,
            radius: canvas_width / 3,            
        };

        let demons = Demons::new(5, canvas_width, canvas_height);

        let bean = Bean {
            x: canvas_width / 2,
            y: canvas_height / 2,
            radius: 10,
        };

        let user_input = UserInput {
            mouse_x: 0,
            mouse_y: 0,            
        };

        Self {
            canvas_context,
            canvas_width,
            canvas_height,
            demon_image,
            bean_image,
            circle,
            demons,
            bean,
            score: 0,
            counter: DEFAULT_COUNTER,
            user_input,
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

        // サークルの描画
        self.circle.draw(&self.canvas_context);

        // 鬼の描画
        self.demons.draw(&self.canvas_context);

        // 大豆の描画
        self.bean.draw(&self.canvas_context);
        // スコアの描画
        self.draw_score();
        self.draw_counter();

        // 衝突処理
        self.collision_detection();           

        // 大豆をマウスに追従させる
        let canvas = self.canvas_context.canvas().unwrap();
        let relative_x = self.user_input.mouse_x.saturating_sub(canvas.offset_left());
        if relative_x > self.bean.diameter() && relative_x < self.canvas_width {
            self.bean.x = relative_x.saturating_sub(self.bean.radius);
        }
        let relative_y = self.user_input.mouse_y.saturating_sub(canvas.offset_top());
        if relative_y > self.bean.diameter() && relative_y < self.canvas_height {
            self.bean.y = relative_y.saturating_sub(self.bean.radius);
        }

        for demon in &mut self.demons.inner {
            // 鬼の移動先
            let moved_demon_x = demon.x.saturating_add(demon.dx);
            let moved_demon_y = demon.y.saturating_add(demon.dy);

            // 鬼と左右の壁の衝突
            if moved_demon_x > self.canvas_width - demon.width || moved_demon_x < 0 {
                demon.dx = -demon.dx;
            }

            // 鬼と上下の壁の衝突
            if moved_demon_y > self.canvas_height - demon.height || moved_demon_y < 0 {
                demon.dy = -demon.dy;
            }

            // 鬼と大豆の衝突
            if (self.bean.x - self.bean.radius < moved_demon_x && moved_demon_x < self.bean.x + self.bean.radius) ||
            (self.bean.y - self.bean.radius < moved_demon_y && moved_demon_y < self.bean.y + self.bean.radius)
            {
                demon.dx = -demon.dx;
                demon.dy = -demon.dy;
            }

            if demon.stopped() {
                demon.panic();
            }                          
        }

        // 鬼の移動
        for demon in &mut self.demons.inner {
            demon.x = demon.x.saturating_add(demon.dx);
            demon.y = demon.y.saturating_add(demon.dy);       
        }

        // スコアの計算
        self.score = 0;
        for demon in &mut self.demons.inner {
            if (self.circle.x - self.circle.radius < demon.x && demon.x < self.circle.x + self.circle.radius) &&
            (self.circle.y - self.circle.radius < demon.y && demon.y < self.circle.y + self.circle.radius)
            {
                self.score += 1;
            }              
        }                

        self.counter -= 1;

        if self.counter <= 0 {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            self.counter = DEFAULT_COUNTER;
            window.alert_with_message(&format!("ゲーム終了!!\nスコア: {}", self.score)).unwrap();
            document.location().unwrap().reload().unwrap();
        }

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

    // メソッドではなく、関連関数なので Game::set_input() として呼び出す
    // 引数には自分自身を Rc<RefCell<>> で包んだものを渡す
    pub fn set_input_event(game: Rc<RefCell<Self>>) {
        let game_mouse_move = game.clone();
        let document = web_sys::window().unwrap().document().unwrap();

        let closure = Closure::new(Box::new(move |event: web_sys::MouseEvent| {
            let mut g = game_mouse_move.borrow_mut();
            g.user_input
                .set_mouse_position(event.client_x(), event.client_y());
        }) as Box<dyn FnMut(_)>);

        document.set_onmousemove(Some(&closure.as_ref().unchecked_ref()));
        // forget()するとRust側はdropされるが、into_js_value()されてブラウザ側に残る
        closure.forget();         
    }

    // スコア描画
    fn draw_score(&self) {
        self.canvas_context.set_font("16px Arial");
        self.canvas_context
            .set_fill_style(&JsValue::from_str("#000000"));
        self.canvas_context
            .fill_text(&format!("スコア: {}", self.score), 8.0, 20.0)
            .unwrap();
    }

    // タイムリミット描画
    fn draw_counter(&self) {
        self.canvas_context.set_font("16px Arial");
        self.canvas_context
            .set_fill_style(&JsValue::from_str("#000000"));
        self.canvas_context
            .fill_text(&format!("残り時間: {}", self.counter), self.canvas_width as f64 - 120.0, 20.0)
            .unwrap();
    }         

    fn collision_detection(&mut self) {
    }    
}

fn random_integer(length: f64) -> i32 {
    let random_length = 2.0 * length * js_sys::Math::random();
    (random_length - length).ceil() as i32
}

fn new_image() -> Result<HtmlImageElement> {
    HtmlImageElement::new().map_err(|err| anyhow!("Could not create HtmlImageElement: {:#?}", err))
}

fn closure_once<F, A, R>(fn_once: F) -> Closure<F::FnMut>
where
    F: 'static + WasmClosureFnOnce<A, R>,
{
    Closure::once(fn_once)
}

async fn load_image(source: &str) -> Result<HtmlImageElement> {
    let image = new_image()?;

    let (complete_tx, complete_rx) = channel::<Result<()>>();
    let success_tx = Rc::new(Mutex::new(Some(complete_tx)));
    let error_tx = Rc::clone(&success_tx);
    let success_callback = closure_once(move || {
        if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
            if let Err(err) = success_tx.send(Ok(())) {
                error!("Could not send successful image loaded message! {:#?}", err);
            }
        }
    });

    let error_callback: Closure<dyn FnMut(JsValue)> = closure_once(move |err| {
        if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
            if let Err(err) = error_tx.send(Err(anyhow!("Error Loading Image: {:#?}", err))) {
                error!("Could not send error message on loading image! {:#?}", err);
            }
        }
    });

    image.set_onload(Some(success_callback.as_ref().unchecked_ref()));
    image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
    image.set_src(source);

    complete_rx.await??;

    Ok(image)
}

#[wasm_bindgen]
pub async fn run() {
    let game = Game::new().await;
    let game = Rc::new(RefCell::new(game));
    Game::set_game_loop_and_start(game.clone());
    Game::set_input_event(game.clone());    
}
