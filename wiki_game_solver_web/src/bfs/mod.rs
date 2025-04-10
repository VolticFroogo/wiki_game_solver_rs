mod link_data;
mod file_watcher;

use std::collections::HashMap;
pub(crate) use crate::bfs::link_data::get_link_data;
pub(crate) use crate::bfs::link_data::LinkData;
pub(crate) use crate::bfs::file_watcher::watch_file;

pub(crate) fn bfs_bidirectional(link_data: &LinkData, start: u32, target: u32) -> Option<Vec<u32>> {
    let mut forward_link_queue: Vec<u32> = Vec::new();
    let mut reverse_link_queue: Vec<u32> = Vec::new();
    let mut forward_seen_by: HashMap<u32, u32> = HashMap::new();
    let mut reverse_seen_by: HashMap<u32, u32> = HashMap::new();

    forward_link_queue.push(start);
    reverse_link_queue.push(target);
    forward_seen_by.insert(start, start);
    reverse_seen_by.insert(target, target);

    while !forward_link_queue.is_empty() && !reverse_link_queue.is_empty() {
        let mut new_forward_link_queue: Vec<u32> = Vec::new();

        if let Some(path) = bfs_directional_search_step(
            &link_data.links_forward,
            &mut forward_link_queue,
            &mut forward_seen_by,
            &mut reverse_seen_by,
            &mut new_forward_link_queue) {
            return Some(path);
        }

        forward_link_queue = new_forward_link_queue;
        let mut new_reverse_link_queue: Vec<u32> = Vec::new();

        if let Some(path) = bfs_directional_search_step(
            &link_data.links_reverse,
            &mut reverse_link_queue,
            &mut reverse_seen_by,
            &mut forward_seen_by,
            &mut new_reverse_link_queue) {
            return Some(path.iter().map(|&x| x).rev().collect());
        }

        reverse_link_queue = new_reverse_link_queue;
    }

    None
}

fn bfs_directional_search_step(
    links: &HashMap<u32, Vec<u32>>,
    link_queue: &mut Vec<u32>,
    seen_by: &mut HashMap<u32, u32>,
    reverse_seen_by: &mut HashMap<u32, u32>,
    new_link_queue: &mut Vec<u32>)
    -> Option<Vec<u32>> {
    for &current_page_id in link_queue.iter() {
        if let Some(next_page_links) = links.get(&current_page_id) {
            for &next_page_id in next_page_links {
                if seen_by.contains_key(&next_page_id) {
                    continue;
                }

                seen_by.insert(next_page_id, current_page_id);

                if reverse_seen_by.contains_key(&next_page_id) {
                    return Some(bfs_generate_path(seen_by, reverse_seen_by, next_page_id));
                }

                new_link_queue.push(next_page_id);
            }
        }
    }

    None
}

fn bfs_generate_path(seen_by: &mut HashMap<u32, u32>, reverse_seen_by: &mut HashMap<u32, u32>, next_page_id: u32) -> Vec<u32> {
    let mut path = Vec::new();

    populate_path_with_seen_by_until_circular(seen_by, &mut path, next_page_id);

    path.reverse();
    path.push(next_page_id);

    populate_path_with_seen_by_until_circular(reverse_seen_by, &mut path, next_page_id);

    path
}

fn populate_path_with_seen_by_until_circular(seen_by: &mut HashMap<u32, u32>, path: &mut Vec<u32>, current_page_id: u32) {
    let mut current_page_id = current_page_id;

    while let Some(&next_page_id) = seen_by.get(&current_page_id) {
        if current_page_id == next_page_id {
            break;
        }

        path.push(next_page_id);
        current_page_id = next_page_id;
    }
}
