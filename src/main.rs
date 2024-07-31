use rand::Rng;

mod hnsw;

use crate::hnsw::{Graph, knn_search, cosine_distance};

fn main() {
    println!("Lets make a vector database!");
    let q = [1.0, 2.0, 3.0, 4.0];
    let mut g = Box::new(Graph::new(&q, 5.0, 5, 10, 20));

    let mut rng = rand::thread_rng();

    for     _i in 0..50_000 {
        let vec: [f64; 4] = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
        //let vec: [f64; 4] = [rng.gen_range(0..10), rng.gen_range(0..10), rng.gen_range(0..10), rng.gen_range(0..10)];
        g.insert(
            &vec,
        );
    }
    //println!("heres the graph:");
    //print_graph(&g);
    println!("{} layers wow!!", g.layer_count);

    println!("lets try a search!");
    //let vec: [f64; 4] = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
    //let vec = g.nodes.last().unwrap().borrow().vector.clone();
    //let vec: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
    let vec = g.nodes.first().unwrap().borrow().vector.clone();

    print!("[");
    vec.iter().for_each(|x| print!("{}, ", x));
    print!("]\n");

    println!("\nthinking...");
    let search = knn_search(&g, &vec, 5, 20);

    search.iter().for_each(|x| {
        print!("{}, {}: ", x.borrow().index, cosine_distance(&vec, &x.borrow().vector));
        print!("[");
        x.borrow().vector.iter().for_each(|v| print!("{v}, "));
        print!("]\n");
    });

    println!("\n");
}
