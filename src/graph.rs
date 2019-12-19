
use {
    std::{
        cmp::Reverse,
        collections::{HashMap, HashSet},
        mem::swap,
    },
    priority_queue::PriorityQueue,
};

pub trait Graph<Data> {
    fn get_node(&self, id: NodeID) -> &Data;
    fn start(&self) -> NodeID;
    fn goal(&self) -> NodeID;
    fn nodes(&self) -> &HashMap<NodeID, Data>;
}

pub type NodeID = std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Edge {
    pub min: NodeID,
    pub max: NodeID,
}

impl Edge {
    pub fn new(a: NodeID, b: NodeID) -> Edge {
        Edge { min: std::cmp::min(a, b), max: std::cmp::max(a, b) }
    }
}

struct GraphCommon<Data> {
    pub nodes: HashMap<NodeID, Data>,
    pub start: NodeID,
    pub goal:  NodeID
}

pub struct EdgeSetGraph<Data> {
    pub com:   GraphCommon<Data>,
    pub edges: HashSet<Edge>,
}

pub struct AdjacencyGraph<Data> {
    pub com:  GraphCommon<Data>,
    pub adjs: HashMap<NodeID, HashSet<NodeID>>,
}

pub struct DijkstraGraph<Data> {
    pub inner: AdjacencyGraph<(Data, i32)>,
    pub paths: HashMap<NodeID, NodeID>,
}

impl<Data> Graph<Data> for EdgeSetGraph<Data> {
    fn get_node(&self, id: NodeID) -> &Data {
        &self.com.nodes[&id]
    }
    fn start(&self) -> NodeID { self.com.start }
    fn goal(&self) -> NodeID { self.com.goal }
    fn nodes(&self) -> &HashMap<NodeID, Data> {
        &self.com.nodes
    }
}

impl<Data> Graph<Data> for AdjacencyGraph<Data> {
    fn get_node(&self, id: NodeID) -> &Data {
        &self.com.nodes[&id]
    }
    fn start(&self) -> NodeID { self.com.start }
    fn goal(&self) -> NodeID { self.com.goal }
    fn nodes(&self) -> &HashMap<NodeID, Data> {
        &self.com.nodes
    }
}

impl<Data> Graph<Data> for DijkstraGraph<Data> {
    fn get_node(&self, id: NodeID) -> &Data {
        &self.inner.get_node(id).0
    }
    fn start(&self) -> NodeID { self.inner.start() }
    fn goal(&self) -> NodeID { self.inner.goal() }
    fn nodes(&self) -> &HashMap<NodeID, Data> {
        self.inner.nodes()
    }
}

impl<Data> EdgeSetGraph<Data> {
    pub fn new(nodes: HashMap<NodeID, Data>, start: NodeID, goal: NodeID, edges: HashSet<Edge>)
        -> EdgeSetGraph<Data>
    {
        let com = GraphCommon { nodes, start, goal };
        EdgeSetGraph { com, edges }
    }

    pub fn to_adjacency_graph(self) -> AdjacencyGraph<Data> {
        let mut adjs = HashMap::new();

        for Edge { min, max } in self.edges {
            adjs.entry(min).or_insert(HashSet::new()).insert(max);
            adjs.entry(max).or_insert(HashSet::new()).insert(min);
        }

        AdjacencyGraph { com: self.com, adjs }
    }

    pub fn prune(self) -> EdgeSetGraph<Data> {
        let mut edges = self.edges;
        let mut edges_temp: HashSet<Edge> = HashSet::with_capacity(edges.len());

        let mut nodes = self.com.nodes;
        let mut nodes_temp: HashMap<NodeID, Data> = HashMap::with_capacity(nodes.len());

        let mut degrees: HashMap<NodeID, usize> = HashMap::with_capacity(nodes.len());
        let mut dead_ends: HashSet<NodeID> = HashSet::with_capacity(nodes.len());

        loop {
            degrees.clear();
            degrees.extend(nodes
                .keys()
                .map(|id| (*id, 0))
            );

            for edge in edges.iter() {
                *degrees.get_mut(&edge.min).unwrap() += 1;
                *degrees.get_mut(&edge.max).unwrap() += 1;
            }

            dead_ends.clear();
            dead_ends.extend(degrees
                .iter()
                .filter(|(id, degree)| **degree < 2 && **id != self.start() && **id != self.goal())
                .map(|(id, _)| *id)
            );

            if dead_ends.is_empty() {
                break;
            }

            edges_temp.clear();
            edges_temp.extend(edges.iter()
                .copied()
                .filter(|edge| !dead_ends.contains(&edge.min) && !dead_ends.contains(&edge.max))
            );
            swap(&mut edges, &mut edges_temp);

            nodes_temp.clear();
            nodes_temp.extend(nodes.iter()
                .filter(|(id, _)| !dead_ends.contains(&id))
                .map(|(id, data)| (*id, *data))
            );
            swap(&mut nodes, &mut nodes_temp);
        }

        let com = GraphCommon { nodes, ..self.com };
        EdgeSetGraph { com, edges }
    }
}

