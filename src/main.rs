#[allow(dead_code)]
#[allow(unused_variables)]
#[allow(unused_must_use)]
use rand::Rng;

mod hnsw;
use crate::hnsw::{cosine_distance, knn_search, Graph, Node};

mod file;
use crate::file::GraphFile;

fn main() {
    println!("Lets make a vector database!");
    let q = [1.0, 2.0, 3.0, 4.0];
    let mut g = Box::new(Graph::new(&q, 5.0, 5, 10, 20));

    _test_write_read(&mut g);
    _test_insert(&mut g);
}

fn _test_insert(g: &mut Graph) {
    let mut rng = rand::thread_rng();
    for _i in 0..10 {
        let vec: [f64; 4] = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
        //let vec: [f64; 4] = [rng.gen_range(0..10), rng.gen_range(0..10), rng.gen_range(0..10), rng.gen_range(0..10)];
        g.insert(&vec);
    }

    g.print();
}

fn _test_write_read(g: &mut Graph) {
    _test_graph(g);
    g.print();
    println!("writing to disk...");
    let mut gf = GraphFile::create("test.vite".into());
    gf.write(&g);

    let mut gf_new = GraphFile::open("test.vite".into());
    let g_new = gf_new.read().unwrap();

    g_new.print();
}

fn _test_file(g: &mut Graph) {
    g.print();
    println!("trying file writing....");
    let mut gf = GraphFile::create("test.vite".into());
    gf.write(&g);

    let mut new_gf = GraphFile::open("test.vite".into());
    let graph_bytes = new_gf.read().unwrap();
    //let new_grah = Graph::deserialize(&graph_bytes.as_ref().try_into().unwrap());
    //new_grah.print();
}

fn _test_graph(g: &mut Graph) {
    let mut rng = rand::thread_rng();

    for _i in 0..10 {
        let vec: [f64; 4] = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
        //let vec: [f64; 4] = [rng.gen_range(0..10), rng.gen_range(0..10), rng.gen_range(0..10), rng.gen_range(0..10)];
        g.insert(&vec);
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
        print!(
            "{}, {}: ",
            x.borrow().index,
            cosine_distance(&vec, &x.borrow().vector)
        );
        print!("[");
        x.borrow().vector.iter().for_each(|v| print!("{v}, "));
        print!("]\n");
    });

    println!("\n");
}
