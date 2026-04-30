fn main() {
    println!("Hello, world!");
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
            let root = HVList::new(Axis2::Y);
            root.borrow_mut()
                .add_item(ListItemSize::Fit, HVList::new(Axis2::X));
            root.borrow_mut().size(ComponentParentSize {
                size: Vector::<2, Option<f32>> {
                    items: [Some(1.0), Some(2.0)],
                },
            });
            root.borrow_mut().render();
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
    trait Sizable1 {
        fn size(&mut self, parent_size: ComponentParentSize) -> ComponentResolvedSize {
            todo!();
        }
    }
    trait EguiRenderable {
        fn render(&mut self) -> () {}
    }
    trait Component1: Sizable1 + EguiRenderable {
        fn get_component<'a>(&'a mut self) -> &'a mut Component;
    }

    #[derive(Copy, Clone)]
    struct ComponentParentSize {
        size: Vector<2, Option<f32>>,
    }
    struct ComponentResolvedSize {
        size: Vector<2, f32>,
    }

    struct Component {
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

    struct HVList {
        component: Component,

        direction: Axis2, // it should be called axis or otherwise it should have all four directions (right, down, left, up)
        items: Vec<ListItem>,
    }
    impl HVList {
        fn new(direction: Axis2) -> Rc<RefCell<HVList>> {
            Rc::new(RefCell::new(HVList {
                component: Component::new(),
                direction,
                items: vec![],
            }))
        }
        fn add_item(&mut self, size: ListItemSize, item: Rc<RefCell<dyn Component1>>) {
            self.items.push(ListItem { size, item });
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
    impl EguiRenderable for HVList {}
    impl Component1 for HVList {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
    }
    struct ZStack {
        component: Component,

        items: Vec<ZStackItem>,
    }
    impl Sizable1 for ZStack {}
    impl EguiRenderable for ZStack {}
    impl Component1 for ZStack {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
    }
    struct ZStackItem {
        component: Component,

        value: Rc<RefCell<dyn Component1>>,
        sizing: ZStackItemSizing,
    }
    enum ZStackItemSizing {
        Leader,
        Follower,
    }
    struct Clickable {
        component: Component,

        // takes up the full size. use a vstack.
        mask: ClickableMask,
        callback: Box<dyn FnMut(MouseEvent)>,
    }
    impl Sizable1 for Clickable {}
    impl EguiRenderable for Clickable {}
    impl Component1 for Clickable {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
    }
    struct Focusable {
        component: Component,

        callback: Box<dyn FnMut(FocusEvent)>,
    }
    impl Sizable1 for Focusable {}
    impl EguiRenderable for Focusable {}
    impl Component1 for Focusable {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
    }
    struct Scrollable {
        component: Component,

        callback: Box<dyn FnMut(ScrollEvent)>,
    }
    impl Sizable1 for Scrollable {}
    impl EguiRenderable for Scrollable {}
    impl Component1 for Scrollable {
        fn get_component<'a>(&'a mut self) -> &'a mut Component {
            &mut self.component
        }
    }
    struct ScrollEvent {
        amount: Vector<2, f32>,
    }
    struct Color {}
    struct Text {
        msg: String,
    }
    struct ClickableMask {
        left: bool,
        middle: bool,
        right: bool,
        // consider capture/bubble options
    }
    struct MouseEvent {
        pos: Vector<2, f32>,
        buttons: ClickableMask,
    }
    struct FocusEvent {
        mode: FocusEventMode,
    }
    enum FocusEventMode {
        Enter,
        Leave,
    }
    struct ListItem {
        size: ListItemSize,
        item: Rc<RefCell<dyn Component1>>,
    }
    enum ListItemSize {
        Fill(f32),
        Fit,
    }
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
