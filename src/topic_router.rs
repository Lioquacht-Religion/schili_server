// topic_router.rs

use std::{
    fmt, hash::{DefaultHasher, Hasher}, marker::PhantomData, ops::{Index, IndexMut}, pin::Pin, str::Chars,
};

pub mod macros {
    #[allow(unused_macros)]
    macro_rules! into_async {
        ($func:expr) => {
            |topic_values| Box::pin(async { $func(topic_values).await })
        };
    }
    #[allow(unused_imports)]
    pub(crate) use into_async;

    #[allow(unused_macros)]
    macro_rules! into_async1 {
        ($func:expr) => {
            |topic_values, input1| Box::pin(async { $func(topic_values, input1).await })
        };
    }
    #[allow(unused_imports)]
    pub(crate) use into_async1;

    macro_rules! into_async2 {
        ($func:expr) => {
            |topic_values, input1, input2| {
                Box::pin(async { $func(topic_values, input1, input2).await })
            }
        };
    }
    pub(crate) use into_async2;
}

const ROOT_NODE_ID: NodeIdType = NodeIdType::Ident(NodeId::new(0));

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait StoredFnTrait: Clone {
    type Input<'new>: Copy;
    type Result<'ret>;
    fn exec<'new, 'v>(
        &self,
        topice_values: &'v [TopicValue<'new>],
        input: Self::Input<'new>,
    ) ->  BoxFuture<'v, Self::Result<'v>>;
}

impl StoredFnTrait for for<'o, 'v> fn(&'v [TopicValue<'o>]) -> BoxFuture<'v, anyhow::Result<()>> {
    type Input<'new> = ();
    type Result<'ret> = anyhow::Result<()>;
    fn exec<'new, 'v>(
        &self,
        topice_values: &'v [TopicValue<'new>],
        _input: Self::Input<'new>,
    ) ->  BoxFuture<'v, Self::Result<'v>>{
        self(topice_values)
    }
}

impl<T1: 'static> StoredFnTrait
    for for<'o, 'v> fn(&'v [TopicValue<'o>], &'o T1) -> BoxFuture<'v, anyhow::Result<()>>
{
    type Input<'new> = &'new T1;
    type Result<'ret> = anyhow::Result<()>;
    fn exec<'new, 'v>(
        &self,
        topice_values: &'v [TopicValue<'new>],
        input: Self::Input<'new>,
    ) ->  BoxFuture<'v, Self::Result<'v>>{
        self(topice_values, input)
    }
}

impl<T1: 'static, T2: 'static> StoredFnTrait
    for for<'o, 'v> fn(&'v [TopicValue<'o>], &'o T1, &'o T2) -> BoxFuture<'v, anyhow::Result<()>>
{
    type Input<'new> = (&'new T1, &'new T2);
    type Result<'ret> = anyhow::Result<()>;
    fn exec<'new, 'v>(
        &self,
        topice_values: &'v [TopicValue<'new>],
        input: Self::Input<'new>,
    ) ->  BoxFuture<'v, Self::Result<'v>>{
        self(topice_values, input.0, input.1)
    }
}

///example topic route: {var1}/routepart1/routepart2/[enumval1|enumval2|enumval3]/routepart3
#[derive(Debug)]
pub struct TopicRouter<F> {
    str_buffer: String,
    ident_nodes: IdentNodes<F>,
    var_nodes: VarNodes<F>,
    root: IdentNodeId,
}

#[derive(Debug)]
struct Nodes<T, M>{
    vec: Vec<T>,
    _marker: PhantomData<M>,
}

type IdentNodes<F> = Nodes<IdentNode<F>, IdentNodeMarker>;
type VarNodes<F> = Nodes<VarNode<F>, VarNodeMarker>;

impl<T, M> Nodes<T, M>{
    fn new() -> Self{
        Self { vec: Vec::new(), _marker: PhantomData::default() }
    }
    fn with_vec(vec: Vec<T>) -> Self{
        Self { vec, _marker: PhantomData::default() }
    }
}

impl<T, M> Index<NodeId<M>> for Nodes<T, M>{
    type Output = T;
    fn index(&self, index: NodeId<M>) -> &Self::Output {
        &self.vec[index.id]
    }
}

impl<T, M> IndexMut<NodeId<M>> for Nodes<T, M>{
    fn index_mut(&mut self, index: NodeId<M>) -> &mut Self::Output {
        &mut self.vec[index.id]
    }
}

impl<F: Clone> TopicRouter<F> {
    pub fn new() -> Self {
        let root_node = IdentNode {
            segment: StrSegment {
                kind: StrSegmentKind::Str,
                ident: IdentRange { start: 0, end: 0 },
            },
            handler: None,
            children: ChildNodes::new(),
        };
        Self {
            str_buffer: String::with_capacity(100),
            ident_nodes: Nodes::with_vec(vec![root_node]),
            var_nodes: Nodes::new(),
            root: IdentNodeId::new(0),
        }
    }

