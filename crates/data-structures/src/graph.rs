use std::collections::HashMap;

pub trait Relationship {
    fn nodes(&self) -> Vec<NodeIdentifier>;
}

#[derive(Debug)]
struct Node<T> {
    data: T,
    /// indicies of relationsips referencing this node
    relationships: Vec<usize>,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            relationships: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeIdentifier {
    index: usize,
}

impl NodeIdentifier {
    fn new(index: usize) -> Self {
        Self { index }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationshipIdentifier {
    index: usize,
}

impl RelationshipIdentifier {
    fn new(index: usize) -> Self {
        Self { index }
    }
}

#[derive(Debug)]
pub struct Graph<T, R>
where
    R: Relationship,
{
    nodes: HashMap<usize, Node<T>>,
    relationships: HashMap<usize, R>,
    next_id: usize,
}

impl<T, R> Graph<T, R>
where
    R: Relationship,
{
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            relationships: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn insert_node(&mut self, data: T) -> NodeIdentifier {
        let mut index = &mut self.next_id;
        self.nodes.insert(*index, Node::new(data));
        let identifier = NodeIdentifier::new(*index);
        *index += 1;
        identifier
    }

    pub fn get_node(&self, NodeIdentifier { index, .. }: NodeIdentifier) -> Option<&T> {
        Some(&self.nodes.get(&index)?.data)
    }

    pub fn remove_node(&mut self, NodeIdentifier { index, .. }: NodeIdentifier) -> Option<T> {
        let node = self.nodes.remove(&index)?;
        for index in node.relationships {
            self.relationships.remove(&index);
        }
        Some(node.data)
    }

    pub fn insert_relationship(&mut self, relationship: R) -> RelationshipIdentifier {
        let index = &mut self.next_id;
        let identifier = RelationshipIdentifier::new(*index);
        for NodeIdentifier { index, .. } in relationship.nodes() {
            self.nodes
                .get_mut(&index)
                .unwrap()
                .relationships
                .push(identifier.index);
        }
        self.relationships.insert(*index, relationship);
        *index += 1;
        identifier
    }

    pub fn get_relationship(
        &self,
        RelationshipIdentifier { index, .. }: RelationshipIdentifier,
    ) -> Option<&R> {
        self.relationships.get(&index)
    }

    pub fn remove_relationship(
        &mut self,
        RelationshipIdentifier { index, .. }: RelationshipIdentifier,
    ) -> Option<R> {
        self.relationships.remove(&index)
    }
}

#[derive(Debug, Clone)]
pub struct DirectedRelationship {
    left: NodeIdentifier,
    right: NodeIdentifier,
}

impl DirectedRelationship {
    pub fn new(left: NodeIdentifier, right: NodeIdentifier) -> Self {
        Self { left, right }
    }

    pub fn get_nodes<'a, T>(&self, graph: &'a Graph<T, Self>) -> (&'a T, &'a T) {
        (
            graph.get_node(self.left).unwrap(),
            graph.get_node(self.right).unwrap(),
        )
    }
}

impl Relationship for DirectedRelationship {
    fn nodes(&self) -> Vec<NodeIdentifier> {
        vec![self.left, self.right]
    }
}

#[test]
fn graph() {
    let mut graph = Graph::new();
    let a = graph.insert_node("a");
    let b = graph.insert_node("b");
    let c = graph.insert_node("c");
    let relationship = graph.insert_relationship(DirectedRelationship::new(a, c));
    dbg!(&relationship, &graph);
    graph.remove_node(b);
    dbg!(&relationship, &graph);
    assert_eq!(
        graph
            .get_relationship(relationship)
            .unwrap()
            .get_nodes(&graph),
        (&"a", &"c")
    )
}
