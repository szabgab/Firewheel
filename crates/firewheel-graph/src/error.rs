// Audio graph compilation algorithm adapted from:
// https://github.com/m-hilgendorf/audio-graph/tree/39c254073a73780335606f83e069afda230f0d3f

use std::error::Error;
use std::fmt;

use firewheel_core::{node::AudioNodeInfo, ChannelConfig, ChannelCount};

use crate::graph::{Edge, EdgeID, InPortIdx, NodeID, OutPortIdx};

/// An error occurred while attempting to add an edge to the graph.
#[derive(Debug, Clone)]
pub enum AddEdgeError {
    /// The given source node was not found in the graph.
    SrcNodeNotFound(NodeID),
    /// The given destination node was not found in the graph.
    DstNodeNotFound(NodeID),
    /// The given input port index is out of range.
    InPortOutOfRange {
        node: NodeID,
        port_idx: InPortIdx,
        num_in_ports: ChannelCount,
    },
    /// The given output port index is out of range.
    OutPortOutOfRange {
        node: NodeID,
        port_idx: OutPortIdx,
        num_out_ports: ChannelCount,
    },
    /// The edge already exists in the graph.
    EdgeAlreadyExists,
    /// The input port is already connected.
    InputPortAlreadyConnected(NodeID, InPortIdx),
    /// This edge would have created a cycle in the graph.
    CycleDetected,
}

impl Error for AddEdgeError {}

impl fmt::Display for AddEdgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SrcNodeNotFound(node_id) => {
                write!(
                    f,
                    "Could not add edge: could not find source node with ID {:?}",
                    node_id
                )
            }
            Self::DstNodeNotFound(node_id) => {
                write!(
                    f,
                    "Could not add edge: could not find destination node with ID {:?}",
                    node_id
                )
            }
            Self::InPortOutOfRange {
                node,
                port_idx,
                num_in_ports,
            } => {
                write!(
                    f,
                    "Input port idx {:?} is out of range on node {:?} with {:?} input ports",
                    port_idx, node, num_in_ports,
                )
            }
            Self::OutPortOutOfRange {
                node,
                port_idx,
                num_out_ports,
            } => {
                write!(
                    f,
                    "Output port idx {:?} is out of range on node {:?} with {:?} output ports",
                    port_idx, node, num_out_ports,
                )
            }
            Self::EdgeAlreadyExists => {
                write!(f, "Could not add edge: edge already exists in the graph",)
            }
            Self::InputPortAlreadyConnected(node_id, port_id) => {
                write!(
                    f,
                    "Could not add edge: input port with ID {:?} on node with ID {:?} is already connected",
                    port_id,
                    node_id,
                )
            }
            Self::CycleDetected => {
                write!(f, "Could not add edge: cycle was detected")
            }
        }
    }
}

/// An error occurred while attempting to compile the audio graph
/// into a schedule.
#[derive(Debug)]
pub enum CompileGraphError {
    /// A cycle was detected in the graph.
    CycleDetected,
    /// The input data contained an edge referring to a non-existing node.
    NodeOnEdgeNotFound(Edge, NodeID),
    /// The input data contained multiple nodes with the same ID.
    NodeIDNotUnique(NodeID),
    /// The input data contained multiple edges with the same ID.
    EdgeIDNotUnique(EdgeID),
    /// The input port has more than one connection.
    ManyToOneError(NodeID, InPortIdx),
    /// The message channel is full.
    MessageChannelFull,
}

impl Error for CompileGraphError {}

impl fmt::Display for CompileGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CycleDetected => {
                write!(f, "Failed to compile audio graph: a cycle was detected")
            }
            Self::NodeOnEdgeNotFound(edge, node_id) => {
                write!(f, "Failed to compile audio graph: input data contains an edge {:?} referring to a non-existing node {:?}", edge, node_id)
            }
            Self::NodeIDNotUnique(node_id) => {
                write!(f, "Failed to compile audio graph: input data contains multiple nodes with the same ID {:?}", node_id)
            }
            Self::EdgeIDNotUnique(edge_id) => {
                write!(f, "Failed to compile audio graph: input data contains multiple edges with the same ID {:?}", edge_id)
            }
            Self::ManyToOneError(node_id, port_id) => {
                write!(f, "Failed to compile audio graph: input data contains multiple edges that go to the same input port with ID {:?} on node with id {:?}", port_id, node_id)
            }
            Self::MessageChannelFull => {
                write!(f, "Failed to compile audio graph: Message channel is full")
            }
        }
    }
}

#[derive(Debug)]
pub enum NodeError {
    InvalidChannelConfig {
        channel_config: ChannelConfig,
        node_info: AudioNodeInfo,
        msg: Option<Box<dyn Error>>,
    },
    ActivationFailed {
        node_id: Option<NodeID>,
        error: Box<dyn Error>,
    },
}

impl Error for NodeError {}

impl std::fmt::Display for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeError::InvalidChannelConfig {
                channel_config,
                node_info,
                msg,
            } => {
                write!(
                    f,
                    "Invalid channel configuration {:?} on node with info: {:?}: custom message: {:?}",
                    channel_config, node_info, msg
                )
            }
            NodeError::ActivationFailed { node_id, error } => {
                if let Some(node_id) = node_id {
                    write!(
                        f,
                        "Node with ID {:?} failed to activate: {}",
                        node_id, error
                    )
                } else {
                    write!(f, "Node failed to activate: {}", error)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ActivateCtxError {
    AlreadyActivated,
    NodeFailedToActived(NodeError),
}

impl Error for ActivateCtxError {}

impl std::fmt::Display for ActivateCtxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivateCtxError::AlreadyActivated => {
                write!(f, "Firewheel context is already activated")
            }
            ActivateCtxError::NodeFailedToActived(e) => {
                write!(f, "Audio node failed to activate: {}", e)
            }
        }
    }
}
