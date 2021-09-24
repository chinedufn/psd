#![feature(proc_macro_hygiene)]

use console_error_panic_hook;

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::*;

use css_rs_macro::css;
use virtual_dom_rs::prelude::*;

use psd::Psd;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

/// Wraps our application so that we can return it to the caller of this WebAssembly module.
/// This ensures that our closures that we're holding on to in the App struct don't get dropped.
///
/// If we we didn't do this our closures would get dropped and wouldn't work.
#[wasm_bindgen]
struct AppWrapper(Rc<RefCell<App>>);

#[wasm_bindgen]
impl AppWrapper {
    /// Create a new AppWrapper. We'll call this in a script tag in index.html
    #[wasm_bindgen(constructor)]
    pub fn new() -> AppWrapper {
        console_error_panic_hook::set_once();

        let mut app = App::new();

        let closure_holder = Rc::clone(&app.raf_closure_holder);

        let store = Rc::clone(&app.store);

        let app = Rc::new(RefCell::new(app));
        let app_clone = Rc::clone(&app);

        // Whenever state gets updated we'll re-render the page
        {
            let on_msg = move || {
                let store = Rc::clone(&store);
                let app = Rc::clone(&app);
                let closure_holder = Rc::clone(&closure_holder);

                let re_render = move || {
                    let store = Rc::clone(&store);
                    let app = Rc::clone(&app);

                    let vdom = app.borrow().render();
                    app.borrow_mut().update(vdom);

                    store.borrow_mut().msg(&Msg::SetIsRendering(false));
                };
                let mut re_render = Closure::wrap(Box::new(re_render) as Box<FnMut()>);

                window().request_animation_frame(&re_render.as_ref().unchecked_ref());

                *closure_holder.borrow_mut() = Some(Box::new(re_render));
            };

            {
                let mut app = app_clone.borrow_mut();
                app.store.borrow_mut().on_msg = Some(Box::new(on_msg));
                app.start();
            }
        }

        AppWrapper(app_clone)
    }
}

/// Our client side web application
#[wasm_bindgen]
struct App {
    store: Rc<RefCell<Store>>,
    dom_updater: DomUpdater,
    /// Holds the most recent RAF closure
    raf_closure_holder: Rc<RefCell<Option<Box<dyn AsRef<JsValue>>>>>,
}

#[wasm_bindgen]
impl App {
    /// Create a new App
    fn new() -> App {
        let vdom = html! { <div> </div> };
        let mut dom_updater = DomUpdater::new_append_to_mount(vdom, &body());

        let state = State {
            psd: None,
            layer_visibility: HashMap::new(),
            is_rendering: false,
        };

        let on_msg = None;
        let store = Store { state, on_msg };
        let store = Rc::new(RefCell::new(store));

        App {
            store,
            dom_updater,
            raf_closure_holder: Rc::new(RefCell::new(None)),
        }
    }

    /// Start the demo
    fn start(&mut self) {
        let demo_psd = include_bytes!("../demo.psd");

        self.store.borrow_mut().msg(&Msg::ReplacePsd(demo_psd));

        let vdom = self.render();
        self.update(vdom);
    }

    /// Render the virtual-dom
    fn render(&self) -> VirtualNode {
        let store = &self.store;
        let store_clone = Rc::clone(store);

        let store = store.borrow();

        let psd = store.psd.as_ref().unwrap();

        let mut layers: Vec<VirtualNode> = psd
            .layers()
            .iter()
            .enumerate()
            .map(|(idx, layer)| {
                let store = Rc::clone(&store_clone);

                let checked = *store.borrow().layer_visibility.get(layer.name()).unwrap();

                let background_color_class = if checked {
                    "layer-dark-background"
                } else {
                    "layer-light-background"
                };

                let checked = if checked { "true" } else { "false" };

                let name = layer.name();

                html! {
                <div
                    style="cursor: pointer; margin-bottom: 8px; display: flex;"
                    class=background_color_class
                >
                  <label
                    style="cursor: pointer; padding-top: 15px; padding-bottom: 15px; padding-left: 5px; padding-right: 5px; display: block; width: 100%;"
                  >
                    <input
                     key=name
                     type="checkbox"
                     checked=checked
                     // TODO: make virtual-dom-rs allow for variables .. `onchange=onchange`
                     // To be able to move the callback outside of the html macro..
                     //
                     // If the attribute starts with `on` treat the value as a closure.
                     onchange=move |event: web_sys::Event| {
                       let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
                       let msg = Msg::SetLayerVisibility(idx, input.checked());
                       store.borrow_mut().msg(&msg);
                     }
                     />
                    <span style="cursor: pointer; margin-left: 5px;">{ name.to_string() }</span>
                  </label>
                </div>
                }
            })
            .collect();
        layers.reverse();

        let vdom = html! {
           <div class=APP_CONTAINER>

             <div class="left-column">
               <canvas id="psd-visual"></canvas>
               <div
                 style="height: 100px; display: flex; align-items: center; justify-content: center;"
                 ondragenter=|event: web_sys::DragEvent| {
                    event.prevent_default();
                    event.stop_propagation();
                 }
                 ondragover=|event: web_sys::DragEvent| {
                    event.prevent_default();
                    event.stop_propagation();
                 }
                 ondrop=move |event: web_sys::DragEvent| {
                    event.prevent_default();
                    event.stop_propagation();

                    let store = Rc::clone(&store_clone);

                    let dt = event.data_transfer().unwrap();
                    let files = dt.files().unwrap();
                    let psd = files.item(0).unwrap();

                    let file_reader = web_sys::FileReader::new().unwrap();
                    file_reader.read_as_array_buffer(&psd).unwrap();

                    let mut onload = Closure::wrap(Box::new(move |event: Event| {
                        let file_reader: FileReader = event.target().unwrap().dyn_into().unwrap();
                        let psd = file_reader.result().unwrap();
                        let psd = js_sys::Uint8Array::new(&psd);

                        let mut psd_file = vec![0; psd.length() as usize];
                        psd.copy_to(&mut psd_file);

                        store.borrow_mut().msg(&Msg::ReplacePsd(&psd_file));
                    }) as Box<FnMut(_)>);

                    file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                 }
               >
                 <strong>{ "Drag and drop here to upload a PSD" }</strong>
               </div>
             </div>

             <div class="right-column">
               <h2>Layers</h2>
               { layers }
             </div>
           </div>
        };

        vdom
    }

