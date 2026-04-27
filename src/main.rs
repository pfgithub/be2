use std::hash::Hash;

fn main() {
    println!("Hello, world!");
}

struct Reactive<T: PartialEq + Hash> {
    
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

*/