    pub fn debug_print(&self) {
        let mut p_node_ids = vec![ROOT_NODE_ID];
        loop {
            if p_node_ids.is_empty() {
                break;
            }
            let mut next_node_ids = Vec::new();
            for p_node_id in p_node_ids.iter() {
                let children = match p_node_id {
                    NodeIdType::Ident(node_id) => &self.ident_nodes[*node_id].children,
                    NodeIdType::Variable(node_id) => &self.var_nodes[*node_id].children,
                };

                if !children.ident_node_ids.node_ids.is_empty() || children.var_node_id.is_some() {
                    print!("p: {}; ", p_node_id.id());
                    for (_hash, c_node_id) in children.ident_node_ids.node_ids.iter() {
                        let node = &self.ident_nodes[*c_node_id];
                        print!("c: {}; s: {}; k: {:?}; ", 
                            c_node_id.id, 
                            node.segment.ident.substr(&self.str_buffer),
                            node.segment.kind
                        );
                        next_node_ids.push(NodeIdType::Ident(*c_node_id));
                    }
                    if let Some(var_node_id) = children.var_node_id {
                        let node = &self.var_nodes[var_node_id];
                        print!("c: {}; d: {:?}; ", 
                            var_node_id.id, 
                            node.segment.datatype
                        );
                        next_node_ids.push(NodeIdType::Variable(var_node_id));
                    }
                    println!("");
                }
            }
            p_node_ids = next_node_ids;
        }
    }

