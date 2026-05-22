use eframe::egui;
fn main() -> std::result::Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(be2_eframe::MyEguiApp::new(cc)))),
    )
}

mod be2_eframe {
    use super::*;
    use retained_ui::*;
    use std::any::{Any, TypeId};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use util::*;

    pub struct MyEguiApp {
        root: Rc<RefCell<dyn Component1>>,
    }

    pub trait EguiRenderable {
        fn render(&mut self, ui: &mut eframe::egui::Ui) -> () {}
    }

    impl EguiRenderable for HVList {}
    impl EguiRenderable for ZStack {}
    impl EguiRenderable for Clickable {}
    impl EguiRenderable for Focusable {}
    impl EguiRenderable for Scrollable {}
    impl EguiRenderable for Text {
        fn render(&mut self, ui: &mut eframe::egui::Ui) -> () {
            ui.heading(self.message.clone());
        }
    }

    // we don't really need it to be traits in this case. we can register render functions themselves
    // probably would save a level of indirection
    type RenderCaster = fn(&mut dyn Any) -> Option<&mut dyn EguiRenderable>;
    pub struct EguiRenderableRegistry {
        casters: HashMap<TypeId, RenderCaster>,
    }
    impl EguiRenderableRegistry {
        pub fn new() -> Self {
            Self {
                casters: HashMap::<TypeId, RenderCaster>::new(),
            }
        }
        pub fn register<T: EguiRenderable + Any>(&mut self) {
            self.casters.insert(TypeId::of::<T>(), |any| {
                any.downcast_mut::<T>()
                    .map(|c| c as &mut dyn EguiRenderable)
            });
        }

        pub fn resolve<'a>(
            &self,
            comp: &'a mut dyn Component1,
        ) -> Option<&'a mut dyn EguiRenderable> {
            self.casters
                .get(&comp.as_any().type_id())
                .and_then(|caster| caster(comp.as_any_mut()))
        }
    }

    impl From<eframe::egui::Vec2> for Vector<2, f32> {
        fn from(value: eframe::egui::Vec2) -> Self {
            Self::from_main_other(Axis2::X, value.x, value.y)
        }
    }
    impl From<eframe::egui::Vec2> for Vector<2, Option<f32>> {
        fn from(value: eframe::egui::Vec2) -> Self {
            Self::from_main_other(Axis2::X, Some(value.x), Some(value.y))
        }
    }

    impl MyEguiApp {
        pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
            // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
            // Restore app state using cc.storage (requires the "persistence" feature).
            // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
            // for e.g. egui::PaintCallback.

            // let root = HVList::new(
            //     Axis2::Y,
            //     vec![ListItem {
            //         size: ListItemSize::Fit,
            //         item: HVList::new(Axis2::X, vec![]),
            //     }],
            // );
            let root = Text::new("abc".to_string());

            Self { root }
        }
    }

    impl eframe::App for MyEguiApp {
        fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show_inside(ui, |ui| {
                let avails = ui.available_size();

                let mut root = self.root.borrow_mut();
                root.size(ComponentParentSize {
                    size: avails.into(),
                });

                let mut registry = EguiRenderableRegistry::new();
                registry.register::<HVList>();
                registry.register::<ZStack>();
                registry.register::<Clickable>();
                registry.register::<Focusable>();
                registry.register::<Scrollable>();
                registry.register::<Text>();
                if let Some(renderable) = registry.resolve(&mut *root) {
                    renderable.render(ui);
                }
            });
        }
    }
}

mod component {
    struct ComponentDB {
        descriptor: &'static ComponentDBDescriptor,
    }
    struct ComponentDBDescriptor {}

    // table operations: splice, move
    //   splice (start, delete_count, insert_items)
    //   move (src, len, dst)
    // multiple operations can always be combined into one
    // an editable string is just a string table that you operate on with splice and move
}

mod reactive {
    use std::{hash::Hash, ops::Deref};

    struct Reactive<T> {
        current: T,
    }
    impl<T: Copy> Reactive<T> {
        fn new(value: T) -> Reactive<T> {
            Reactive::<T> { current: value }
        }
        fn update(&mut self, value: T) {
            self.current = value;
        }
        fn view(&self, viewer: ReactiveViewer) -> T {
            // needs to register
            self.current
        }
    }
    impl<T: Copy> Deref for Reactive<T> {
        type Target = T;
        fn deref(&self) -> &T {
            &self.current
        }
    }
    struct ReactiveViewer {}