    /// Patch the DOM with a new virtual dom and update our Canvas' pixels
    fn update(&mut self, vdom: VirtualNode) -> Result<(), JsValue> {
        self.dom_updater.update(vdom);

        let psd = &self.store.borrow();
        let psd = &psd.psd;
        let psd = psd.as_ref().unwrap();

        // Flatten the PSD into only the pixels from the layers that are currently
        // toggled on.
        let mut psd_pixels = psd
            .flatten_layers_rgba(&|(idx, layer)| {
                let layer_visible = *self
                    .store
                    .borrow()
                    .layer_visibility
                    .get(layer.name())
                    .unwrap();

                layer_visible
            })
            .unwrap();

        let psd_pixels = Clamped(&mut psd_pixels[..]);
        let psd_pixels =
            ImageData::new_with_u8_clamped_array_and_sh(psd_pixels, psd.width(), psd.height())?;

        let canvas: HtmlCanvasElement = document()
            .get_element_by_id("psd-visual")
            .unwrap()
            .dyn_into()?;
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

        canvas.set_width(psd.width());
        canvas.set_height(psd.height());

        context.put_image_data(&psd_pixels, 0., 0.)?;

        Ok(())
    }
}

/// A light wrapper around State, useful when you want to accept a Msg and handle
/// anything impure (such as working with local storage) before passing the Msg
/// along the State. Allowing you to keep State pure.
struct Store {
    state: State,
    on_msg: Option<Box<dyn Fn()>>,
}

/// You'll usually just want the underlying State, so we Deref for convenience.
impl Deref for Store {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

/// Handles application state
struct State {
    /// The current PSD that is being displayed
    psd: Option<Psd>,
    /// Layer name -> whether or not it currently toggled on
    layer_visibility: HashMap<String, bool>,
    /// Whether or not we've already requested to render on the next animation frame
    is_rendering: bool,
}

impl Store {
    /// Send a new message to our Store, usually to update State
    fn msg(&mut self, msg: &Msg) {
        let is_rendering = self.state.is_rendering;

        self.state.msg(msg);

        if !is_rendering {
            self.state.msg(&Msg::SetIsRendering(true));
            self.on_msg.as_ref().unwrap()();
        }
    }
}

impl State {
    /// Update State given some new Msg
    fn msg(&mut self, msg: &Msg) {
        match msg {
            // Replace the current PSD with a new one
            // Happens on page load and after drag/drop
            Msg::ReplacePsd(psd) => {
                let psd = Psd::from_bytes(psd).unwrap();

                // When we upload a new PSD we set all layers to visible
                let mut layer_visibility = HashMap::new();
                for layer in psd.layers().iter() {
                    layer_visibility.insert(layer.name().to_string(), true);
                }

                self.psd = Some(psd);
                self.layer_visibility = layer_visibility;
            }
            // Set whether or not a layer is currently toggled on/off
            Msg::SetLayerVisibility(idx, visible) => {
                let visibility = self
                    .layer_visibility
                    .get_mut(
                        self.psd
                            .as_mut()
                            .unwrap()
                            .layer_by_idx(*idx)
                            .name(),
                    )
                    .unwrap();

                *visibility = *visible;
            }
            // Have we already queued up a re-render?
            Msg::SetIsRendering(is_rendering) => {
                self.is_rendering = *is_rendering;
            }
        }
    }
}

/// All of our Msg variants that are used to update application state
enum Msg<'a> {
    /// Replace the current PSD with a new one, usually after drag and drop
    ReplacePsd(&'a [u8]),
    /// Set whether or not a layer (by index) should be visible
    SetLayerVisibility(usize, bool),
    /// Set that the application is planning to render on the next request animation frame
    SetIsRendering(bool),
}

fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}

fn document() -> web_sys::Document {
    window().document().unwrap()
}

fn body() -> web_sys::HtmlElement {
    document().body().unwrap()
}

static APP_CONTAINER: &'static str = css! {r#"
:host {
    display: flex;
    width: 100%;
    height: 100%;
}
"#};

static _LAYOUT: &'static str = css! {r#"
.left-column {
}

.right-column {
    background-color: #f7f7f7;
    padding-left: 5px;
    padding-right: 5px;
}

.layer-dark-background {
   background-color: #b8b8b8;
}

.layer-light-background {
   background-color: #e0e0e0;
}
"#};

// Just like println! but works in the browser
//
// clog!("Hello world {}", some_variable);
#[macro_export]
macro_rules! clog {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
