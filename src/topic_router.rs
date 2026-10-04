// topic_router.rs

use std::{hash::{DefaultHasher, Hasher}, str::Chars};

///example topic route: {var1}/routepart1/routepart2/[enumval1|enumval2|enumval3]/routepart3
#[derive(Debug)]
struct TopicRouter{
    str_buffer: String,
    ident_nodes: Vec<IdentNode>,
    var_nodes: Vec<VarNode>,
    root: NodeId,
}

impl TopicRouter{
    fn new() -> Self{
        let root_node = IdentNode{
            segment: StrSegment { ident: IdentRange { start: 0, end: 0 } },
            handler: None,
            children: ChildNodes::new(),
        };
        Self {
            str_buffer: String::with_capacity(100),
            ident_nodes: vec![root_node],
            var_nodes: Vec::new(),
            root: NodeId(0),
        }
    }

    fn debug_print(&self){
        println!("str_buffer: {}", &self.str_buffer);
        let mut p_node_ids = vec![Self::ROOT_NODE_ID];
        loop{
            if p_node_ids.is_empty(){
                break;
            }
            let mut next_node_ids = Vec::new();
            for p_node_id in p_node_ids.iter(){
                let children= match p_node_id{
                    NodeIdType::Ident(node_id) => &self.ident_nodes[node_id.0].children,
                    NodeIdType::Variable(node_id) => &self.var_nodes[node_id.0].children,
                };

                print!("p: {}:", p_node_id.id().0);
                print!("{{");
                for (hash, c_node_id) in children.ident_node_ids.node_ids.iter(){
                    let node = &self.ident_nodes[c_node_id.0];
                    print!(
                        "p:{}; c:{}; s: '{}'; h: {}; ",
                        p_node_id.id().0,
                        c_node_id.0,
                        node.segment.ident.substr(&self.str_buffer),
                        hash.0
                    );
                    next_node_ids.push(NodeIdType::Ident(*c_node_id));
                }
                if let Some(var_node_id) = children.var_node_id{
                    let node = &self.var_nodes[var_node_id.0];
                    print!(
                        "p:{}; c:{}; v: '{:?}'; ",
                        p_node_id.id().0,
                        var_node_id.0,
                        node.segment.datatype
                    );
                    next_node_ids.push(NodeIdType::Variable(var_node_id));
                }
                println!("}}");
            }
            p_node_ids = next_node_ids;
        }
    }