    mod tests {
        use super::*;

        fn test_one() -> () {
            let mut counter = Reactive::<f64>::new(0.0);
            counter.update(1.0);
            let viewer = ReactiveViewer {};
            println!("counter is: {}", counter.view(viewer));
            // viewer.check(); // false
            counter.update(2.0);
            // viewer.check(); // true
            println!("counter is: {}", *counter);
        }
    }
}

struct DatabaseComponent {
    // this will depend on the db you define
    // ideally we want it like dbl but it can be more manual, less automated
}

// a block typically contains a DatabaseComponent
// ui is retained, solidjs-like

/*
ui example

let count = signal(0);

// count.set(25); // this will mark a request for a ui rerender
// when the ui actually rerenders, it will get the value of count

VList( reactive_static [
    Fit( HList( reactive_static [
        Text("abc")
    ] ) )
    Fit(Button(Text(reactive_static("++"))))
    Fit(Text(reactive(count)))
    Fill()
] )

// internally, ui is retained. so VList has addChild, removeChild, ?moveChild
// when the reactive list updates, it automatically makes those calls

*/

mod retained_ui {
    use std::{
        cell::RefCell,
        rc::{Rc, Weak},
    };

    use super::*;
    use util::*;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_one() -> () {
            let root = HVList::new(
                Axis2::Y,
                vec![ListItem {
                    size: ListItemSize::Fit,
                    item: HVList::new(Axis2::X, vec![]),
                }],
            );
            root.borrow_mut().size(ComponentParentSize {
                size: Vector::<2, Option<f32>> {
                    items: [Some(1.0), Some(2.0)],
                },
            });
            // root.borrow_mut().render();
        }
    }
    // button component:
    // - click handler
    // - focus handler
    // mouse event handling:
    // - components post their event handlers into a big list, cut as needed by clip components
    // - then we search that list to find the targets and trigger them

    // a component sized with the parent specifying the container boundary sizes
    // we could have others for one where it needs to know its size eg
    pub trait Sizable1 {
        fn size(&mut self, parent_size: ComponentParentSize) -> ComponentResolvedSize {
            todo!();
        }
    }
    pub trait Component1: Sizable1 {
        fn get_component<'a>(&'a mut self) -> &'a mut Component;
        fn as_any(&self) -> &dyn std::any::Any;
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    }

    #[derive(Copy, Clone)]
    pub struct ComponentParentSize {
        pub size: Vector<2, Option<f32>>, // alternatively, use 'infinity' for f32
    }
    pub struct ComponentResolvedSize {
        pub size: Vector<2, f32>,
    }

    pub struct Component {
        parent: Option<Weak<RefCell<dyn Component1>>>,

        want_resize: bool,
        want_rerender: bool,
    }
    impl Component {
        fn new() -> Component {
            Component {
                parent: None,
                want_resize: true,
                want_rerender: true,
            }
        }
        fn request_resize(&mut self) -> () {
            self.want_resize = true;
            if let Some(parent_weak) = &self.parent {
                if let Some(parent) = parent_weak.upgrade() {
                    parent.borrow_mut().get_component().request_resize();
                }
            }
        }
        fn request_rerender(&mut self) -> () {
            self.want_rerender = true;
            if let Some(parent_weak) = &self.parent {
                if let Some(parent) = parent_weak.upgrade() {
                    parent.borrow_mut().get_component().request_resize();
                }
            }
        }
    }

    pub struct Text {
        component: Component,
        pub message: String,
    }
    impl Text {
        pub fn new(message: String) -> Rc<RefCell<Text>> {
            Rc::new(RefCell::new(Text {
                component: Component::new(),
                message,
            }))
        }
        pub fn set_text(self: &mut Text, message: String) -> () {
            self.message = message;
            self.get_component().request_resize();
        }
    }
    impl Sizable1 for Text {
        fn size(&mut self, parent_size: ComponentParentSize) -> ComponentResolvedSize {
            // TODO:
            // - hb layout
            // - word wrap
            // - etc
            ComponentResolvedSize {
                size: Vector::<2, f32>::from_main_other(
                    Axis2::X,
                    (self.message.len() as f32) * 5.0,
                    20.0,
                ),
            }
        }
    }
    impl Component1 for Text {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    pub struct HVList {
        component: Component,

        direction: Axis2, // it should be called axis or otherwise it should have all four directions (right, down, left, up)
        items: Vec<ListItem>,
    }
    impl HVList {
        pub fn new(direction: Axis2, children: Vec<ListItem>) -> Rc<RefCell<HVList>> {
            Rc::new(RefCell::new(HVList {
                component: Component::new(),
                direction,
                items: children,
            }))
        }
        pub fn add_item(&mut self, item: ListItem) {
            self.items.push(item);
            self.get_component().request_resize();
        }
    }
    impl Sizable1 for HVList {
        fn size(&mut self, parent_size: ComponentParentSize) -> ComponentResolvedSize {
            // require on the non-main axis
            assert_ne!(parent_size.size[self.direction.inverse()], None);

            // 1. size Fit items
            let mut taken: f32 = 0.0;
            let mut fill_count: f32 = 0.0;
            let mut have_fill = false;
            for item in &self.items {
                match item.size {
                    ListItemSize::Fit => {
                        let resolved = item.item.borrow_mut().size(parent_size);
                        taken += resolved.size[self.direction];
                    }
                    ListItemSize::Fill(portion) => {
                        fill_count += portion;
                        have_fill = true;
                    }
                }
            }
            // 2. size Fill items in remaining space (assert parent_size[axis] is defined)
            if have_fill {
                if let Some(total) = parent_size.size[self.direction] {
                    let remaining = total - taken;
                    let divided = remaining / fill_count;
                    for item in &self.items {
                        match item.size {
                            ListItemSize::Fit => {}
                            ListItemSize::Fill(portion) => {
                                let self_size = divided / portion;
                                // to be pixel perfect, we can make the vars mut and then subtract from them each iteration
                                // that way we can always round up
                                let mut rsize = parent_size;
                                rsize.size[self.direction] = Some(self_size);
                                let _resolved = item.item.borrow_mut().size(rsize);
                            }
                        }
                    }
                    ComponentResolvedSize {
                        size: Vector::<2, f32>::from_main_other(
                            self.direction,
                            parent_size.size[self.direction].expect("oops"),
                            parent_size.size[self.direction.inverse()].expect("oops"),
                        ),
                    }
                } else {
                    panic!(
                        "not allowed ListItemSize::Fill inside container that does not specify maximum size"
                    );
                    // TODO: would be nice to enforce this via component types (eg sized1 vs sized2)
                }
            } else {
                ComponentResolvedSize {
                    size: Vector::<2, f32>::from_main_other(
                        self.direction,
                        taken,
                        parent_size.size[self.direction.inverse()].expect("oops"),
                    ),
                }
            }
        }
    }
    impl Component1 for HVList {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    pub struct ZStack {
        component: Component,

        items: Vec<ZStackItem>,
    }
    impl Sizable1 for ZStack {}
    impl Component1 for ZStack {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    pub struct ZStackItem {
        component: Component,

        value: Rc<RefCell<dyn Component1>>,
        sizing: ZStackItemSizing,
    }
    pub enum ZStackItemSizing {
        Leader,
        Follower,
    }
    pub struct Clickable {
        component: Component,

        // takes up the full size. use a vstack.
        mask: ClickableMask,
        callback: Box<dyn FnMut(MouseEvent)>,
    }
    impl Sizable1 for Clickable {}
    impl Component1 for Clickable {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    pub struct Focusable {
        component: Component,

        callback: Box<dyn FnMut(FocusEvent)>,
    }
    impl Sizable1 for Focusable {}
    impl Component1 for Focusable {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    pub struct Scrollable {
        component: Component,

        callback: Box<dyn FnMut(ScrollEvent)>,
    }
    impl Sizable1 for Scrollable {}
    impl Component1 for Scrollable {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    pub struct ScrollEvent {
        amount: Vector<2, f32>,
    }
    pub struct Color {}
    pub struct ClickableMask {
        left: bool,
        middle: bool,
        right: bool,
        // consider capture/bubble options
    }
    pub struct MouseEvent {
        pos: Vector<2, f32>,
        buttons: ClickableMask,
    }
    pub struct FocusEvent {
        mode: FocusEventMode,
    }
    pub enum FocusEventMode {
        Enter,
        Leave,
    }
    pub struct ListItem {
        pub size: ListItemSize,
        pub item: Rc<RefCell<dyn Component1>>,
    }
    pub enum ListItemSize {
        Fill(f32),
        Fit,
    }
}

mod blocks {
    use eframe::egui::accesskit::Uuid;

    use super::*;
    // server:
    // requests:
    // - read_and_watch(uuid) -> {data: []const u8, refs: []uuid}
    // replies:
    // - full_contents (uuid, data, refs)
    // -

    // what is a block?
    // - a block has a uuid, a format uuid, and data Vec<u8>
    // - you can submit updates to a block by sending an update request
    //   - the format field defines how this is applied
    //   - if you try to submit an update with an update number <= the server's update number, your update will be rejected and needs to be resent
    //     - your client needs to rebase the updates
    //   - TODO: be able to determine if an update has already been applied so if you send an update and never get a response, you won't send it again except in exceptional cases
    // - you receive updates

    pub struct BlockID {
        value: u128,
    }
    impl BlockID {
        fn uuid(u: u128) -> BlockID {
            BlockID { value: u }
        }
        fn format_component1() -> BlockID {
            BlockID::uuid(0x887cad84_a455_4a62_8f73_5d766e702364)
        }
        fn format_session1() -> BlockID {
            BlockID::uuid(0xc1d01fa8_50fe_4297_8bec_1a9ed2a826b4)
        }
    }
    pub struct Block {
        uuid: BlockID,
        format: BlockID, // this has format: BlockID::format_component1()
        data: Vec<u8>,
        refs: Vec<BlockID>,
        presence: Vec<BlockID>, // these have format: BlockID::format_session1()
    }

    pub enum Request {
        Read(RequestRead),
        Unwatch(RequestUnwatch),
        Create(RequestCreate),
        Update(RequestUpdate),
        History(RequestHistory),
        SubmitSnapshot(RequestSubmitSnapshot),
        BroadcastPresence(RequestBroadcastPresence),
    }
    pub struct RequestRead {
        uuid: BlockID,

        data_offset: u64,
        data_len: u64,
        watch: bool,
    }
    pub struct RequestUnwatch {
        uuid: BlockID,
    }
    pub struct RequestCreate {
        uuid: BlockID,

        format: BlockID,
        data: Vec<u8>,
    }
    pub struct RequestUpdate {
        uuid: BlockID,
        update_number: u64,
        update: Vec<u8>,
    }
    pub struct RequestHistory {
        uuid: BlockID,
        start_update_number: u64,
        len: u64,
    }
    pub struct RequestSubmitSnapshot {
        uuid: BlockID,
        update_number: u64,
        data: Vec<u8>,
    }
    pub struct RequestBroadcastPresence {
        uuid: BlockID,
        data: Vec<u8>,
    }

    pub enum Response {
        Create(ResponseCreateBlock),
        Read(ResponseReadBlock),
        Update(ResponseUpdate),
    }
    pub struct ResponseError {
        message: String,
    }
    pub struct ResponseCreateBlock {
        uuid: BlockID,
        status: Result<(), ResponseError>,
    }
    pub struct ResponseReadBlock {
        uuid: BlockID,
        update_number: u64,

        format: BlockID,
        data: Vec<u8>,
        data_total_len: u64,

        presence: Vec<u8>,
    }
    pub struct ResponseUpdate {
        uuid: BlockID,
        update_number: u64,

        update: Vec<u8>,
    }
    pub struct ResponseBroadcastPresence {
        uuid: BlockID,
        update: Vec<u8>,
    }

    mod client {
        use super::*;

        struct BlocksClient {}
        impl BlocksClient {
            // new() -> BlocksClient
            // connect()
            // - watch any server blocks we have
            // - upload any new blocks we created
            // - upload any unapplied updates we have
            // - if their update numbers are past ours, we will need to read history
            // disconnect()
            // observable connectionStatus
            //
            // createBlock(type: uuid) -> uuid
        }
    }
    mod server {
        use super::*;
    }

    mod block_types {
        use super::*;
        // excalidraw
        mod lockable {
            use super::*;

            fn format_lockable1() -> BlockID {
                BlockID::uuid(0x40b882fd_8e5d_47cf_a462_70ab3f99865b)
            }

            pub struct Lockable1 {
                // offline:
                // - warn when offline for lockables, or when going from online to offline
                // excalidraw usage example:
                // - the format field will be for a specific version of excalidraw. it will reference blocks containing the files of that version of excalidraw.
                // - if owned: if session: connect to the session. else: request a session
                // - not owned: set owned
                // blender usage example:
                // - if owned: warn about session stealing. use RequestSession as a soft takeover request
                // - not owned: set owned
                // video game usage example:
                // - the format field will say the game & version. maybe it can also reference blocks containing the game data in some cases.
                // - if owned: is the game multiplayer? join the game. else: prompt to request takeover
                // - not owned: set owned
                format: BlockID, // since this contains opaque data, we need to know the format again (.format lockable -> .format excalidraw -> data)
                data: Vec<u8>,   // structure is determined by the Lockable.format
            }
            pub enum Lockable1Update {
                Patch(Lockable1UpdatePatch),
            }
            struct Lockable1UpdatePatch {
                offset: u64,
                data: Vec<u8>,
            }

            pub enum Lockable1Presence {
                AnnounceOwnership(Lockable1PresenceAnnounceOwnership),
                RequestSession(Lockable1PresenceRequestSession), // ask for a session to be created, or when that can't be done, ask for the file to be unlocked
                AnnounceSession(Lockable1PresenceAnnounceSession), // we are the owner and someone asked for a session. it will take a moment for the session to start, let them know we're online.
            }
            struct Lockable1PresenceAnnounceOwnership {}
            struct Lockable1PresenceRequestSession {}
            struct Lockable1PresenceAnnounceSession {
                session: Vec<u8>,
            }
        }
    }

    // some basic block types:
    // - lockable:
    //   - you can lock and unlock it. this is for anything not integrated with the block system, ie blender or excalidraw
    //     - for excalidraw, when you lock the file you can have the ability to request starting a live collaboration session
    //     - so someone who opens the file will post a 'request_live_collaboration' update to the block and then get back
    //       a 'live_collaboration_session' response with a url. then their client will connect.
    //  - when offline these will need a warning because in bad-network or offline it's pretty easy to have two people lock
    //    the same file at once. resolving is a "choose which one to keep" which sucks.
}

mod util {
    use std::ops::{AddAssign, Index, IndexMut};

    #[derive(Copy, Clone)]
    pub enum Axis2 {
        X = 0,
        Y = 1,
    }
    impl Axis2 {
        pub fn inverse(self) -> Axis2 {
            // mod((self as usize) + 1, 2)
            match self {
                Axis2::X => Axis2::Y,
                Axis2::Y => Axis2::X,
            }
        }
    }

    #[derive(Copy, Clone)]
    pub struct Vector<const N: usize, T> {
        pub items: [T; N],
    }
    impl<const N: usize, T: AddAssign + Copy> AddAssign for Vector<N, T> {
        fn add_assign(&mut self, rhs: Self) {
            for i in 0..N {
                self.items[i] += rhs.items[i];
            }
        }
    }
    impl<T: Copy> Vector<2, T> {
        pub fn from_main_other(axis: Axis2, main: T, other: T) -> Vector<2, T> {
            match axis {
                Axis2::X => Vector::<2, T> {
                    items: [main, other],
                },
                Axis2::Y => Vector::<2, T> {
                    items: [other, main],
                },
            }
        }
    }
    impl<T: Copy> Index<Axis2> for Vector<2, T> {
        type Output = T;
        fn index(&self, axis: Axis2) -> &Self::Output {
            &self.items[axis as usize]
        }
    }
    impl<T: Copy> IndexMut<Axis2> for Vector<2, T> {
        fn index_mut(&mut self, axis: Axis2) -> &mut Self::Output {
            &mut self.items[axis as usize]
        }
    }
}