    pub fn parse_route(route: &str) -> Result<Vec<TemplateSegment>, TopicRouteParseError> {
        let mut segments = Vec::new();
        let mut parser = Parser::new(route);
        loop {
            match parser.advance_route_template_parser(route) {
                Ok(Some(segment)) => segments.push(segment),
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(segments)
    }

    fn add_ident_node(
        &mut self,
        child_str: &str,
        str_segment_kind: StrSegmentKind,
    ) -> (Hash, IdentNodeId) {
        let mut hasher = DefaultHasher::new();
        hasher.write(child_str.as_bytes());
        let hash = hasher.finish();
        let start = self.str_buffer.len();
        self.str_buffer.push_str(child_str);
        let end = self.str_buffer.len();
        let child_node_id = IdentNodeId::new(self.ident_nodes.vec.len());
        let child = IdentNode {
            segment: StrSegment {
                kind: str_segment_kind,
                ident: IdentRange { start, end },
            },
            children: ChildNodes::new(),
            handler: None,
        };
        self.ident_nodes.vec.push(child);
        (Hash(hash), child_node_id)
    }

    fn add_variable_node(&mut self, datatype: SegmentVarType) -> VarNodeId {
        let child_node_id = NodeId::new(self.var_nodes.vec.len());
        let child = VarNode {
            segment: VarSegment { datatype },
            children: ChildNodes::new(),
            handler: None,
        };
        self.var_nodes.vec.push(child);
        child_node_id
    }

    fn add_child_segment_to_node(
        &mut self,
        route: &str,
        segment: &TemplateSegment,
        parent_node_id: NodeIdType,
    ) -> NodeInsertResult {
        match segment {
            TemplateSegment::Segment { ident } => NodeInsertResult::SingleIdent(
                self.add_ident_segment(parent_node_id, ident.substr(route), StrSegmentKind::Str),
            ),
            TemplateSegment::Enum { enum_values } => {
                let mut node_ids = Vec::new();
                for ident in enum_values.iter() {
                    node_ids.push(self.add_ident_segment(parent_node_id, ident.substr(route), StrSegmentKind::Enum));
                }
                NodeInsertResult::MultipleIdent(node_ids)
            }
            TemplateSegment::Var { datatype } => {
                NodeInsertResult::Variable(self.add_variable_segment(parent_node_id, *datatype))
            }
        }
    }

    fn add_child_segment_to_enum_parent_nodes(
        &mut self,
        route: &str,
        segment: &TemplateSegment,
        parent_node_ids: &[IdentNodeId],
    ) -> NodeInsertResult {
        match segment {
            TemplateSegment::Segment { ident } => NodeInsertResult::SingleIdent(
                self.add_ident_segment_to_enum_parent_nodes(parent_node_ids, ident.substr(route)),
            ),
            TemplateSegment::Enum { enum_values } => {
                let mut node_ids = Vec::new();
                for ident in enum_values.iter() {
                    node_ids.push(self.add_ident_segment_to_enum_parent_nodes(
                        parent_node_ids,
                        ident.substr(route),
                    ));
                }
                NodeInsertResult::MultipleIdent(node_ids)
            }
            TemplateSegment::Var { datatype } => NodeInsertResult::Variable(
                self.add_var_segment_to_enum_parent_nodes(parent_node_ids, *datatype),
            ),
        }
    }

    fn add_ident_segment_to_enum_parent_nodes(
        &mut self,
        parent_node_ids: &[IdentNodeId],
        child_ident: &str,
    ) -> IdentNodeId {
        let mut child_node_id = None;
        for parent_node_id in parent_node_ids.iter() {
            let parent_children = &self.ident_nodes[*parent_node_id].children;
            if let Some(node_id) = parent_children.ident_node_ids.get(
                &self.str_buffer,
                &self.ident_nodes,
                &child_ident,
            ) {
                child_node_id = Some(node_id);
                break;
            }
        }
        let (hash, child_node_id) = match child_node_id {
            Some(hash_node_id) => hash_node_id,
            None => self.add_ident_node(child_ident, StrSegmentKind::Str),
        };
        for parent_node_id in parent_node_ids.iter() {
            let parent_children = &self.ident_nodes[*parent_node_id].children;
            if let None = parent_children.ident_node_ids.get(
                &self.str_buffer,
                &self.ident_nodes,
                &child_ident,
            ) {
                let parent_children = &mut self.ident_nodes[*parent_node_id].children;
                parent_children.ident_node_ids.insert(hash, child_node_id);
            }
        }
        child_node_id
    }

    fn add_var_segment_to_enum_parent_nodes(
        &mut self,
        parent_node_ids: &[IdentNodeId],
        datatype: SegmentVarType,
    ) -> VarNodeId {
        let mut child_node_id = None;
        for parent_node_id in parent_node_ids.iter() {
            let parent_children = &mut self.ident_nodes[*parent_node_id].children;
            if let Some(node_id) = parent_children.var_node_id {
                child_node_id = Some(node_id);
                break;
            }
        }
        let child_node_id = match child_node_id {
            Some(hash_node_id) => hash_node_id,
            None => self.add_variable_node(datatype),
        };
        for parent_node_id in parent_node_ids.iter() {
            let parent_children = &self.ident_nodes[*parent_node_id].children;
            if let None = parent_children.var_node_id {
                let parent_children = &mut self.ident_nodes[*parent_node_id].children;
                parent_children.var_node_id = Some(child_node_id);
            }
        }
        child_node_id
    }

    fn add_ident_segment(&mut self, parent_node_id: NodeIdType, ident: &str, str_seg_kind: StrSegmentKind) -> IdentNodeId {
        let parent_children = parent_node_id.get_children(self);
        match parent_children
            .ident_node_ids
            .get(&self.str_buffer, &self.ident_nodes, &ident)
        {
            Some((_hash, node_id)) => node_id,
            None => {
                let (hash, child_node_id) = self.add_ident_node(&ident, str_seg_kind);
                let parent_children = parent_node_id.get_children_mut(self);
                parent_children.ident_node_ids.insert(hash, child_node_id);
                child_node_id
            }
        }
    }

    fn add_variable_segment(&mut self, parent_node_id: NodeIdType, datatype: SegmentVarType) -> VarNodeId {
        let parent_children = parent_node_id.get_children(self);
        match parent_children.var_node_id {
            Some(node_id) => {
                let node = &mut self.var_nodes[node_id];
                if let (SegmentVarType::Str, SegmentVarType::Integer) =
                    (datatype, node.segment.datatype)
                {
                    node.segment.datatype = datatype;
                };
                node_id
            }
            None => {
                let child_node_id = self.add_variable_node(datatype);
                let parent_children = parent_node_id.get_children_mut(self);
                parent_children.var_node_id = Some(child_node_id);
                child_node_id
            }
        }
    }

    pub fn add_route(&mut self, route: &str, handler: F) -> Result<(), TopicRouteParseError> {
        let mut parser = Parser::new(route);
        let mut cur_node_result = NodeInsertResult::SingleIdent(self.root);

        loop {
            let segment = parser.advance_route_template_parser(route)?;
            if let Some(segment) = segment {
                cur_node_result = match cur_node_result {
                    NodeInsertResult::SingleIdent(node_id) => {
                        self.add_child_segment_to_node(route, &segment, NodeIdType::Ident(node_id))
                    }
                    NodeInsertResult::MultipleIdent(node_ids) => {
                        self.add_child_segment_to_enum_parent_nodes(route, &segment, &node_ids)
                    }
                    NodeInsertResult::Variable(node_id) => {
                        self.add_child_segment_to_node(route, &segment, NodeIdType::Variable(node_id))
                    }
                }
            } else {
                match cur_node_result {
                    NodeInsertResult::SingleIdent(node_id) => {
                        self.ident_nodes[node_id].handler = Some(handler)
                    }
                    NodeInsertResult::MultipleIdent(node_ids) => {
                        for node_id in node_ids {
                            self.ident_nodes[node_id].handler = Some(handler.clone());
                        }
                    }
                    NodeInsertResult::Variable(node_id) => {
                        self.var_nodes[node_id].handler = Some(handler)
                    }
                }
                break;
            }
        }
        Ok(())
    }
}

impl<F: StoredFnTrait> TopicRouter<F> {

    pub async fn exec_handler_for_route<'r>(
        &self,
        input: &F::Input<'r>,
        route: &'r str,
    ) -> Result<(), TopicRouteExecError<F::Result<'r>>> {
        let mut parser = Parser::new(route);
        self.exec_handler_for_route_inner(input, route, &mut parser, ROOT_NODE_ID).await
    }

