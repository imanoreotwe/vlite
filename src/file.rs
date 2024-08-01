use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::rc::Rc;

use crate::hnsw::{Graph, Node};

/*
offset  size(b) description
----------------------------------------------
0       14      vite format 0\000
--------- GRAPH DATA - 64b --------------------
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
        8       length of layer
        8*n     index of friend
*/

pub struct GraphFile {
    file: File,
    filename: String,
}

impl GraphFile {
    pub fn create(path: String) -> Self {
        GraphFile {
            file: File::create(path.clone()).unwrap(),
            filename: path,
        }
    }

    pub fn open(path: String) -> Self {
        GraphFile {
            file: File::open(path.clone()).unwrap(),
            filename: path,
        }
    }

    pub fn write(&mut self, g: &Graph) -> std::io::Result<()> {
        self.file.write(b"vite format 0\0")?;
        self.file.write(&*g.serialize())?;
        g.nodes.iter().for_each(|n| {
            self.file.write(&*n.borrow().serialize());
        });
        self.file.write(b"\0\0\0\0")?;
        Ok(())
    }

    pub fn read(&mut self) -> Result<Box<Graph>, io::Error> {
        let mut header_buffer = [0; 78];
        self.file.read(&mut header_buffer)?;
        let graph_bytes: [u8; 64] = header_buffer[14..78].try_into().unwrap();
        let mut g = Graph::deserialize(&graph_bytes);

        let mut size_buff = [0; 4];
        let mut handle = self.file.try_clone().unwrap().take(4);
        handle.read(&mut size_buff)?;

        let mut node_size = u32::from_be_bytes(size_buff);
        let mut buff: Vec<[u8; 1024]> = vec![];
        let mut i = 0;

        println!("node size {}", node_size);
        while node_size > 0 {
            buff.push([0; 1024]);
            handle = self
                .file
                .try_clone()
                .unwrap()
                .take(node_size.try_into().unwrap());
            let read = handle.read(&mut buff[i])?;

            node_size -= read as u32;
            if node_size == 0 {
                g.nodes.push(Node::deserialize(&buff.concat(), g.dimension));

                handle = self.file.try_clone().unwrap().take(4);
                handle.read(&mut size_buff)?;
                node_size = u32::from_be_bytes(size_buff);
                println!("node size {}", node_size);
                i = 0;
            } else {
                println!("reloading!");
                i += 1;
            }
        }
        println!("done!");
        Ok(g)
    }
}
