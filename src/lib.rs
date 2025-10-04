use std::os::raw::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};
use std::ptr;

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::{
    prelude::*,
    webview::{WebViewBuilder, WebViewAttributes},
};

#[repr(C)]
pub struct GuiHandle {
    // Placeholder dla handle - w rzeczywistości użyj Box lub raw ptr do struktury
    ptr: *mut c_void,
}

// Struktura wewnętrzna do przechowywania stanu
struct GuiState {
    event_loop: Option<EventLoop<()>>,
    window: Option<tao::window::Window>,
    webview: Option<WebView>,
}

unsafe impl Send for GuiState {}
unsafe impl Sync for GuiState {}

type StatePtr = Arc<Mutex<Option<GuiState>>>;

// Eksportowane funkcje C dla FFI (bindings dla Nula)
#[no_mangle]
pub extern "C" fn nula_gui_init(width: c_int, height: c_int) -> *mut c_void {
    let state = Arc::new(Mutex::new(None));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(tao::dpi::LogicalSize::new(width as f64, height as f64))
        .with_title("Nula App")
        .build(&event_loop)
        .expect("Failed to create window");

    let webview_attrs = WebViewAttributes::new(WryWindow::new(&window));
    let webview = WebViewBuilder::new(webview_attrs)
        .with_url("data:text/html,<html><body><h1>Hello from Nula GUI!</h1><script>console.log('JS ready');</script></body></html>")
        .with_ipc_handler(|message| {
            // Tauri-like bridge: obsługa wiadomości z JS
            println!("IPC from JS: {}", message);
            // Tutaj możesz wywołać funkcje z Nula lub wysłać dane
            // np. via global state lub callbacks
        })
        .build()
        .expect("Failed to create WebView");

    let mut guard = state.lock().unwrap();
    *guard = Some(GuiState {
        event_loop: Some(event_loop),
        window: Some(window),
        webview: Some(webview),
    });

    Arc::into_raw(state) as *mut c_void
}

#[no_mangle]
pub extern "C" fn nula_gui_load_html(handle: *mut c_void, html: *const c_char) {
    if handle.is_null() {
        return;
    }
    let state_ptr = handle as *mut StatePtr;
    let state = unsafe { &*state_ptr };
    let html_str = unsafe { CStr::from_ptr(html).to_str().unwrap_or("") };

    if let Ok(mut guard) = state.lock() {
        if let Some(ref mut gui) = *guard {
            if let Some(ref mut wv) = gui.webview {
                // Ładuj HTML - użyj data URL lub file
                let data_url = format!("data:text/html,{}", urlencoding::encode(html_str));
                wv.load_url(&data_url).expect("Failed to load HTML");
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn nula_gui_run(handle: *mut c_void) {
    if handle.is_null() {
        return;
    }
    let state_ptr = handle as *mut StatePtr;
    let state = unsafe { Arc::from_raw(state_ptr) };

    if let Ok(mut guard) = state.lock() {
        if let Some(gui) = guard.take() {
            let event_loop = gui.event_loop.unwrap();
            let window = gui.window.unwrap();
            let mut webview = gui.webview.unwrap();

            event_loop.run(move |event, elwt| {
                elwt.set_control_flow(ControlFlow::Wait);

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => elwt.exit(),
                    _ => {
                        webview.event(event);
                    }
                }
            });
        }
    }
}

#[no_mangle]
pub extern "C" fn nula_gui_destroy(handle: *mut c_void) {
    if !handle.is_null() {
        // Cleanup - drop state
        unsafe { ptr::drop_in_place(handle as *mut StatePtr) };
    }
}

// Dodatkowe funkcje, np. do komunikacji JS-Rust
#[no_mangle]
pub extern "C" fn nula_gui_emit_js(handle: *mut c_void, js_code: *const c_char) {
    if handle.is_null() {
        return;
    }
    let state_ptr = handle as *mut StatePtr;
    let state = unsafe { &*state_ptr };
    let js_str = unsafe { CStr::from_ptr(js_code).to_str().unwrap_or("") };

    if let Ok(guard) = state.lock() {
        if let Some(ref gui) = *guard {
            if let Some(ref wv) = gui.webview {
                wv.eval(js_str).expect("Failed to evaluate JS");
            }
        }
    }
}

// Uwaga: Dodaj urlencoding dependency jeśli potrzeba: urlencoding = "2.1"