impl<Data> AdjacencyGraph<Data> {
    pub fn new(
        nodes: HashMap<NodeID, Data>, start: NodeID, goal: NodeID,
        adjs: HashMap<NodeID, HashSet<NodeID>>)
        -> AdjacencyGraph<Data>
    {
        let com = GraphCommon { nodes, start, goal };
        AdjacencyGraph { com, adjs }
    }

    pub fn neighbors(&self, id: NodeID) -> &HashSet<NodeID> {
        self.adjs.get(&id).unwrap()
    }

    pub fn into_dijkstra(self) -> DijkstraGraph<Data> {
        let mut dists: HashMap<NodeID, Option<i32>> = self.com.nodes.keys()
            .map(|id| {
                let dist = if *id == self.start() { Some(0) } else { None };
                (*id, dist)
            })
            .collect();

        let mut paths: HashMap<NodeID, NodeID> = HashMap::new();

        let mut queue: PriorityQueue<NodeID, Reverse<i32>> = dists.iter()
            .map(|(id, dist)| (*id, Reverse(dist.unwrap_or(std::i32::MAX))))
            .collect();

        while let Some((u, _)) = queue.pop() {
            let neighbors = self.neighbors(u);

            for v in neighbors.iter() {
                let new_dist = dists[&u].unwrap_or(0) + 1;
                if new_dist < dists[&v].unwrap_or(std::i32::MAX) {
                    dists.insert(*v, Some(new_dist));
                    paths.insert(*v, u);
                    queue.change_priority(&v, Reverse(new_dist));
                }
            }
        }

        let nodes: HashMap<NodeID, (Data, i32)> = self.com.nodes.iter()
            .map(|(id, data)| (*id, (*data, dists[id].unwrap_or(std::i32::MAX))))
            .collect();

        let com = GraphCommon { nodes, start: self.com.start, goal: self.com.goal };
        let inner = AdjacencyGraph { com, adjs: self.adjs };
        DijkstraGraph { inner, paths }
    }
}

impl<Data> DijkstraGraph<Data> {
    pub fn distance(&self, id: NodeID) -> i32 {
        self.inner.get_node(id).1
    }

    pub fn goal_distance(&self) -> i32 {
        self.distance(self.goal())
    }

    pub fn predecessor(&self, id: NodeID) -> Option<NodeID> {
        self.paths.get(&id).map(|pred| *pred)
    }
}

// XXX uses _far_ too much memory to be practical on anything but the smallest mazes
// TODO check that i haven't just fucked up
//fn a_star(adjs: &HashMap<V2, Vec<V2>>, start: V2, goal: V2)
//    -> Option<Vec<V2>>
//{
//    let heur = |p: V2| -> i32 { let d = p - goal; d.x.abs() + d.y.abs() };
//
//    let mut queue: PriorityQueue<V2, Reverse<i32>> = PriorityQueue::new();
//    queue.push(start, Reverse(heur(start)));
//
//    let mut metrics: HashMap<V2, (V2, i32)> = HashMap::new();
//    metrics.insert(start, (start, 0));
//
//    while let Some((current, _)) = queue.pop() {
//        if current == goal {
//            let mut path = vec![current];
//            let mut prev = current;
//            while let Some((current, _)) = metrics.get(&prev) {
//                path.push(*current);
//                prev = *current;
//            }
//            return Some(path);
//        }
//
//        for neighbor in adjs[&current].iter() {
//            let new_score = metrics.get(&current) .map_or(std::i32::MAX, |(_, g)| *g) + 1;
//            let old_score = metrics.get(&neighbor).map_or(std::i32::MAX, |(_, g)| *g);
//            if new_score < old_score {
//                let new_metrics = (current, new_score);
//                metrics.insert(*neighbor, new_metrics);
//                queue.push(*neighbor, Reverse(heur(*neighbor)));
//            }
//        }
//    }
//
//    None
//}

