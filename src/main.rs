use log::{debug, error, info, log_enabled, Level};
#[allow(dead_code)]
#[allow(unused_variables)]
#[allow(unused_must_use)]
use rand::Rng;
use std::env;
use std::io;
use std::io::Write;

mod hnsw;
use crate::hnsw::{cosine_distance, knn_search, Graph};

mod file;
use crate::file::GraphFile;

macro_rules! flush {
    () => {
        io::stdout().flush().unwrap()
    };
}

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if log_enabled!(Level::Debug) {
        debug!("started with the following {} arguments:", args.len());
        let mut buff = String::new();
        buff.push_str("[");
        for arg in &args {
            buff.push_str(&format!("{}, ", arg))
        }
        buff.push_str("]");
        debug!("{}", buff);
    }

    if args.len() == 1 {
        error!("not enough arguments!");
        return;
    }

    // vite <filename>
    if args.len() == 2 {
        let filename = &args[1];
        info!("opening: {}", filename);
        let mut gf = GraphFile::open(filename.into());
        let g = gf.read().unwrap();

        g.print();
        interperter_loop(&g);
        return;
    }

    // vite <command> <filename>
    let mut g = GraphFile::open(args[2].clone().into())
        .read()
        .expect("Could not read graph");
    g.try_weaken_ep();
    match args[1].as_str() {
        "new" => {}
        "add" => {}
        "search" => {
            info!("search selected with: vector={} k={}", args[3], args[4]);
            search_vector(&g, args[3].as_str(), args[4].as_str())
        }
        _ => error!("invalid command"),
    }
}

/*
fn new_graph_wizard() -> Graph {

}
*/

fn search_vector(g: &Graph, q_str: &str, k_str: &str) {
    let q = parse_vector(q_str);
    let k = u64::from_str_radix(k_str, 10).unwrap();
    let search = knn_search(&g, &q, k.try_into().unwrap(), 20);

    for elem in search {
        print!("{}, ", elem.borrow().index)
    }
}

fn parse_vector(string: &str) -> Vec<f64> {
    return vec![1.0, 2.0, 3.0, 4.0];
}

fn interperter_loop(g: &Graph) {
    let mut input = String::new();
    print!("> ");
    flush!();
    {
        while let Ok(n_bytes) = io::stdin().read_line(&mut input) {
            if n_bytes == 0 {
                continue;
            }
            println!("*{}", input.trim());
            input.clear();
            print!("> ");
            flush!();
        }
    }
}

fn _test_search(g: &Graph) {
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

fn _test_insert(g: &mut Graph) {
    let mut rng = rand::thread_rng();
    for _i in 0..100 {
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
    gf.write(&g).unwrap();

    let mut gf_new = GraphFile::open("test.vite".into());
    let g_new = gf_new.read().unwrap();

    g_new.print();
}

fn _test_file(g: &mut Graph) {
    g.print();
    println!("trying file writing....");
    let mut gf = GraphFile::create("test.vite".into());
    gf.write(&g).unwrap();

    let mut new_gf = GraphFile::open("test.vite".into());
    let _graph_bytes = new_gf.read().unwrap();
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
