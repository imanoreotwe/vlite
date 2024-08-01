use core::cmp::{Ordering, Reverse};
use rand::distributions::Uniform;
use rand::Rng;
use std::{
    cell::RefCell,
    cmp::min,
    collections::BinaryHeap,
    io::Write,
    rc::{Rc, Weak},
};

pub enum EntrencePoint {
    Weak(NodeWeak),
    Index(usize),
}

impl EntrencePoint {
    fn weak(&self) -> Option<&NodeWeak> {
        if let EntrencePoint::Weak(weak) = self {
            Some(weak)
        } else {
            None
        }
    }

    fn index(&self) -> Option<&usize> {
        if let EntrencePoint::Index(index) = self {
            Some(index)
        } else {
            None
        }
    }
}

pub struct Graph {
    // the starting vector will be at vectors[0]
    pub nodes: Vec<NodeRef>,
    pub entrence_point: EntrencePoint,
    pub layer_count: usize,
    pub m_l: f64,
    pub m_max: usize,
    pub m_max0: usize,
    pub canidate_list_size: usize,
    pub dimension: usize,
}

impl Graph {
    pub fn new(q: &[f64], m: f64, m_max: usize, m_max0: usize, canidate_list_size: usize) -> Self {
        let node = Node::new(0, 0, q);
        Graph {
            nodes: vec![node.clone()],
            entrence_point: EntrencePoint::Weak(Rc::downgrade(&node)),
            layer_count: 1,
            m_l: 1.0 / m.ln(),
            m_max: m_max,
            m_max0: m_max0,
            canidate_list_size: canidate_list_size,
            dimension: q.len(),
        }
    }

    /*
    M: target number of total connections (fudgy)
    M_max: max number of connections per layer
    M_max0: max number of connections at layer 0
    M_l: normalization factor for level generation [1/ln(M) is a good choice]
    */
    pub fn insert(&mut self, q: &[f64]) {
        let new_level = min(calc_level(self.m_l), self.layer_count);

        self.nodes.push(Node::new(self.nodes.len(), new_level, q));

        let mut push_ep = false;
        let old_ep = self
            .entrence_point
            .weak()
            .unwrap()
            .clone()
            .upgrade()
            .unwrap();
        if new_level >= self.layer_count {
            push_ep = true;
            self.layer_count += 1;
            self.entrence_point =
                EntrencePoint::Weak(Rc::downgrade(&self.nodes.last().unwrap().clone()));
        }

        for _i in 0..new_level {
            let mut new_node = self.nodes.last().unwrap().borrow_mut();
            new_node.friend_layers.push(Vec::new());
        }

        let mut ep = self
            .entrence_point
            .weak()
            .unwrap()
            .clone()
            .upgrade()
            .unwrap();
        let ep_level = ep.borrow().max_level;
        for i in ep_level..=new_level {
            ep = if let Some(ep) = search_layer(&q, ep.clone(), 1, i).pop() {
                ep.ptr().unwrap().clone()
            } else {
                ep
            }
        }

        for i in (0..=min(self.layer_count - 1, new_level)).rev() {
            // for each layer we need to fill in the neighbors of new_node
            let nearest_nodes = search_layer(&q, ep.clone(), self.canidate_list_size, i); // hmmm
            let m = if i > 0 { self.m_max } else { self.m_max0 };

            let neighbors = select_neighbors_simple(&q, &nearest_nodes, m);

            // fill friends
            for v in &neighbors {
                let new_node = self.nodes.last().unwrap();
                if !v.ptr().unwrap().eq(new_node) {
                    push_friend(
                        new_node.clone(),
                        v.ptr().unwrap().clone(),
                        i,
                        self.m_max,
                        self.m_max0,
                        true,
                    );
                }
            }
            if push_ep {
                ep = old_ep.clone();
                push_ep = false;
            } else {
                ep = nearest_nodes[0].ptr().unwrap().clone();
            }
        }
    }

