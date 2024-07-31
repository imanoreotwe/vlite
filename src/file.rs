use std::io::Write;

mod hnsw;
use crate::hnsw::{Graph, Node};

/*
offset  size(b) description
----------------------------------------------
0       14      vite format 0\000
16      4       file change counter
20      4       size of db in pages
24      4       offset of first free page
--------- GRAPH DATA --------------------------
28      8       vector dimension
36      8       graph size
42      8       enternce point index
50      4       layers
54      8       m_l
62      8       m_max
70      8       m_max0
78      4       canidate list size


--------------- NODE FORMAT -------------------
offset  size(b) description
0       4       node size
4       8       index
12      4       layers
16      8*d     vector
        8*l     size of each layer 
        8*?*?   index of each node on each layer 
*/

pub fn write_graph(g: &Graph) {

}