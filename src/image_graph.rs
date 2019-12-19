
use {
    crate::{
        math::*,
        graph::{Edge, EdgeSetGraph, NodeID},
    },
    std::{
        collections::{HashMap, HashSet, VecDeque,},
    },
    image as im,
};

const WHITE: im::Luma<u8> = im::Luma([255; 1]);

fn grow_rect(grid: &Grid, seed: V2) -> Rect {
    let mut mins = V2::new(seed.x, seed.y);
    let mut maxs = V2::new(seed.x + 1, seed.y + 1);

    loop {
        let pos = V2::new(mins.x - 1, seed.y);
        if !grid.in_bounds(pos) || grid.get(pos) != GridSquare::Clear {
            break;
        }
        mins.x -= 1;
    }

    loop {
        let pos = V2::new(maxs.x, seed.y);
        if !grid.in_bounds(pos) || grid.get(pos) != GridSquare::Clear {
            break
        }
        maxs.x += 1;
    }

    'ymin_loop: loop {
        let y = mins.y - 1;
        if !grid.in_bounds(V2::new(mins.x, y)) {
            break;
        }

        for x in mins.x .. maxs.x {
            if grid.get(V2::new(x, y)) != GridSquare::Clear {
                break 'ymin_loop;
            }
        }
        mins.y -= 1;
    }

    'ymax_loop: loop {
        if !grid.in_bounds(V2::new(mins.x, maxs.y)) {
            break;
        }

        for x in mins.x .. maxs.x {
            if grid.get(V2::new(x, maxs.y)) != GridSquare::Clear {
                break 'ymax_loop;
            }
        }
        maxs.y += 1;
    }

    Rect::new(mins, maxs)
}


type SeedQueue = VecDeque<V2>;

fn scan_edge(
    grid:  &Grid,
    queue: &mut SeedQueue,
    edges: &mut HashSet<Edge>,
    id:    NodeID,
    start: V2,
    step:  V2,
    count: i32)
{
    let mut pos = start;
    let mut prev_square = GridSquare::Wall;

    for _ in 0 .. count {
        if !grid.in_bounds(pos) { break; }

        let square = grid.get(pos);
        if square != prev_square {
            match prev_square {
                GridSquare::Covered(prev_id) => {
                    edges.insert(Edge::new(id, prev_id));
                }
                GridSquare::Clear => {
                    queue.push_back(pos - step);
                }
                GridSquare::Wall => { }
            }

            prev_square = square;
        }

        pos = pos + step;
    }

    match prev_square {
        GridSquare::Covered(prev_id) => {
            edges.insert(Edge::new(id, prev_id));
        }
        GridSquare::Clear => {
            queue.push_back(pos - step);
        }
        GridSquare::Wall => { }
    }
}

fn scan_rect_boundary(
    grid:  &Grid,
    queue: &mut SeedQueue,
    edges: &mut HashSet<Edge>,
    id:    NodeID,
    rect:  Rect)
{
    scan_edge(grid, queue, edges, id,
        V2::new(rect.mins.x, rect.mins.y-1), V2::new(1, 0), rect.width());
    scan_edge(grid, queue, edges, id,
        V2::new(rect.mins.x, rect.maxs.y), V2::new(1, 0), rect.width());
    scan_edge(grid, queue, edges, id,
        V2::new(rect.mins.x-1, rect.mins.y), V2::new(0, 1), rect.height());
    scan_edge(grid, queue, edges, id,
        V2::new(rect.maxs.x, rect.mins.y), V2::new(0, 1), rect.height());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GridSquare {
    Clear,
    Wall,
    Covered(NodeID)
}

struct Grid {
    squares: Vec<GridSquare>,
    width:   usize,
    height:  usize,
}

impl Grid {
    fn new_from_image(image: &im::GrayImage) -> Grid {
        let width = image.width() as usize;
        let height = image.height() as usize;
        let squares: Vec<GridSquare> = image.pixels()
            .map(|pixel|
                if *pixel == WHITE { GridSquare::Clear }
                else               { GridSquare::Wall }
            )
            .collect();

        Grid { squares, width, height }
    }

    fn get_mut(&mut self, pos: V2) -> &mut GridSquare {
        let index = pos.y as usize * self.width + pos.x as usize;
        &mut self.squares[index]
    }

    fn get(&self, pos: V2) -> GridSquare {
        let index = pos.y as usize * self.width + pos.x as usize;
        self.squares[index]
    }

    fn in_bounds(&self, pos: V2) -> bool {
        Rect::new_unchecked(V2::new(0, 0), V2::new(self.width as i32, self.height as i32))
            .contains(pos)
    }
}

pub fn extract_graph(image: &im::GrayImage, start: V2, goal: V2) -> Option<EdgeSetGraph<Rect>> {
    let mut grid = Grid::new_from_image(image);

    let mut nodes: HashMap<NodeID, Rect> = HashMap::new();
    let mut edges: HashSet<Edge> = HashSet::new();

    let mut queue: SeedQueue = SeedQueue::new();
    queue.push_back(start);

    let mut id = NodeID::new(1).unwrap();

    let mut start_id = None;
    let mut goal_id = None;

    while let Some(seed) = queue.pop_front() {
        if grid.get(seed) != GridSquare::Clear {
            continue;
        }

        // grow rect into the space around the seed
        let rect = grow_rect(&grid, seed);

        // claim the covered squares
        for y in rect.mins.y .. rect.maxs.y {
            for x in rect.mins.x .. rect.maxs.x {
                *grid.get_mut(V2::new(x, y)) = GridSquare::Covered(id);
            }
        }

        // scan the edge of the rect for adjacent spaces
        scan_rect_boundary(&grid, &mut queue, &mut edges, id, rect);

        nodes.insert(id, rect);

        if rect.contains(start) {
            debug_assert!(start_id.is_none());
            start_id = Some(id);
        }

        if rect.contains(goal) {
            debug_assert!(goal_id.is_none());
            goal_id = Some(id);
        }

        id = NodeID::new(id.get() + 1).unwrap();
    }

    let start = start_id.unwrap();
    let goal  = goal_id.unwrap();

    Some(EdgeSetGraph::new(nodes, start, goal, edges))
}

//fn traverse(image: &im::GrayImage, start: V2) -> HashMap<V2, HashSet<V2>> {
//    let mut adjs: HashMap<V2, HashSet<V2>> = HashMap::new();
//
//    let mut queue = VecDeque::new();
//    queue.push_back(start);
//
//    while let Some(position) = queue.pop_front() {
//        const DIRS: [V2; 4] = [V2::new(1, 0), V2::new(0, 1), V2::new(-1, 0), V2::new(0, -1)];
//        for dir in DIRS.iter().copied() {
//            let new_position = position + dir;
//
//            use hash_map::Entry::*;
//            match adjs.entry(new_position) {
//                Occupied(mut entry) => {
//                    entry.get_mut().insert(position);
//                    adjs.entry(position).or_insert(HashSet::new()).insert(new_position);
//                }
//                Vacant(entry) => {
//                    if !image.in_bounds(new_position.x as u32, new_position.y as u32) {
//                        continue;
//                    }
//
//                    let pixel = *image.get_pixel(new_position.x as u32, new_position.y as u32);
//                    let clear = pixel == im::Luma([255]);
//
//                    // Move and recurse
//                    if clear {
//                        let mut set = HashSet::new();
//                        set.insert(position);
//                        entry.insert(set);
//                        adjs.entry(position).or_insert(HashSet::new()).insert(new_position);
//                        queue.push_back(new_position);
//                    }
//                }
//            }
//        }
//    }
//
//    adjs
//}

