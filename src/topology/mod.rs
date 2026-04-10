//! Isoptropic Vector Matrix structre.

use std::collections::{HashMap, HashSet};

/// IVM position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IvmCoordinate {
    /// X
    pub x: i32,
    /// Y
    pub y: i32,
    /// Z
    pub z: i32,
}

/// IVM nodes and edges
#[derive(Clone, Debug)]
pub struct IvmTopology {
    /// 64 Nodes
    pub nodes: HashMap<u64, IvmCoordinate>,
    /// Node edges
    pub edges: HashMap<IvmCoordinate, HashSet<IvmCoordinate>>,
}

impl Default for IvmTopology {
    fn default() -> Self {
        Self::new()
    }
}

impl IvmTopology {
    /// Generates the 64 nodes of the Isotropic Vector Matrix and wires their topological edges.
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let mut qdu_counter = 0;

        // 1. Generate the 64 discrete coordinates
        // We use a scaled grid (-3, -1, 1, 3) to represent the structural
        // centers of the 64 tetrahedrons without using floating point math.
        let grid_points = [-3, -1, 1, 3];

        for &x in &grid_points {
            for &y in &grid_points {
                for &z in &grid_points {
                    let coord = IvmCoordinate { x, y, z };
                    nodes.insert(qdu_counter, coord);
                    qdu_counter += 1;
                }
            }
        }

        // 2. Define Adjacency (The Edges)
        // In this discrete matrix, two tetrahedrons are structurally adjacent
        // if they are exactly 2 units apart on a single axis (sharing a localized boundary).
        for (&_id_a, &coord_a) in &nodes {
            let mut neighbors = HashSet::new();

            for (&_id_b, &coord_b) in &nodes {
                // Skip self
                if coord_a == coord_b {
                    continue;
                }

                // Calculate the squared Euclidean distance
                let dx = coord_a.x - coord_b.x;
                let dy = coord_a.y - coord_b.y;
                let dz = coord_a.z - coord_b.z;
                let dist_sq = dx * dx + dy * dy + dz * dz;

                // A distance squared of exactly 4 means they are direct, immediate neighbors
                // in the fractal hierarchy (e.g., differing by exactly 2 on one axis).
                if dist_sq == 4 {
                    neighbors.insert(coord_b);
                }
            }
            edges.insert(coord_a, neighbors);
        }

        IvmTopology { nodes, edges }
    }

    /// The Locality Rule: Checks if two QDUs are physically adjacent
    pub fn are_adjacent(&self, qdu_a: u64, qdu_b: u64) -> bool {
        if let (Some(coord_a), Some(coord_b)) = (self.nodes.get(&qdu_a), self.nodes.get(&qdu_b))
            && let Some(neighbors) = self.edges.get(coord_a)
        {
            return neighbors.contains(coord_b);
        }
        false
    }
}
