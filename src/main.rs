
mod graph;
mod image_graph;
mod math;

use {
    crate::{
        math::*,
        image_graph::extract_graph,
        graph::Graph,
    },
    image::{self as im, ConvertBuffer, Pixel},
    //rand::{prelude::*},
    //pcg_rand,
};


fn fill_rect<P: Pixel> (image: &mut impl im::GenericImage<Pixel = P>, rect: Rect, with: P) {
    let bounds = Rect::new_unchecked(
        V2::new(0, 0),
        V2::new(image.width() as i32, image.height() as i32)
    );
    let rect = rect.intersect(bounds);

    for y in rect.mins.y .. rect.maxs.y {
        for x in rect.mins.x .. rect.maxs.x {
            image.put_pixel(x as u32, y as u32, with);
        }
    }
}

fn main() {
    let in_image = im::open("maze2.png")
        .expect("loading image")
        .to_luma();

    let width  = in_image.width() as usize;
    let height = in_image.height() as usize;

    // TODO: compute
    let start_pos = V2::new(2, 0);
    let goal_pos  = V2::new((width - 1) as i32, (height - 3) as i32);

    eprintln!("Building graph...");
    let mut graph = extract_graph(&in_image, start_pos, goal_pos).unwrap();

    eprintln!("Pruning graph...");
    graph = graph.prune();

    eprintln!("Finding path...");
    let graph = graph.to_adjacency_graph();
    let graph = graph.into_dijkstra();

    let solution_length = graph.goal_distance();
    eprintln!("Solution length: {}", solution_length);

    // render
    eprintln!("Rendering...");
    let mut image: im::RgbImage = in_image.convert();

    //let mut rand = rand::distributions::Uniform::new_inclusive(0x80, 0xff)
    //    .sample_iter(pcg_rand::Pcg32Basic::seed_from_u64(12345));
    //let components: Vec<u8> = rand.take(3).collect();
    //fill_rect(debug_image, rect, *im::Rgb::from_slice(&components));

    for rect in graph.nodes().values() {
        const GREEN: im::Rgb<u8> = im::Rgb([0x00, 0xff, 0x00]);
        fill_rect(&mut image, *rect, GREEN);
    }

    {   let mut next_id = Some(graph.goal());
        while let Some(id) = next_id {
            let rect = *graph.get_node(id);
            const RED: im::Rgb<u8> = im::Rgb([0xff, 0x00, 0x00]);
            fill_rect(&mut image, rect, RED);
            next_id = graph.predecessor(id);
        }
    }

    image.save("solved.png").expect("saving maze");
}