    async fn exec_handler_for_route_inner<'r: 'p, 'p>(
        &self,
        input: &F::Input<'r>,
        route: &'r str,
        parser: &'p mut Parser<'r>,
        mut cur_parent_node_id: NodeIdType,
    ) -> Result<(), TopicRouteExecError<F::Result<'r>>> {
        let mut topic_values = Vec::new();
        loop {
            let segment = parser.advance_topic_parser(route)
                .map_err(|e| TopicRouteExecError::ParseError(e))?;
            if let Some(ValueSegment {
                value_type,
                str_range,
            }) = segment
            {
                let segment_str = str_range.substr(route);
                cur_parent_node_id = if let Some(next_node_id) = self.exec_nodes_handler(
                    input,
                    route,
                    parser,
                    cur_parent_node_id,
                    value_type,
                    segment_str,
                    &mut topic_values,
                ).await?{
                    next_node_id
                } else {
                    break;
                };
            } else {
                break;
            }
        }
        Ok(())
    }

    async fn exec_nodes_handler<'r: 'p + 't, 'p, 't>(
        &self,
        input: &F::Input<'r>,
        route: &'r str,
        parser: &'p mut Parser<'r>,
        parent_node_id: NodeIdType,
        value_type: SegmentValType,
        segment_str: &'r str,
        topic_values: &'t mut Vec<TopicValue<'r>>,
    ) -> Result<Option<NodeIdType>, TopicRouteExecError<F::Result<'r>>> {
        let children = parent_node_id.get_children(self);
        let ident_res = if let Some((_hash, node_id)) =
            children
                .ident_node_ids
                .get(&self.str_buffer, &self.ident_nodes, segment_str)
        {
            let child_node = &self.ident_nodes[node_id];
            match child_node.segment.kind {
                StrSegmentKind::Str => (),
                StrSegmentKind::Enum => topic_values.push(TopicValue::EnumVar(segment_str)),
            }
            if let Some(handler) = &child_node.handler {
                //TODO: propegate error, change trait exec function return type
                let e = handler.exec(topic_values, *input).await;
                    //.map_err(|e| Err(TopicRouteExecError::HandlerError(e)))?;
            }
            Some(NodeIdType::Ident(node_id))
        } else {
            None
        };

        let var_res = if let Some(var_node_id) = children.var_node_id {
            let child_node = &self.var_nodes[var_node_id];

            let int_var = matches!(child_node.segment.datatype, SegmentVarType::Integer)
                && matches!(value_type, SegmentValType::Integer);

            let str_var = matches!(child_node.segment.datatype, SegmentVarType::Str)
                && matches!(value_type, SegmentValType::Ident);

            if int_var || str_var {
                if str_var {
                    topic_values.push(TopicValue::StrVar(segment_str));
                } else if int_var {
                    //TODO: should i64 parsing be done here with error handling?
                    topic_values.push(TopicValue::IntVar(segment_str.parse().unwrap_or(0)));
                }
                if let Some(handler) = &child_node.handler {
                    let e = handler.exec(topic_values, *input).await;
                }
                Some(NodeIdType::Variable(var_node_id))
            } else {
                None
            }
        } else {
            None
        };

        match (ident_res, var_res) {
            (Some(ident_node_id), Some(var_node_id)) => {
                let e1 = Box::pin(self.exec_handler_for_route_inner(input, route, &mut parser.clone(), ident_node_id)).await;
                let e2 = Box::pin(self.exec_handler_for_route_inner(input, route, &mut parser.clone(), var_node_id)).await;
                match (e1, e2){
                    (Ok(()), Ok(())) => Ok(None),
                    (Err(e1), Ok(())) => Err(e1),
                    (Ok(()), Err(e2)) => Err(e2),
                    (
                        Err(e1),
                        Err(e2)
                    ) => Err(TopicRouteExecError::MultiError(vec![e1, e2])),
                }
            }
            (Some(ident_node_id), None) => Ok(Some(ident_node_id)),
            (None, Some(var_node_id)) => Ok(Some(var_node_id)),
            (None, None) => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeIdType {
    Ident(IdentNodeId),
    Variable(VarNodeId),
}

impl NodeIdType{
    fn id(&self) -> usize{
        match self{
            NodeIdType::Ident(node_id) => node_id.id,
            NodeIdType::Variable(node_id) => node_id.id,
        }
    }

    fn get_children<'a, 'b, F>(&'a self, router: &'b TopicRouter<F>) -> &'b ChildNodes{
        match self{
            NodeIdType::Ident(id) => &router.ident_nodes[*id].children,
            NodeIdType::Variable(id) => &router.var_nodes[*id].children,
        }
    }

    fn get_children_mut<'a, 'b, F>(&'a self, router: &'b mut TopicRouter<F>) -> &'b mut ChildNodes{
        match self{
            NodeIdType::Ident(id) => &mut router.ident_nodes[*id].children,
            NodeIdType::Variable(id) => &mut router.var_nodes[*id].children,
        }
    }
}

