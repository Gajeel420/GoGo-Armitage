//! Network graph representation

use armitage_core::{Host, Service};
use petgraph::graph::{Graph, NodeIndex};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NetworkGraph {
    graph: Graph<GraphNode, GraphEdge>,
    node_map: HashMap<Uuid, NodeIndex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphNode {
    Host(Host),
    Service(Service),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub relationship: String,
}

impl NetworkGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_host(&mut self, host: Host) -> NodeIndex {
        let idx = self.graph.add_node(GraphNode::Host(host.clone()));
        self.node_map.insert(host.id, idx);
        idx
    }

    pub fn add_service(&mut self, service: Service) -> NodeIndex {
        let idx = self.graph.add_node(GraphNode::Service(service.clone()));
        self.node_map.insert(service.id, idx);
        idx
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, label: &str) {
        self.graph.add_edge(
            from,
            to,
            GraphEdge {
                relationship: label.to_string(),
            },
        );
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}

impl Default for NetworkGraph {
    fn default() -> Self {
        Self::new()
    }
}