    // will always be 64 bytes according to file.rs
    pub fn serialize(&self) -> Box<[u8; 64]> {
        let mut collect: Vec<u8> = Vec::new();
        // vector dimension
        self.nodes[0]
            .borrow()
            .vector
            .len()
            .to_be_bytes()
            .iter()
            .for_each(|&x| collect.push(x));

        // node count
        self.nodes
            .len()
            .to_be_bytes()
            .iter()
            .for_each(|&x| collect.push(x));

        // entrence point index
        self.entrence_point
            .weak()
            .unwrap()
            .upgrade()
            .unwrap()
            .borrow()
            .index
            .to_be_bytes()
            .iter()
            .for_each(|&x| collect.push(x));

        // layers
        self.layer_count
            .to_be_bytes()
            .iter()
            .for_each(|&x| collect.push(x));

        // m_l
        self.m_l.to_be_bytes().iter().for_each(|&x| collect.push(x));

        // m_max
        self.m_max
            .to_be_bytes()
            .iter()
            .for_each(|&x| collect.push(x));

        // m_max0
        self.m_max0
            .to_be_bytes()
            .iter()
            .for_each(|&x| collect.push(x));

        // canidate list length
        self.canidate_list_size
            .to_be_bytes()
            .iter()
            .for_each(|&x| collect.push(x));

        Box::new(collect.try_into().unwrap())
    }

    pub fn deserialize(bytes: &[u8; 64]) -> Box<Graph> {
        let dimension = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let entrence_point_index = u64::from_be_bytes(bytes[16..24].try_into().unwrap());
        let layer_count = u64::from_be_bytes(bytes[24..32].try_into().unwrap());
        let m_l = f64::from_be_bytes(bytes[32..40].try_into().unwrap());
        let m_max = u64::from_be_bytes(bytes[40..48].try_into().unwrap());
        let m_max0 = u64::from_be_bytes(bytes[48..56].try_into().unwrap());
        let canidate = u64::from_be_bytes(bytes[56..64].try_into().unwrap());

        Box::new(Graph {
            entrence_point: EntrencePoint::Index(entrence_point_index as usize),
            layer_count: layer_count as usize,
            m_l: m_l,
            m_max: m_max as usize,
            m_max0: m_max0 as usize,
            canidate_list_size: canidate as usize,
            nodes: Vec::new(),
            dimension: dimension as usize,
        })
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        println!("layer count:\t{}", self.layer_count);
        println!("m_l:\t{}", self.m_l);
        println!("m_max:\t{}", self.m_max);
        println!("m_max0:\t{}", self.m_max0);
        println!("canidate list size:\t{}", self.canidate_list_size);
        println!("dimension:\t{}", self.dimension);

        let ep_index = match &self.entrence_point {
            EntrencePoint::Index(index) => index,
            EntrencePoint::Weak(weak) => &weak.upgrade().unwrap().borrow().index.clone(),
        };
        for node in self.nodes.iter() {
            if node.borrow().index == *ep_index {
                print!("*");
            }
            node.borrow().print();
        }
        print!("\n");
    }
}

pub struct Node {
    pub index: usize,
    pub friend_layers: Vec<Vec<NodePtr>>,
    pub max_level: usize,
    pub vector: Box<[f64]>,
}

type NodeRef = Rc<RefCell<Node>>;
type NodeWeak = Weak<RefCell<Node>>;

#[derive(Clone)]
pub enum NodePtr {
    Ptr(NodeRef),
    Index(usize),
}

impl NodePtr {
    fn ptr(&self) -> Option<&NodeRef> {
        if let NodePtr::Ptr(ptr) = self {
            Some(ptr)
        } else {
            None
        }
    }

    fn index(&self) -> Option<&usize> {
        if let NodePtr::Index(index) = self {
            Some(index)
        } else {
            None
        }
    }

    fn try_into_ptr(&mut self, g: &Graph) -> Option<NodePtr> {
        let index = self.index().unwrap();
        Some(NodePtr::Ptr(g.nodes[*index].clone()))
    }
}

impl Node {
    pub fn new(index: usize, max_level: usize, vector: &[f64]) -> NodeRef {
        Rc::new(RefCell::new(Node {
            index: index,
            max_level: max_level,
            vector: vector.into(),
            friend_layers: vec![Vec::new()],
        }))
    }