enum NodeInsertResult {
    SingleIdent(IdentNodeId),
    MultipleIdent(Vec<IdentNodeId>),
    Variable(VarNodeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeId<T>{
    id: usize,
    _marker: PhantomData<T>,
}

impl<T> NodeId<T>{
    const fn new(id: usize) -> Self{
        Self { id, _marker: PhantomData }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IdentNodeMarker;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VarNodeMarker;

type IdentNodeId = NodeId<IdentNodeMarker>;
type VarNodeId = NodeId<VarNodeMarker>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentVarType {
    Str,
    Integer,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TemplateSegment {
    Segment { ident: IdentRange },
    Enum { enum_values: Vec<IdentRange> },
    Var { datatype: SegmentVarType },
}

struct ValueSegment {
    value_type: SegmentValType,
    str_range: IdentRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentValType {
    Ident,
    Integer,
}

#[derive(Debug, PartialEq, Eq)]
pub struct IdentRange {
    start: usize,
    end: usize,
}

impl IdentRange {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn substr<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StrSegmentKind {
    Str,
    Enum,
}

#[derive(Debug, PartialEq, Eq)]
struct StrSegment {
    kind: StrSegmentKind,
    ident: IdentRange,
}

#[derive(Debug)]
struct VarSegment {
    datatype: SegmentVarType,
}

#[derive(Debug)]
pub enum TopicValue<'a> {
    IntVar(i64),
    StrVar(&'a str),
    EnumVar(&'a str),
}

struct NodeInner<T, F> {
    segment: T,
    handler: Option<F>,
    children: ChildNodes,
}

impl<T, F> fmt::Debug for NodeInner<T, F> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //TODO
        Ok(())
    }
}

type IdentNode<F> = NodeInner<StrSegment, F>;

type VarNode<F> = NodeInner<VarSegment, F>;

#[derive(Debug)]
struct ChildNodes {
    ident_node_ids: SortedNodeIds,
    var_node_id: Option<VarNodeId>,
}

impl ChildNodes {
    fn new() -> Self {
        Self {
            ident_node_ids: SortedNodeIds::new(),
            var_node_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hash(u64);

#[derive(Debug)]
struct SortedNodeIds {
    node_ids: Vec<(Hash, IdentNodeId)>,
}

impl SortedNodeIds {
    fn new() -> Self {
        Self {
            node_ids: Vec::new(),
        }
    }

    fn insert(&mut self, hash: Hash, node_id: IdentNodeId) {
        self.node_ids.push((hash, node_id));
        self.node_ids.sort_by_key(|n| n.0.0);
    }

    fn get<I>(
        &self,
        str_buffer: &str,
        nodes: &IdentNodes<I>,
        segment: &str,
    ) -> Option<(Hash, IdentNodeId)> {
        let mut hasher = DefaultHasher::new();
        hasher.write(segment.as_bytes());
        let hash = hasher.finish();
        match self
            .node_ids
            .binary_search_by_key(&hash, |hash_node_id| hash_node_id.0.0)
        {
            Ok(index) => {
                for (_hash, node_id) in self.node_ids[..index].iter().rev() {
                    let str_seg = nodes[*node_id].segment.ident.substr(&str_buffer);
                    if str_seg == segment {
                        return Some((Hash(hash), *node_id));
                    } else {
                        break;
                    }
                }
                for (_hash, node_id) in self.node_ids[index..].iter() {
                    let str_seg = nodes[*node_id].segment.ident.substr(&str_buffer);
                    if str_seg == segment {
                        return Some((Hash(hash), *node_id));
                    } else {
                        break;
                    }
                }
                None
            }
            Err(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Cursor<'a> {
    chars: Chars<'a>,
    len_remaining: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    Integer,
    OpenSquareBrace,
    CloseSquareBrace,
    OpenCurlyBrace,
    CloseCurlyBrace,
    Pipe,
    Slash,
    Whitespace,
    Unknown,
    EOF,
}

const EOF_CHAR: char = '\0';

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    kind: TokenKind,
    len: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TopicRouteExecError<R> {
    ParseError(TopicRouteParseError),
    HandlerError(R),
    MultiError(Vec<TopicRouteExecError<R>>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TopicRouteParseError {
    UnexpectedToken(Token),
    ExpectedDiffToken { expected: TokenKind, found: Token },
    VarTypeDoesNotExist { invalid: Token },
}

#[derive(Debug, Clone)]
struct Parser<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Parser<'a> {
    fn new(route: &'a str) -> Parser<'a> {
        Parser {
            cursor: Cursor::new(route),
        }
    }

    fn bump(&mut self) -> Token {
        self.cursor.advance_token()
    }

    fn first(&mut self) -> Token {
        self.cursor.clone().advance_token()
    }

    fn eat_while(&mut self, predicate: impl Fn(Token) -> bool) {
        while predicate(self.first()) && !self.cursor.is_eof() {
            self.bump();
        }
    }

    fn eat_whitespace(&mut self) {
        self.eat_while(|t| matches!(t.kind, TokenKind::Whitespace));
    }

    fn get_token_range(&self, prev_token_len: usize, token: &Token) -> IdentRange {
        let start = prev_token_len;
        let end = token.len as usize;
        IdentRange::new(start, end)
    }

    fn get_token_str(&self, route: &'a str, prev_token_len: usize, token: &Token) -> &'a str {
        let start = prev_token_len;
        let end = token.len as usize;
        let token_str = &route[start..end];
        token_str
    }

    fn advance_route_template_parser(
        &mut self,
        route: &str,
    ) -> Result<Option<TemplateSegment>, TopicRouteParseError> {
        let cur_pos = self.cursor.pos_within_token() as usize;
        let token = self.bump();
        match token.kind {
            TokenKind::Ident | TokenKind::Integer => self.parse_template_ident(cur_pos, &token),
            TokenKind::OpenSquareBrace => self.parse_enum(),
            TokenKind::OpenCurlyBrace => self.parse_var(route),
            TokenKind::Slash => {
                self.eat_whitespace();
                self.advance_route_template_parser(route)
            }
            TokenKind::Whitespace => {
                self.eat_whitespace();
                self.advance_route_template_parser(route)
            }
            TokenKind::CloseSquareBrace
            | TokenKind::CloseCurlyBrace
            | TokenKind::Unknown
            | TokenKind::Pipe => Err(TopicRouteParseError::UnexpectedToken(token)),
            TokenKind::EOF => return Ok(None),
        }
    }

    fn advance_topic_parser(
        &mut self,
        route: &str,
    ) -> Result<Option<ValueSegment>, TopicRouteParseError> {
        let cur_pos = self.cursor.pos_within_token() as usize;
        let token = self.bump();
        match token.kind {
            TokenKind::Ident => self.parse_topic_ident(cur_pos, &token),
            TokenKind::Integer => self.parse_topic_integer(cur_pos, &token),
            TokenKind::Slash => {
                self.eat_whitespace();
                self.advance_topic_parser(route)
            }
            TokenKind::Whitespace => {
                self.eat_whitespace();
                self.advance_topic_parser(route)
            }
            TokenKind::OpenSquareBrace
            | TokenKind::OpenCurlyBrace
            | TokenKind::CloseSquareBrace
            | TokenKind::CloseCurlyBrace
            | TokenKind::Unknown
            | TokenKind::Pipe => Err(TopicRouteParseError::UnexpectedToken(token)),
            TokenKind::EOF => return Ok(None),
        }
    }

    fn parse_template_ident(
        &mut self,
        prev_token_len: usize,
        token: &Token,
    ) -> Result<Option<TemplateSegment>, TopicRouteParseError> {
        self.eat_whitespace();
        let next_token = self.bump();
        match next_token.kind {
            TokenKind::Slash | TokenKind::EOF => Ok(Some(TemplateSegment::Segment {
                ident: self.get_token_range(prev_token_len, token),
            })),
            _ => Err(TopicRouteParseError::UnexpectedToken(next_token)),
        }
    }

    fn parse_topic_ident(
        &mut self,
        prev_token_len: usize,
        token: &Token,
    ) -> Result<Option<ValueSegment>, TopicRouteParseError> {
        self.parse_topic_segment(prev_token_len, token, SegmentValType::Ident)
    }

    fn parse_topic_integer(
        &mut self,
        prev_token_len: usize,
        token: &Token,
    ) -> Result<Option<ValueSegment>, TopicRouteParseError> {
        self.parse_topic_segment(prev_token_len, token, SegmentValType::Integer)
    }

    fn parse_topic_segment(
        &mut self,
        prev_token_len: usize,
        token: &Token,
        value_type: SegmentValType,
    ) -> Result<Option<ValueSegment>, TopicRouteParseError> {
        self.eat_whitespace();
        let next_token = self.bump();
        match next_token.kind {
            TokenKind::Slash | TokenKind::EOF => Ok(Some(ValueSegment {
                value_type: value_type,
                str_range: self.get_token_range(prev_token_len, token),
            })),
            _ => Err(TopicRouteParseError::UnexpectedToken(next_token)),
        }
    }

    fn parse_enum(&mut self) -> Result<Option<TemplateSegment>, TopicRouteParseError> {
        let mut enum_values = Vec::new();
        loop {
            self.eat_whitespace();
            let cur_pos = self.cursor.pos_within_token() as usize;
            let token = self.bump();
            if let TokenKind::Ident = token.kind {
                enum_values.push(self.get_token_range(cur_pos, &token));
                self.eat_whitespace();
                let token = self.bump();
                match token.kind {
                    TokenKind::Pipe => continue,
                    TokenKind::CloseSquareBrace => {
                        return Ok(Some(TemplateSegment::Enum { enum_values }));
                    }
                    _ => return Err(TopicRouteParseError::UnexpectedToken(token)),
                }
            } else {
                return Err(TopicRouteParseError::UnexpectedToken(token));
            }
        }
    }

    fn parse_var(&mut self, route: &str) -> Result<Option<TemplateSegment>, TopicRouteParseError> {
        self.eat_whitespace();
        let cur_pos = self.cursor.pos_within_token() as usize;
        let token = self.bump();
        if let TokenKind::Ident = token.kind {
            let datatype = match self.get_token_str(route, cur_pos, &token) {
                "str" => SegmentVarType::Str,
                "integer" => SegmentVarType::Integer,
                _ => return Err(TopicRouteParseError::VarTypeDoesNotExist { invalid: token }),
            };
            self.eat_whitespace();
            let token = self.bump();
            if let TokenKind::CloseCurlyBrace = token.kind {
                Ok(Some(TemplateSegment::Var { datatype }))
            } else {
                Err(TopicRouteParseError::ExpectedDiffToken {
                    expected: TokenKind::CloseCurlyBrace,
                    found: token,
                })
            }
        } else {
            Err(TopicRouteParseError::ExpectedDiffToken {
                expected: TokenKind::Ident,
                found: token,
            })
        }
    }
}

impl<'a> Cursor<'a> {
    fn new(src: &'a str) -> Cursor<'a> {
        Self {
            len_remaining: src.len(),
            chars: src.chars(),
        }
    }

    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    fn bump(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn eat_while(&mut self, predicate: impl Fn(char) -> bool) {
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }

    fn white_space(&mut self) -> TokenKind {
        self.eat_while(char::is_whitespace);
        TokenKind::Whitespace
    }

    fn is_ident_start(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }

    fn is_ident_continue(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
    }

    fn ident(&mut self) -> TokenKind {
        self.eat_while(Self::is_ident_continue);
        TokenKind::Ident
    }

    fn number(&mut self) -> TokenKind {
        self.eat_while(Self::is_number_continue);
        TokenKind::Integer
    }

    fn is_number_start(c: char) -> bool {
        matches!(c, '0'..='9')
    }

    fn is_number_continue(c: char) -> bool {
        matches!(c, '0'..='9' | '_')
    }

    fn pos_within_token(&self) -> u32 {
        (self.len_remaining - self.chars.as_str().len()) as u32
    }

    fn advance_token(&mut self) -> Token {
        let Some(first_char) = self.chars.next() else {
            return Token {
                kind: TokenKind::EOF,
                len: 0,
            };
        };

        let kind: TokenKind = match first_char {
            '/' => TokenKind::Slash,
            '[' => TokenKind::OpenSquareBrace,
            ']' => TokenKind::CloseSquareBrace,
            '{' => TokenKind::OpenCurlyBrace,
            '}' => TokenKind::CloseCurlyBrace,
            '|' => TokenKind::Pipe,
            c if c.is_whitespace() => self.white_space(),
            c if Self::is_ident_start(c) => self.ident(),
            c if Self::is_number_start(c) => self.number(),
            _ => TokenKind::Unknown,
        };
        Token {
            kind,
            len: self.pos_within_token(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::{self, AtomicIsize};

use anyhow::anyhow;

use crate::topic_router::{
        BoxFuture, IdentRange, SegmentVarType, TemplateSegment, TopicRouter, TopicValue, macros::into_async1,
    };

    fn assert_substr(route: &str, substr: &str, start: usize, end: usize) -> IdentRange {
        let ident = IdentRange::new(start, end);
        assert_eq!(substr, ident.substr(route));
        ident
    }

    #[test]
    fn url_router_test() {
        let topic1 =
            "{integer}/routepart1/{str}/routepart2/[enumval1|enumval2|enumval3]/routepart3";
        let segments = TopicRouter::<()>::parse_route(topic1);
        dbg!(&segments);
        assert_eq!(
            Ok(vec![
                TemplateSegment::Var {
                    datatype: SegmentVarType::Integer
                },
                TemplateSegment::Segment {
                    ident: assert_substr(topic1, "routepart1", 10, 20)
                },
                TemplateSegment::Var {
                    datatype: SegmentVarType::Str
                },
                TemplateSegment::Segment {
                    ident: assert_substr(topic1, "routepart2", 27, 37)
                },
                TemplateSegment::Enum {
                    enum_values: vec![
                        assert_substr(topic1, "enumval1", 39, 47),
                        assert_substr(topic1, "enumval2", 48, 56),
                        assert_substr(topic1, "enumval3", 57, 65)
                    ]
                },
                TemplateSegment::Segment {
                    ident: assert_substr(topic1, "routepart3", 67, 77)
                },
            ]),
            segments
        );
        let topic2 = "routepart1/routepart2/[enumval1|enumval2]";
        let segments = TopicRouter::<fn() -> ()>::parse_route(topic2);
        assert_eq!(
            Ok(vec![
                TemplateSegment::Segment {
                    ident: assert_substr(topic2, "routepart1", 0, 10)
                },
                TemplateSegment::Segment {
                    ident: assert_substr(topic2, "routepart2", 11, 21)
                },
                TemplateSegment::Enum {
                    enum_values: vec![
                        assert_substr(topic2, "enumval1", 23, 31),
                        assert_substr(topic2, "enumval2", 32, 40)
                    ]
                },
            ]),
            segments
        );
        dbg!(&segments);

        let topic3 = "routepart1/routepart2/routepart3/routpart4";

        async fn print_topic1(topic_values: &[TopicValue<'_>], counter: &AtomicIsize) -> anyhow::Result<()> {
            dbg!(topic_values);
            if let [TopicValue::IntVar(_), TopicValue::StrVar(_), TopicValue::EnumVar(_)] = topic_values{
                counter.fetch_add(1, atomic::Ordering::Relaxed);
                println!("test topic 1");
                Ok(())
            }
            else{
                Err(anyhow!("Unexpected topic values received: {:?}", topic_values))
            }
        }
        async fn print_topic2(_topic_values: &[TopicValue<'_>], counter: &AtomicIsize) -> anyhow::Result<()> {
            counter.fetch_add(2, atomic::Ordering::Relaxed);
            println!("test topic 2");
            Ok(())
        }
        async fn print_topic3(_topic_values: &[TopicValue<'_>], counter: &AtomicIsize) -> anyhow::Result<()> {
            counter.fetch_add(3, atomic::Ordering::Relaxed);
            println!("test topic 3");
            Ok(())
        }

        let mut router: TopicRouter<
            for<'o, 'v> fn(&'v [TopicValue<'o>], &'o AtomicIsize) -> BoxFuture<'v, anyhow::Result<()>>,
        > = TopicRouter::new();

        assert_eq!(Ok(()), router.add_route(topic1, into_async1!(print_topic1)));
        router.debug_print();
        assert_eq!(Ok(()), router.add_route(topic2, into_async1!(print_topic2)));
        router.debug_print();
        assert_eq!(Ok(()), router.add_route(topic3, into_async1!(print_topic3)));
        router.debug_print();
        let topic1_1 = "42/routepart1/str_var1/routepart2/enumval1/routepart3";
        let topic1_2 = "9102988/routepart1/str23j_var2/routepart2/enumval2/routepart3";
        let topic1_3 = "3242/routepart1/str_334dfvar3/routepart2/enumval3/routepart3";

        smol::block_on(async {
            let counter = AtomicIsize::new(0);
            assert!(dbg!(
                    router.exec_handler_for_route(&&counter, topic1_1).await.is_ok()
            ));
            assert_eq!(1, counter.load(atomic::Ordering::Relaxed));
            assert!(router.exec_handler_for_route(&&counter, topic1_2).await.is_ok());
            assert_eq!(2, counter.load(atomic::Ordering::Relaxed));
            assert!(router.exec_handler_for_route(&&counter, topic1_3).await.is_ok());
            assert_eq!(3, counter.load(atomic::Ordering::Relaxed));

            let topic2_1 = "routepart1/routepart2/enumval1";
            let topic2_2 = "routepart1/routepart2/enumval2";

            assert!(router.exec_handler_for_route(&&counter, topic2_1).await.is_ok());
            assert_eq!(5, counter.load(atomic::Ordering::Relaxed));
            assert!(router.exec_handler_for_route(&&counter, topic2_2).await.is_ok());
            assert_eq!(7, counter.load(atomic::Ordering::Relaxed));
            assert!(router.exec_handler_for_route(&&counter, topic3).await.is_ok());
            assert_eq!(10, counter.load(atomic::Ordering::Relaxed));
        });
        assert!(false);

    }
}
