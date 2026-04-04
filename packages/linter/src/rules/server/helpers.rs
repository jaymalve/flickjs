use oxc_syntax::node::NodeId;

pub fn is_inside_route_handler(_node_id: NodeId) -> bool {
    false
}