    fn parse_route(route: &str) -> Result<Vec<TemplateSegment>, TopicRouteParseError>{
        let mut segments = Vec::new();
        let mut parser = Parser::new(route);
        loop{
            match parser.advance_route_template_parser(route){
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

    fn add_ident_node(&mut self, child_str: &str) -> (Hash, NodeId){
        let mut hasher = DefaultHasher::new();
        hasher.write(child_str.as_bytes());
        let hash = hasher.finish();
        let start = self.str_buffer.len();
        self.str_buffer.push_str(child_str);
        let end = self.str_buffer.len();
        let child_node_id = NodeId(self.ident_nodes.len());
        let child = IdentNode{
            segment: StrSegment { ident: IdentRange { start, end } },
            children: ChildNodes::new(),
            handler: None
        };
        self.ident_nodes.push(child);
        (Hash(hash), child_node_id)
    }

    fn add_variable_node(&mut self, datatype: SegmentVarType) -> NodeId{
        let child_node_id = NodeId(self.var_nodes.len());
        let child = VarNode{
            segment: VarSegment { datatype },
            children: ChildNodes::new(),
            handler: None, 
        };
        self.var_nodes.push(child);
        child_node_id
    }

    fn add_child_segment_to_node(
        &mut self, route: &str, segment: &TemplateSegment, 
        parent_node_id: NodeId,
    ) -> NodeInsertResult{
        match segment{
            TemplateSegment::Segment { ident } =>
                NodeInsertResult::SingleIdent(
                    self.add_ident_segment(
                        parent_node_id, ident.substr(route))),
            TemplateSegment::Enum { enum_values } => {
                let mut node_ids = Vec::new();
                for ident in enum_values.iter(){
                    node_ids.push(
                        self.add_ident_segment(
                            parent_node_id, ident.substr(route)));
                }
                NodeInsertResult::MultipleIdent(node_ids)
            }
            TemplateSegment::Var { datatype } => 
                NodeInsertResult::Variable(self.add_variable_segment(parent_node_id, *datatype)),
        }
    }

    fn add_child_segment_to_enum_parent_nodes(
        &mut self, route: &str, segment: &TemplateSegment, 
        parent_node_ids: &[NodeId],
    ) -> NodeInsertResult{
        match segment{
            TemplateSegment::Segment { ident } => 
                NodeInsertResult::SingleIdent(
                    self.add_ident_segment_to_enum_parent_nodes(
                        parent_node_ids, ident.substr(route))
                ),
            TemplateSegment::Enum { enum_values } => {
                let mut node_ids = Vec::new();
                for ident in enum_values.iter(){
                    node_ids.push(
                        self.add_ident_segment_to_enum_parent_nodes(
                            parent_node_ids, ident.substr(route))
                    );
                }
                NodeInsertResult::MultipleIdent(node_ids)
            }
            TemplateSegment::Var { datatype } => 
                NodeInsertResult::Variable(self.add_var_segment_to_enum_parent_nodes(parent_node_ids, *datatype)),
        }
    }

    fn add_ident_segment_to_enum_parent_nodes(
        &mut self, 
        parent_node_ids: &[NodeId],
        child_ident: &str,
    ) -> NodeId{
        let mut child_node_id = None;
        for parent_node_id in parent_node_ids.iter(){
            let parent_children = &self.ident_nodes[parent_node_id.0].children;
            if let Some(node_id) = parent_children.ident_node_ids.get(&self.str_buffer, &self.ident_nodes, &child_ident){
                child_node_id = Some(node_id);
                break;
            }
        };
        let (hash, child_node_id) = match child_node_id{
            Some(hash_node_id) => hash_node_id,
            None => 
                self.add_ident_node(child_ident),
        };
        for parent_node_id in parent_node_ids.iter(){
            let parent_children = &self.ident_nodes[parent_node_id.0].children;
            if let None = parent_children.ident_node_ids.get(&self.str_buffer, &self.ident_nodes, &child_ident){
                let parent_children = &mut self.ident_nodes[parent_node_id.0].children;
                parent_children.ident_node_ids.insert(hash, child_node_id);
            }
        }
        child_node_id
    }

    fn add_var_segment_to_enum_parent_nodes(
        &mut self, 
        parent_node_ids: &[NodeId],
        datatype: SegmentVarType,
    ) -> NodeId{
        let mut child_node_id = None;
        for parent_node_id in parent_node_ids.iter(){
            let parent_children = &mut self.ident_nodes[parent_node_id.0].children;
            if let Some(node_id) = parent_children.var_node_id{
                child_node_id = Some(node_id);
                break;
            }
        };
        let child_node_id = match child_node_id{
            Some(hash_node_id) => hash_node_id,
            None => 
                self.add_variable_node(datatype),
        };
        for parent_node_id in parent_node_ids.iter(){
            let parent_children = &self.ident_nodes[parent_node_id.0].children;
            if let None = parent_children.var_node_id{
                let parent_children = &mut self.ident_nodes[parent_node_id.0].children;
                parent_children.var_node_id = Some(child_node_id);
            }
        }
        child_node_id
    }

    fn add_ident_segment(
        &mut self, 
        parent_node_id: NodeId,
        ident: &str,
    ) -> NodeId{
        let parent_children = &self.ident_nodes[parent_node_id.0].children;
        match parent_children.ident_node_ids.get(&self.str_buffer, &self.ident_nodes, &ident){
            Some((_hash, node_id)) => node_id,
            None => {
                let (hash, child_node_id) = self.add_ident_node(&ident);
                let parent = &mut self.ident_nodes[parent_node_id.0];
                parent.children.ident_node_ids.insert(hash, child_node_id);
                child_node_id
            }
        }
    }

    fn add_variable_segment(
        &mut self, 
        parent_node_id: NodeId,
        datatype: SegmentVarType,
    ) -> NodeId{
        let parent_children = &self.ident_nodes[parent_node_id.0].children;
        match parent_children.var_node_id{
            Some(node_id) => {
                let node = &mut self.var_nodes[node_id.0];
                if let (SegmentVarType::Str, SegmentVarType::Integer) = (datatype, node.segment.datatype){
                    node.segment.datatype = datatype;
                };
                node_id
            }
            None => {
                let child_node_id = self.add_variable_node(datatype);
                let parent = &mut self.ident_nodes[parent_node_id.0];
                parent.children.var_node_id = Some(child_node_id);
                child_node_id
            }
        }
    }

    fn add_route(&mut self, route: &str, handler: Option<NodeHandler>) -> Result<(), TopicRouteParseError>{
        let mut parser = Parser::new(route);
        let mut cur_node_result= NodeInsertResult::SingleIdent(self.root);

        loop{
            let segment = parser.advance_route_template_parser(route)?;
            if let Some(segment) = segment {
                cur_node_result = match cur_node_result{
                    NodeInsertResult::SingleIdent(node_id) => 
                        self.add_child_segment_to_node(
                            route, &segment, node_id),
                    NodeInsertResult::MultipleIdent(node_ids) => 
                        self.add_child_segment_to_enum_parent_nodes(
                            route, &segment, &node_ids),
                    NodeInsertResult::Variable(node_id) => 
                        self.add_child_segment_to_node(
                            route, &segment, node_id),
                }
            }
            else{
                match cur_node_result{
                    NodeInsertResult::SingleIdent(node_id) => 
                        self.ident_nodes[node_id.0].handler = handler,
                    NodeInsertResult::MultipleIdent(node_ids) => {
                        for node_id in node_ids{
                            self.ident_nodes[node_id.0].handler = handler;
                        }
                    }
                    NodeInsertResult::Variable(node_id) => 
                        self.var_nodes[node_id.0].handler = handler,
                }
                break;
            }
        }
        Ok(())
    }

    const ROOT_NODE_ID: NodeIdType = NodeIdType::Ident(NodeId(0));

    fn exec_handler_for_route(&self, route: &str) -> Result<(), TopicRouteParseError>{
        let mut parser = Parser::new(route);
        self.exec_handler_for_route_inner(
            route, &mut parser, Self::ROOT_NODE_ID)
    }

    fn exec_handler_for_route_inner(
        &self, route: &str, parser: &mut Parser, mut cur_parent_node_id: NodeIdType
    ) -> Result<(), TopicRouteParseError>{
        loop{
            let segment = parser.advance_topic_parser(route)?;
            if let Some(ValueSegment {
                value_type,
                str_range 
            }) = segment{
                let segment_str = str_range.substr(route);
                cur_parent_node_id = if let Some(next_node_id) = 
                    self.exec_nodes_handler(
                        route, parser, cur_parent_node_id, value_type, segment_str)
                {
                    next_node_id
                }
                else{
                    break;
                };
            }
            else{
                break;
            }
        }
        Ok(())
    }

    fn exec_nodes_handler(
        &self,
        route: &str,
        parser: &mut Parser,
        parent_node_id: NodeIdType, 
        value_type: SegmentValType,
        segment_str: &str,

    ) -> Option<NodeIdType>{
        let children= match parent_node_id{
            NodeIdType::Ident(id) => &self.ident_nodes[id.0].children,
            NodeIdType::Variable(id) => &self.var_nodes[id.0].children,
        }; 
        let ident_res = if let Some((_hash, node_id)) = children.ident_node_ids.get(
            &self.str_buffer, &self.ident_nodes, segment_str
        ){
            let child_node = &self.ident_nodes[node_id.0];
            if let Some(handler) = child_node.handler{
                handler();
            }
            Some(NodeIdType::Ident(node_id))
        }
        else{
            None
        };

        let var_res = if let Some(var_node_id) = children.var_node_id{
            let child_node = &self.var_nodes[var_node_id.0];
            if (matches!(child_node.segment.datatype, SegmentVarType::Integer)
                && matches!(value_type, SegmentValType::Integer))
                || (matches!(child_node.segment.datatype, SegmentVarType::Str)
                && matches!(value_type, SegmentValType::Ident)){

                if let Some(handler) = child_node.handler{
                    handler();
                }
                Some(NodeIdType::Variable(var_node_id))
            }
            else{
                None
            }
        }
        else{
            None
        };

        match (ident_res, var_res) {
            (Some(ident_node_id), Some(var_node_id))=> {
                self.exec_handler_for_route_inner(
                    route, &mut parser.clone(), ident_node_id);
                self.exec_handler_for_route_inner(
                    route, &mut parser.clone(), var_node_id);
                None
            }
            (Some(ident_node_id), None) => {
                Some(ident_node_id)
            }
            (None, Some(var_node_id)) => {
                Some(var_node_id)
            }
            (None, None) => None,
        }
    }

    fn exec_variable_nodes_handler(
        &self,
        parent_node_id: NodeId, 
        segment_str: &str
    ) -> Option<NodeId>{
        let node = &self.var_nodes[parent_node_id.0];
        if let Some((_hash, node_id)) = node.children.ident_node_ids.get(
            &self.str_buffer, &self.ident_nodes, segment_str
        ){
            let child_node = &self.ident_nodes[node_id.0];
            if let Some(handler) = child_node.handler{
                handler();
            }
            Some(node_id)
        }
        else{
            None
        }
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeIdType{
    Ident(NodeId),
    Variable(NodeId),
}

impl NodeIdType{
    fn id(&self) -> NodeId{
        match self{
            NodeIdType::Ident(node_id) => *node_id,
            NodeIdType::Variable(node_id) => *node_id,
        }
    }
}

enum NodeInsertResult{
    SingleIdent(NodeId),
    MultipleIdent(Vec<NodeId>),
    Variable(NodeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentVarType{
    Str,
    Integer,
}

#[derive(Debug, PartialEq, Eq)]
enum TemplateSegment{
    Segment{ident: IdentRange},
    Enum{enum_values: Vec<IdentRange>},
    Var{datatype: SegmentVarType},
}

struct ValueSegment{
    value_type: SegmentValType,
    str_range: IdentRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentValType{
    Ident,
    Integer,
}

#[derive(Debug, PartialEq, Eq)]
struct IdentRange{
    start: usize,
    end: usize
}

impl IdentRange{
    fn new(start: usize, end: usize) -> Self{
        Self { start, end }
    }

    fn substr<'a>(&'a self, source: &'a str) -> &'a str{
        &source[self.start..self.end]
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Segment{
    Str{ident: IdentRange},
    Var{datatype: SegmentVarType},
}

#[derive(Debug, PartialEq, Eq)]
struct StrSegment{
    ident: IdentRange,
}

#[derive(Debug)]
struct VarSegment{
    datatype: SegmentVarType,
}

type NodeHandler = fn() -> ();

#[derive(Debug)]
struct NodeInner<T>{
    segment: T,
    handler: Option<NodeHandler>,
    children: ChildNodes
}

type IdentNode = NodeInner<StrSegment>;

type VarNode = NodeInner<VarSegment>;

#[derive(Debug)]
struct ChildNodes{
    ident_node_ids: SortedNodeIds,
    var_node_id: Option<NodeId>
}

impl ChildNodes{
    fn new() -> Self{
        Self{
            ident_node_ids: SortedNodeIds::new(),
            var_node_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hash(u64);

#[derive(Debug)]
struct SortedNodeIds{
    node_ids: Vec<(Hash, NodeId)>
}

impl SortedNodeIds{
    fn new() -> Self{
        Self { node_ids: Vec::new() }
    }

    fn insert(&mut self, hash: Hash, node_id: NodeId){
        self.node_ids.push((hash, node_id));
        self.node_ids.sort_by_key(|n| n.0.0);
    }

    fn get(&self, str_buffer: &str, nodes: &[IdentNode], segment: &str) -> Option<(Hash, NodeId)>{
        let mut hasher = DefaultHasher::new();
        hasher.write(segment.as_bytes());
        let hash = hasher.finish();
        match self.node_ids.binary_search_by_key(&hash, |hash_node_id| hash_node_id.0.0){
            Ok(index) => {
                for (_hash, node_id) in self.node_ids[..index].iter().rev(){
                    let str_seg = nodes[node_id.0].segment.ident.substr(&str_buffer);
                    if str_seg == segment{
                        return Some((Hash(hash), *node_id));
                    }
                    else{
                        break;
                    }
                }
                for (_hash, node_id) in self.node_ids[index..].iter(){
                    let str_seg = nodes[node_id.0].segment.ident.substr(&str_buffer);
                    if str_seg == segment{
                        return Some((Hash(hash), *node_id));
                    }
                    else{
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
struct Cursor<'a>{
    chars: Chars<'a>,
    len_remaining: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum TokenKind{
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
struct Token{
    kind: TokenKind,
    len: u32,
}

#[derive(Debug, PartialEq, Eq)]
enum TopicRouteParseError{
    UnexpectedToken(Token),
    ExpectedDiffToken{expected: TokenKind, found: Token},
    VarTypeDoesNotExist{invalid: Token},
}

#[derive(Debug, Clone)]
struct Parser<'a>{
    cursor: Cursor<'a>,
}

impl<'a> Parser<'a>{
    fn new(route: &'a str) -> Parser<'a>{
        Parser { 
            cursor: Cursor::new(route), 
        }
    }

    fn bump(&mut self) -> Token{
        self.cursor.advance_token()
    }

    fn first(&mut self) -> Token{
        self.cursor.clone().advance_token()
    }

    fn eat_while(&mut self, predicate: impl Fn(Token) -> bool){
        while predicate(self.first()) && !self.cursor.is_eof() {
            self.bump();
        }
    }

    fn eat_whitespace(&mut self){
        self.eat_while(|t| matches!(t.kind, TokenKind::Whitespace) );
    }

    fn get_token_range(&self, prev_token_len: usize, token: &Token) -> IdentRange{
        let start= prev_token_len;
        let end = token.len as usize;
        IdentRange::new(start, end)
    }

    fn get_token_str(&self, route: &'a str, prev_token_len: usize, token: &Token) -> &'a str{
        let start= prev_token_len;
        let end = token.len as usize;
        let token_str = &route[start..end];
        token_str
    }

    fn advance_route_template_parser(&mut self, route: &str) -> Result<Option<TemplateSegment>, TopicRouteParseError>{
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
            TokenKind::CloseSquareBrace | TokenKind::CloseCurlyBrace |
            TokenKind::Unknown | TokenKind::Pipe => Err(TopicRouteParseError::UnexpectedToken(token)),
            TokenKind::EOF => return Ok(None),
        }
    }

    fn advance_topic_parser(&mut self, route: &str) -> Result<Option<ValueSegment>, TopicRouteParseError>{
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
            TokenKind::OpenSquareBrace | TokenKind::OpenCurlyBrace |
            TokenKind::CloseSquareBrace | TokenKind::CloseCurlyBrace |
            TokenKind::Unknown | TokenKind::Pipe => Err(TopicRouteParseError::UnexpectedToken(token)),
            TokenKind::EOF => return Ok(None),
        }
    }

    fn parse_template_ident(&mut self, prev_token_len: usize, token: &Token) -> Result<Option<TemplateSegment>, TopicRouteParseError>{
        self.eat_whitespace();
        let next_token = self.bump();
        match next_token.kind{
            TokenKind::Slash | TokenKind::EOF =>
            Ok(Some(TemplateSegment::Segment{
                ident: self.get_token_range(prev_token_len, token)
            })),
            _ => Err(TopicRouteParseError::UnexpectedToken(next_token))
        }
    }

    fn parse_topic_ident(&mut self, prev_token_len: usize, token: &Token) -> Result<Option<ValueSegment>, TopicRouteParseError>{
        self.parse_topic_segment(prev_token_len, token, SegmentValType::Ident)
    }

    fn parse_topic_integer(&mut self, prev_token_len: usize, token: &Token) -> Result<Option<ValueSegment>, TopicRouteParseError>{
        self.parse_topic_segment(prev_token_len, token, SegmentValType::Integer)
    }

    fn parse_topic_segment(&mut self, prev_token_len: usize, token: &Token, value_type: SegmentValType) -> Result<Option<ValueSegment>, TopicRouteParseError>{
        self.eat_whitespace();
        let next_token = self.bump();
        match next_token.kind{
            TokenKind::Slash | TokenKind::EOF =>
            Ok(Some(ValueSegment{
                value_type: value_type,
                str_range: self.get_token_range(prev_token_len, token)
            })),
            _ => Err(TopicRouteParseError::UnexpectedToken(next_token))
        }
    }

    fn parse_enum(&mut self) -> Result<Option<TemplateSegment>, TopicRouteParseError>{
        let mut enum_values = Vec::new();
        loop{
            self.eat_whitespace();
            let cur_pos = self.cursor.pos_within_token() as usize;
            let token = self.bump();
            if let TokenKind::Ident = token.kind{
                enum_values.push(
                    self.get_token_range(cur_pos, &token)
                );
                self.eat_whitespace();
                let token = self.bump();
                match token.kind{
                    TokenKind::Pipe => continue,
                    TokenKind::CloseSquareBrace => return Ok(Some(TemplateSegment::Enum{enum_values})),
                    _ =>  return Err(TopicRouteParseError::UnexpectedToken(token)),
                }
            }
            else{
                return Err(TopicRouteParseError::UnexpectedToken(token));
            }
        }
    }

    fn parse_var(&mut self, route: &str) -> Result<Option<TemplateSegment>, TopicRouteParseError>{
        self.eat_whitespace();
        let cur_pos = self.cursor.pos_within_token() as usize;
        let token = self.bump();
        if let TokenKind::Ident = token.kind{
            let datatype = match self.get_token_str(route, cur_pos, &token){
                "str" => SegmentVarType::Str,
                "integer" => SegmentVarType::Integer,
                _ => return  Err(TopicRouteParseError::VarTypeDoesNotExist {
                    invalid: token 
                }),
            };
            self.eat_whitespace();
            let token = self.bump();
            if let TokenKind::CloseCurlyBrace = token.kind{
                Ok(Some(TemplateSegment::Var { datatype }))
            }
            else{
                Err(TopicRouteParseError::ExpectedDiffToken {
                    expected: TokenKind::CloseCurlyBrace, found: token
                })
            }
        }
        else{
            Err(TopicRouteParseError::ExpectedDiffToken {
                expected: TokenKind::Ident, found: token
            })
        }
    }
}

impl<'a> Cursor<'a>{
    fn new(src: &'a str) -> Cursor<'a>{
        Self { 
            len_remaining: src.len(),
            chars: src.chars(),
        }
    }

    fn is_eof(&self) -> bool{
        self.chars.as_str().is_empty()
    }

    fn as_str(&self) -> &'a str{
        self.chars.as_str()
    }

    fn first(&self) -> char{
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    fn second(&self) -> char{
        let mut chars = self.chars.clone();
        chars.next();
        chars.next()
            .unwrap_or(EOF_CHAR)
    }

    fn bump(&mut self) -> Option<char>{
        self.chars.next()
    }

    fn eat_while(&mut self, predicate: impl Fn(char) -> bool){
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }

    fn eat_until(&mut self, byte: u8){
        let mut bytes = self.as_str().bytes().enumerate();
        self.chars = loop {
            match bytes.next() {
                Some((i, cur_byte)) => if cur_byte == byte{
                    break self.as_str()[i..].chars();
                },
                None => break "".chars(),
            }
        };
    }

    fn white_space(&mut self) -> TokenKind{
        self.eat_while(char::is_whitespace);
        TokenKind::Whitespace
    }

    fn is_ident_start(c: char) -> bool{
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }

    fn is_ident_continue(c: char) -> bool{
        matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
    }

    fn ident(&mut self) -> TokenKind{
        self.eat_while(Self::is_ident_continue);
        TokenKind::Ident
    }

    fn number(&mut self) -> TokenKind{
        self.eat_while(Self::is_number_continue);
        TokenKind::Integer
    }

    fn is_number_start(c: char) -> bool{
        matches!(c, '0'..='9')
    }

    fn is_number_continue(c: char) -> bool{
        matches!(c, '0'..='9' | '_')
    }

    fn pos_within_token(&self) -> u32{
        (self.len_remaining - self.chars.as_str().len()) as u32
    }

    fn len_left(&self) -> u32{
        self.chars.as_str().len() as u32
    }

    fn advance_token(&mut self) -> Token{
        let Some(first_char) = self.chars.next() else {
            return Token{kind: TokenKind::EOF, len: 0};
        };

        let kind: TokenKind = match first_char{
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
        Token { kind, len: self.pos_within_token() }
    }
}

#[cfg(test)]
mod test{
    use crate::topic_router::{IdentRange, TemplateSegment, SegmentVarType, TopicRouter};

    fn assert_substr(
        route: &str, substr: &str, start: usize, end: usize
    ) -> IdentRange{
        let ident = IdentRange::new(start, end);
        assert_eq!(substr, ident.substr(route));
        ident
    }

    #[test]
    fn url_router_test(){
        let topic1 =  "{integer}/routepart1/{str}/routepart2/[enumval1|enumval2|enumval3]/routepart3";
        let segments = TopicRouter::parse_route(topic1);
        dbg!(&segments);
        assert_eq!(Ok(vec![
            TemplateSegment::Var { datatype: SegmentVarType::Integer },
            TemplateSegment::Segment { ident:
                assert_substr(topic1, "routepart1", 10, 20) },
            TemplateSegment::Var { datatype: SegmentVarType::Str },
            TemplateSegment::Segment { ident:
                assert_substr(topic1, "routepart2", 27, 37) },
            TemplateSegment::Enum {enum_values: vec![
                assert_substr(topic1, "enumval1", 39, 47),
                assert_substr(topic1, "enumval2", 48, 56), 
                assert_substr(topic1, "enumval3", 57, 65)
            ]},
            TemplateSegment::Segment { ident: 
                assert_substr(topic1, "routepart3", 67, 77)},
        ]), segments);
        let topic2 =  "routepart1/routepart2/[enumval1|enumval2]";
        let segments = TopicRouter::parse_route(topic2);
        assert_eq!(Ok(vec![
            TemplateSegment::Segment { ident:
                assert_substr(topic2, "routepart1", 0, 10)
            },
            TemplateSegment::Segment { ident:
                assert_substr(topic2, "routepart2", 11, 21)
            },
            TemplateSegment::Enum {enum_values: vec![
                assert_substr(topic2, "enumval1", 23, 31),
                assert_substr(topic2, "enumval2", 32, 40)
            ]},
        ]), segments);
        dbg!(segments);

        let topic3 =  "routepart1/routepart2/routepart3/routpart4";

        let mut router = TopicRouter::new();
        dbg!(router.add_route(topic2, Some(|| println!("test topic 2"))));
        dbg!(&router);
        dbg!(router.add_route(topic1, Some(|| println!("test topic 1"))));
        dbg!(&router);
        dbg!(router.add_route(topic3, Some(|| println!("test topic 3"))));
        dbg!(&router);

        let topic1_1 =  "42/routepart1/str_var1/routepart2/enumval1/routepart3";
        let topic1_2 =  "9102988/routepart1/str23j_var2/routepart2/enumval2/routepart3";
        let topic1_3 =  "3242/routepart1/str_334dfvar3/routepart2/enumval3/routepart3";

        router.debug_print();
        router.exec_handler_for_route(topic1_1);
        router.exec_handler_for_route(topic1_2);
        router.exec_handler_for_route(topic1_3);

        let topic2_1 =  "routepart1/routepart2/enumval1";
        let topic2_2 =  "routepart1/routepart2/enumval2";

        router.exec_handler_for_route(topic2_1);
        router.exec_handler_for_route(topic2_2);

        router.exec_handler_for_route(topic3);


        assert!(false);
    }
}


