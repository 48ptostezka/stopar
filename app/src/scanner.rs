//! Camera scanning: opens the back camera, samples a frame every ~120 ms,
//! and decodes QR codes with `stopa_core::qr` (pure Rust, no JS library).

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use js_sys::{Object, Reflect};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement, MediaStream, MediaStreamTrack};

use crate::platform;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraStatus {
    Starting,
    Scanning,
    /// The player or the browser refused camera access.
    Denied,
    /// The device has no usable camera.
    Missing,
    /// Page opened over plain http; browsers only allow cameras over https.
    Insecure,
    /// Anything else, e.g. another app is using the camera.
    Failed,
}

/// Time between decoded frames.
const FRAME_MS: i32 = 120;
/// Frames are scaled down to at most this many pixels square before decoding.
const SAMPLE_SIZE: u32 = 480;
/// Share of the shorter video side that is decoded, matching the viewfinder.
const CROP: f64 = 0.8;
/// The same code is reported again only after this pause.
const REPEAT_MS: f64 = 2500.0;

/// A running scanner. Dropping it turns the camera off.
pub struct Scanner {
    inner: Rc<Inner>,
}

struct Inner {
    running: Cell<bool>,
    video: HtmlVideoElement,
    stream: RefCell<Option<MediaStream>>,
    sampler: Option<Sampler>,
    on_status: Box<dyn Fn(CameraStatus)>,
    /// Returns true to stop scanning (a code was accepted).
    on_code: Box<dyn Fn(&str) -> bool>,
    last_code: RefCell<(String, f64)>,
}

impl Scanner {
    pub fn start(
        video: HtmlVideoElement,
        on_status: impl Fn(CameraStatus) + 'static,
        on_code: impl Fn(&str) -> bool + 'static,
    ) -> Scanner {
        let sampler = Sampler::new();
        let inner = Rc::new(Inner {
            running: Cell::new(sampler.is_some()),
            video,
            stream: RefCell::new(None),
            sampler,
            on_status: Box::new(on_status),
            on_code: Box::new(on_code),
            last_code: RefCell::new((String::new(), 0.0)),
        });
        if !inner.running.get() {
            (inner.on_status)(CameraStatus::Failed);
            return Scanner { inner };
        }
        let task = inner.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match open_camera().await {
                Ok(stream) => {
                    if !task.running.get() {
                        stop_tracks(&stream);
                        return;
                    }
                    task.video.set_src_object(Some(&stream));
                    *task.stream.borrow_mut() = Some(stream);
                    if let Ok(playing) = task.video.play() {
                        let _ = JsFuture::from(playing).await;
                    }
                    if task.running.get() {
                        (task.on_status)(CameraStatus::Scanning);
                        schedule(task);
                    }
                }
                Err(status) => {
                    if task.running.get() {
                        (task.on_status)(status);
                    }
                }
            }
        });
        Scanner { inner }
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        self.inner.stop();
    }
}

impl Inner {
    fn stop(&self) {
        self.running.set(false);
        if let Some(stream) = self.stream.borrow_mut().take() {
            stop_tracks(&stream);
            self.video.set_src_object(None);
        }
    }
}

/// An off-screen canvas that video frames are copied into for decoding.
struct Sampler {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl Sampler {
    fn new() -> Option<Sampler> {
        let canvas: HtmlCanvasElement =
            platform::document().create_element("canvas").ok()?.dyn_into().ok()?;
        // Tells the browser we read pixels back often, so it keeps the canvas in memory.
        let options = Object::new();
        Reflect::set(&options, &"willReadFrequently".into(), &JsValue::TRUE).ok()?;
        let context = canvas.get_context_with_context_options("2d", &options).ok()??.dyn_into().ok()?;
        Some(Sampler { canvas, context })
    }
}

async fn open_camera() -> Result<MediaStream, CameraStatus> {
    let window = platform::window();
    if !window.is_secure_context() {
        return Err(CameraStatus::Insecure);
    }
    let navigator = window.navigator();
    if !platform::has_property(&navigator, "mediaDevices") {
        return Err(CameraStatus::Missing);
    }
    let devices = navigator.media_devices().map_err(|_| CameraStatus::Missing)?;

    // { audio: false, video: { facingMode: { ideal: "environment" }, width: { ideal: 1280 } } }
    let ideal = |value: JsValue| {
        let o = Object::new();
        let _ = Reflect::set(&o, &"ideal".into(), &value);
        o
    };
    let video = Object::new();
    let _ = Reflect::set(&video, &"facingMode".into(), &ideal("environment".into()));
    let _ = Reflect::set(&video, &"width".into(), &ideal(1280.into()));
    let constraints = Object::new();
    let _ = Reflect::set(&constraints, &"audio".into(), &JsValue::FALSE);
    let _ = Reflect::set(&constraints, &"video".into(), &video);

    let request =
        devices.get_user_media_with_constraints(constraints.unchecked_ref()).map_err(|e| classify(&e))?;
    let stream = JsFuture::from(request).await.map_err(|e| classify(&e))?;
    Ok(stream.unchecked_into())
}

/// Turns a getUserMedia error into something we can explain to the player.
fn classify(error: &JsValue) -> CameraStatus {
    let name = Reflect::get(error, &"name".into()).ok().and_then(|n| n.as_string()).unwrap_or_default();
    match name.as_str() {
        "NotAllowedError" | "SecurityError" | "PermissionDeniedError" => CameraStatus::Denied,
        "NotFoundError" | "OverconstrainedError" | "DevicesNotFoundError" => CameraStatus::Missing,
        _ => CameraStatus::Failed,
    }
}

fn stop_tracks(stream: &MediaStream) {
    for track in stream.get_tracks().iter() {
        track.unchecked_into::<MediaStreamTrack>().stop();
    }
}

fn schedule(inner: Rc<Inner>) {
    let callback = Closure::once_into_js(move || tick(inner));
    let _ = platform::window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), FRAME_MS);
}

fn tick(inner: Rc<Inner>) {
    if !inner.running.get() {
        return;
    }
    if let Some(text) = read_frame(&inner) {
        let now = js_sys::Date::now();
        let repeat = {
            let (last, at) = &*inner.last_code.borrow();
            *last == text && now - at < REPEAT_MS
        };
        if !repeat {
            *inner.last_code.borrow_mut() = (text.clone(), now);
            if (inner.on_code)(&text) {
                inner.stop();
                return;
            }
        }
    }
    schedule(inner);
}

/// Copies the centre of the current video frame into the canvas and decodes it.
fn read_frame(inner: &Inner) -> Option<String> {
    let Sampler { canvas, context } = inner.sampler.as_ref()?;
    let video = &inner.video;
    const HAVE_CURRENT_DATA: u16 = 2;
    if video.ready_state() < HAVE_CURRENT_DATA {
        return None;
    }
    let (w, h) = (video.video_width() as f64, video.video_height() as f64);
    if w == 0.0 || h == 0.0 {
        return None;
    }
    let side = w.min(h) * CROP;
    let size = SAMPLE_SIZE.min(side as u32).max(1);
    if canvas.width() != size {
        canvas.set_width(size);
        canvas.set_height(size);
    }
    let s = size as f64;
    context
        .draw_image_with_html_video_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            video,
            (w - side) / 2.0,
            (h - side) / 2.0,
            side,
            side,
            0.0,
            0.0,
            s,
            s,
        )
        .ok()?;
    let pixels = context.get_image_data(0.0, 0.0, s, s).ok()?.data();
    let luma = stopa_core::qr::rgba_to_luma(&pixels);
    stopa_core::qr::decode_luma(size as usize, size as usize, &luma)
}
