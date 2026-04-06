//! Graph export to various formats

use crate::NetworkGraph;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    GraphMl,
}

pub struct GraphExporter;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportedGraph {
    pub nodes: usize,
    pub edges: usize,
    pub data: String,
}

impl GraphExporter {
    pub fn export(graph: &NetworkGraph, format: ExportFormat) -> anyhow::Result<ExportedGraph> {
        match format {
            ExportFormat::Json => Self::export_json(graph),
            ExportFormat::GraphMl => Self::export_graphml(graph),
        }
    }

    fn export_json(graph: &NetworkGraph) -> anyhow::Result<ExportedGraph> {
        let json = serde_json::json!({
            "nodes": graph.node_count(),
            "edges": graph.edge_count(),
        });

        Ok(ExportedGraph {
            nodes: graph.node_count(),
            edges: graph.edge_count(),
            data: json.to_string(),
        })
    }

    fn export_graphml(graph: &NetworkGraph) -> anyhow::Result<ExportedGraph> {
        // TODO: Implement GraphML export
        Ok(ExportedGraph {
            nodes: graph.node_count(),
            edges: graph.edge_count(),
            data: "<?xml version=\"1.0\"?><graphml></graphml>".to_string(),
        })
    }
}
