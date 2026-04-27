fn main() {
    println!("Hello, world!");
}

mod reactive {
    use std::hash::Hash;

    struct Reactive<T: PartialEq + Hash> {
        current: T,
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
    use super::*;

    trait Component {}

    struct Scrollable {
        child: Box<dyn Component>,
    }
    struct HVList {
        direction: HV,
        items: Vec<ListItem>,
    }
    struct ZStack {
        items: Vec<Box<dyn Component>>,
    }
    struct Clickable {
        // takes up the full size. use a vstack.
        mask: ClickableMask,
        callback: Box<dyn FnMut(MouseEvent)>,
    }
    struct Focusable {
        callback: Box<dyn FnMut(FocusEvent)>,
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
        pos: util::Vector<2, f32>,
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
        item: Box<dyn Component>,
    }
    enum ListItemSize {
        Fill,
        Fit,
    }
    enum HV {
        H,
        V,
    }
}

mod util {
    use std::ops::AddAssign;

    pub struct Vector<const N: usize, T> {
        items: [T; N],
    }
    impl<const N: usize, T: AddAssign + Copy> AddAssign for Vector<N, T> {
        fn add_assign(&mut self, rhs: Self) {
            for i in 0..N {
                self.items[i] += rhs.items[i];
            }
        }
    }
}
