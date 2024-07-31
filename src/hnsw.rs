use core::cmp::{Ordering, Reverse};

use rand::distributions::Uniform;
use rand::Rng;
use std::{
    borrow::Borrow, cell::RefCell, cmp::min, collections::BinaryHeap, io::Write, rc::{Rc, Weak}
};

pub struct Graph {
    // the starting vector will be at vectors[0]
    pub nodes: Vec<NodeRef>,
    pub entrence_point: NodeWeak,
    pub layer_count: usize,
    pub m_l: f64,
    pub m_max: usize,
    pub m_max0: usize,
    pub canidate_list_size: usize
}

impl Graph {
    pub fn new(q: &[f64], m: f64, m_max: usize, m_max0: usize, canidate_list_size: usize) -> Self {
        let node = Node::new(0, 0, q);
        Graph {
            nodes: vec![node.clone()],
            entrence_point: Rc::downgrade(&node),
            layer_count: 1,
            m_l: 1.0 / m.ln(),
            m_max: m_max,
            m_max0: m_max0,
            canidate_list_size: canidate_list_size
        }
    }

    /*
    M: target number of total connections (fudgy)
    M_max: max number of connections per layer
    M_max0: max number of connections at layer 0
    M_l: normalization factor for level generation [1/ln(M) is a good choice]
    */
    pub fn insert(
        &mut self,
        q: &[f64],
    ) {
        let new_level = min(calc_level(self.m_l), self.layer_count);

        self.nodes.push(Node::new(self.nodes.len(), new_level, q));

        let mut push_ep = false;
        let old_ep = self.entrence_point.clone().upgrade().unwrap();
        if new_level >= self.layer_count {
            push_ep = true;
            self.layer_count += 1;
            self.entrence_point = Rc::downgrade(&self.nodes.last().unwrap().clone());
        }

        for _i in 0..new_level {
            let mut new_node = self.nodes.last().unwrap().borrow_mut();
            new_node.friend_layers.push(Vec::new());
        }

        let mut ep = self.entrence_point.clone().upgrade().unwrap();
        let ep_level = ep.borrow().max_level;
        for i in ep_level..=new_level {
            ep = if let Some(ep) = search_layer(&q, ep.clone(), 1, i).pop() {
                ep.clone()
            } else {
                ep
            }
        }

        for i in (0..=min(self.layer_count - 1, new_level)).rev() {
            // for each layer we need to fill in the neighbors of new_node
            let nearest_nodes = search_layer(&q, ep.clone(), self.canidate_list_size, i); // hmmm
            let m = if i > 0 {
                self.m_max
            } else {
                self.m_max0
            };

            let neighbors = select_neighbors_simple(&q, &nearest_nodes, m);

            // fill friends
            for v in &neighbors {
                let new_node = self.nodes.last().unwrap();
                if !v.eq(new_node) {
                    push_friend(new_node.clone(), v.clone(), i, self.m_max, self.m_max0, true);
                } 
            }
            if push_ep {
                ep = old_ep.clone();
                push_ep = false;
            } else {
                ep = nearest_nodes[0].clone();
            }
        }
    }

    pub fn serialize(&self) -> String {
        format!("{}{}{}{}{}{}{}{}", 
            self.nodes[0].borrow().vector.len(),
            self.nodes.len(),
            self.entrence_point.upgrade().unwrap().borrow().index,
            self.layer_count,
            self.m_l,
            self.m_max,
            self.m_max0,
            self.canidate_list_size
        )
    }


    #[allow(dead_code)]
    pub fn print(&self) {
        for node in self.nodes.iter() {
            if node.eq(&Weak::upgrade(&self.entrence_point).unwrap()) {
                print!("*");
            }
            node.borrow().print();
        }
        print!("\n");
    }
}

pub struct Node {
    pub index: usize,
    pub friend_layers: Vec<Vec<NodeRef>>,
    pub max_level: usize,
    pub vector: Box<[f64]>,
}

type NodeRef = Rc<RefCell<Node>>;
type NodeWeak = Weak<RefCell<Node>>;

impl Node {
    fn new(index: usize, max_level: usize, vector: &[f64]) -> NodeRef {
        Rc::new(RefCell::new(Node {
            index: index,
            max_level: max_level,
            vector: vector.into(),
            friend_layers: vec![Vec::new()],
        }))
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        print!("Node {}, Layer {} ", self.index, self.max_level,);

        for (i, v) in self.friend_layers.iter().enumerate() {
            print!("Friends{}: ", i);
            for f in v {
                match f.try_borrow() {
                    Ok(friend) => {
                        print!("{}, ", friend.index);
                    }
                    Err(_e) => {
                        print!("*, ");
                    }
                };
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
        node_borrow.friend_layers[level].push(friend.clone());
    }

    let mut new_neighbors: Vec<NodeRef> = vec![];
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
    let mut entrence_point = g.entrence_point.upgrade().unwrap();
    let level = g.layer_count-1;

    for l in (1..=level).rev() {
        let tmp = search_layer(q, entrence_point, 1, l)[0].clone();
        canidates.push(Reverse(NodeHeapItem {
            distance: cosine_distance(q, &tmp.clone().borrow().vector),
            node: tmp
        }));

        entrence_point = canidates.pop().unwrap().0.node;
    }

    search_layer(q, entrence_point, ef, 0).iter().for_each(|x| canidates.push(Reverse(NodeHeapItem {
        distance: cosine_distance(q, &x.borrow().vector),
        node: x.clone()
    })));

    return canidates
        .into_sorted_vec()
        .into_iter()
        .rev()
        .take(k)
        .map(|a| a.0.node)
        .collect::<Vec<NodeRef>>();
}

fn select_neighbors_simple(q: &[f64], c: &Vec<NodeRef>, m: usize) -> Vec<NodeRef> {
    let mut nearest_heap = BinaryHeap::new();
    for v in c {
        nearest_heap.push(Reverse(NodeHeapItem {
            distance: cosine_distance(q, &v.borrow().vector),
            node: v.clone(),
        }));
    }

    let mut res = Vec::new();
    for _i in 0..min(m, nearest_heap.len()) {
        let Reverse(nearest) = nearest_heap.pop().unwrap();
        res.push(nearest.node);
    }

    return res;
}

/*
`ep` must be on the same layer as `layer`
 */
fn search_layer(q: &[f64], ep: NodeRef, count: usize, layer: usize) -> Vec<NodeRef> {
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

                if !contains_rc(&visited, e.clone()) {
                    visited.push(e.clone());
                    furthest = found.peek().unwrap();
                    if cosine_distance(&e.borrow().vector, q) < furthest.distance
                        || found.len() < count
                    {
                        canidates.push(Reverse(NodeHeapItem {
                            distance: cosine_distance(q, &e.borrow().vector),
                            node: e.clone(),
                        }));
                        found.push(NodeHeapItem {
                            distance: cosine_distance(q, &e.borrow().vector),
                            node: e.clone(),
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
        .map(|a| a.node)
        .collect::<Vec<NodeRef>>();
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