    pub fn serialize(&self) -> Box<[u8]> {
        let mut collect: Vec<u8> = vec![0, 0, 0, 0];

        self.index
            .to_be_bytes()
            .iter()
            .for_each(|&x| collect.push(x));
        self.max_level
            .to_be_bytes()
            .iter()
            .for_each(|&x| collect.push(x));
        self.vector
            .iter()
            .for_each(|x| x.to_be_bytes().iter().for_each(|&b| collect.push(b)));

        self.friend_layers.iter().for_each(|f| {
            f.len().to_be_bytes().iter().for_each(|&b| collect.push(b));
            f.iter().for_each(|x| {
                x.ptr()
                    .unwrap()
                    .borrow()
                    .index
                    .to_be_bytes()
                    .iter()
                    .for_each(|&b| collect.push(b))
            })
        });

        let len_bytes = (collect.len() - 4).to_be_bytes();
        collect[0] = len_bytes[4];
        collect[1] = len_bytes[5];
        collect[2] = len_bytes[6];
        collect[3] = len_bytes[7];

        collect.into_boxed_slice()
    }

    // assuming bytes[0] excludes the length bytes and starts at index
    pub fn deserialize(bytes: &[u8], dimension: usize) -> NodeRef {
        let index = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let max_level = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
        let mut vector: Vec<f64> = Vec::new();
        let mut k = 16 + dimension * 8;
        for x in bytes[16..k].chunks(8) {
            vector.push(f64::from_be_bytes(x.try_into().unwrap()));
        }

        let mut friends: Vec<Vec<NodePtr>> = vec![];

        for i in 0..=max_level as usize {
            let len = u64::from_be_bytes(bytes[k..k + 8].try_into().unwrap());
            k += 8;

            friends.push(Vec::new());
            for _j in 0..len {
                friends[i].push(NodePtr::Index(u64::from_be_bytes(
                    bytes[k..k + 8].try_into().unwrap(),
                ) as usize));
                k += 8;
            }
        }

        Rc::new(RefCell::new(Node {
            index: index as usize,
            friend_layers: friends,
            max_level: max_level as usize,
            vector: vector.into_boxed_slice(),
        }))
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        print!("Node {}, Layer {} ", self.index, self.max_level,);

        for (i, v) in self.friend_layers.iter().enumerate() {
            print!("Friends{}: ", i);
            for f in v {
                match f {
                    NodePtr::Index(index) => print!("{}, ", index),
                    NodePtr::Ptr(ptr) => print!("{}, ", ptr.borrow().index),
                }
            }
        }
        print!("\n");
        std::io::stdout().flush().expect("stinky");
    }
}

fn push_friend(
    node: NodeRef,
    friend: NodeRef,
    level: usize,
    m_max: usize,
    m_max0: usize,
    propagate: bool,
) {
    if propagate {
        push_friend(friend.clone(), node.clone(), level, m_max, m_max0, false);
    }
    let node_bind = node.clone();
    {
        let mut node_borrow = node_bind.borrow_mut();
        node_borrow.friend_layers[level].push(NodePtr::Ptr(friend.clone()));
    }

    let mut new_neighbors: Vec<NodePtr> = vec![];
    {
        let node_iborrow = node_bind.borrow();
        if let Some(m) = shrinkable(
            node_iborrow.friend_layers[level].len(),
            level,
            m_max,
            m_max0,
        ) {
            new_neighbors = select_neighbors_simple(
                &node_iborrow.vector.clone(),
                &node_iborrow.friend_layers[level],
                m,
            );
        }
    }
    if new_neighbors.len() > 0 {
        node_bind.borrow_mut().friend_layers[level] = new_neighbors.clone();
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Node {}

struct NodeHeapItem {
    distance: f64,
    node: NodeRef,
}

impl Ord for NodeHeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.partial_cmp(&other.distance).unwrap()
    }
}

impl PartialOrd for NodeHeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NodeHeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for NodeHeapItem {}

pub fn knn_search(g: &Graph, q: &[f64], k: usize, ef: usize) -> Vec<NodeRef> {
    let mut canidates = BinaryHeap::new();
    let mut entrence_point = g.entrence_point.weak().unwrap().upgrade().unwrap();
    let level = g.layer_count - 1;

    for l in (1..=level).rev() {
        let tmp = search_layer(q, entrence_point, 1, l)[0].clone();
        canidates.push(Reverse(NodeHeapItem {
            distance: cosine_distance(q, &tmp.clone().ptr().unwrap().borrow().vector),
            node: tmp.ptr().unwrap().clone(),
        }));

        entrence_point = canidates.pop().unwrap().0.node;
    }

    search_layer(q, entrence_point, ef, 0).iter().for_each(|x| {
        canidates.push(Reverse(NodeHeapItem {
            distance: cosine_distance(q, &x.ptr().unwrap().borrow().vector),
            node: x.ptr().unwrap().clone(),
        }))
    });

    return canidates
        .into_sorted_vec()
        .into_iter()
        .rev()
        .take(k)
        .map(|a| a.0.node)
        .collect::<Vec<NodeRef>>();
}

