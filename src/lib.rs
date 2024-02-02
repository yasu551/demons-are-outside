use wasm_bindgen::prelude::*;
use web_sys;

#[wasm_bindgen]
pub fn run() {
    let document = web_sys::window().unwrap().document().unwrap();

    // JavaScriptでは… const canvas = document.getElementById("myCanvas");
    let canvas = document.get_element_by_id("myCanvas").unwrap();
    let canvas = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    // JavaScriptでは… const ctx = canvas.getContext("2d");
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    // JavaScriptでは…
    // ctx.beginPath();
    // ctx.rect(20, 40, 50, 50);
    // ctx.fillStyle = "#FF0000";
    // ctx.fill();
    // ctx.closePath();
    ctx.begin_path();
    ctx.rect(20.0, 40.0, 50.0, 50.0);
    ctx.set_fill_style(&JsValue::from_str("#FF0000"));
    ctx.fill();
    ctx.close_path();
}
