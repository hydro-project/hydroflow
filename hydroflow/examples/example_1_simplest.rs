//[use]//
use hydroflow::hydroflow_syntax;
//[/use]//

//[macro_call]//
pub fn main() {
    let mut flow = hydroflow_syntax! {
        source_iter(0..10) -> for_each(|n| println!("Hello {}", n));
    };
    //[/macro_call]//

    //[run]//
    flow.run_available();
    //[/run]//
}
