mod runtime;
use runtime::{Executor, Task};

fn main() {
    let ex = Executor::new();

    let t = Task::new(async {
        println!("Hello from custom runtime!");
    });

    ex.spawn(t);
    ex.run();
}