fn select_neighbors_simple(q: &[f64], c: &Vec<NodePtr>, m: usize) -> Vec<NodePtr> {
    let mut nearest_heap = BinaryHeap::new();
    for v in c {
        nearest_heap.push(Reverse(NodeHeapItem {
            distance: cosine_distance(q, &v.ptr().unwrap().borrow().vector),
            node: v.ptr().unwrap().clone(),
        }));
    }

    let mut res: Vec<NodePtr> = Vec::new();
    for _i in 0..min(m, nearest_heap.len()) {
        let Reverse(nearest) = nearest_heap.pop().unwrap();
        res.push(NodePtr::Ptr(nearest.node));
    }

    return res;
}

/*
`ep` must be on the same layer as `layer`
 */
fn search_layer(q: &[f64], ep: NodeRef, count: usize, layer: usize) -> Vec<NodePtr> {
    assert!(ep.borrow().max_level >= layer);
    let mut visited = Vec::new();
    let mut canidates = BinaryHeap::new();
    let mut found = BinaryHeap::new();

    let init_dist = cosine_distance(q, &ep.borrow().vector);

    visited.push(ep.clone());
    canidates.push(Reverse(NodeHeapItem {
        distance: init_dist,
        node: ep.clone(),
    })); // top of heap is nearest to q

    found.push(NodeHeapItem {
        distance: init_dist,
        node: ep.clone(),
    }); // top of heap is furthest from q

    assert!(found.len() == 1);
    while canidates.len() > 0 {
        let Reverse(canidate) = canidates.pop().unwrap();
        let mut furthest = found.peek().unwrap();

        if canidate.distance > furthest.distance {
            break;
        }

        if canidate.node.borrow().friend_layers.len() > layer {
            for e in &canidate.node.borrow().friend_layers[layer] {
                if !contains_rc(&visited, e.ptr().unwrap().clone()) {
                    visited.push(e.ptr().unwrap().clone());
                    furthest = found.peek().unwrap();
                    if cosine_distance(&e.ptr().unwrap().borrow().vector, q) < furthest.distance
                        || found.len() < count
                    {
                        canidates.push(Reverse(NodeHeapItem {
                            distance: cosine_distance(q, &e.ptr().unwrap().borrow().vector),
                            node: e.ptr().unwrap().clone(),
                        }));
                        found.push(NodeHeapItem {
                            distance: cosine_distance(q, &e.ptr().unwrap().borrow().vector),
                            node: e.ptr().unwrap().clone(),
                        });
                        if found.len() > count {
                            found.pop();
                        }
                    }
                }
            }
        }
    }
    return found
        .into_sorted_vec()
        .into_iter()
        .rev()
        .map(|a| NodePtr::Ptr(a.node))
        .collect::<Vec<NodePtr>>();
}

pub fn cosine_distance(a: &[f64], b: &[f64]) -> f64 {
    let mut num = 0.0;
    for i in 0..a.len() {
        num += a[i] * b[i];
    }

    let mut sum = 0.0;
    for f in a {
        sum += f.powi(2);
    }
    let mut dem = sum.sqrt();
    sum = 0.0;
    for f in b {
        sum += f.powi(2);
    }
    dem = dem * sum.sqrt();

    return 1.0 - (num / dem);
}

fn calc_level(m_l: f64) -> usize {
    let mut rng = rand::thread_rng();
    let side = Uniform::new(0_f64, 1_f64);
    let sample = rng.sample(side);

    return (-sample.ln() * m_l).floor() as usize; //potentially fuckywucky
}

fn contains_rc(v: &Vec<NodeRef>, n: NodeRef) -> bool {
    for e in v {
        if Rc::ptr_eq(&e, &n) {
            return true;
        }
    }
    return false;
}

fn shrinkable(friends_count: usize, layer: usize, m_max: usize, m_max0: usize) -> Option<usize> {
    if layer > 0 {
        if friends_count > m_max {
            return Some(m_max);
        }
    } else {
        if friends_count > m_max0 {
            return Some(m_max0);
        }
    }
    None
}
