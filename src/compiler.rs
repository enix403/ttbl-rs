#![allow(unreachable_patterns)]

use crate::scanner::{Token, TokenType, OperatorType};
use std::collections::HashMap;

type VarLocation = usize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeOperation
{
    BinaryOperation(OperatorType),
    UnaryOperation(OperatorType),
    VariableDeref(VarLocation),
    Literal(bool),

    Subexpression,
    IndexedSubexpression(u32)
}

#[derive(Debug)]
pub struct ASTNode
{
    pub op: NodeOperation,

    pub left: Option<Box<ASTNode>>,
    pub right: Option<Box<ASTNode>>,
}

impl ASTNode
{
    fn create(op: NodeOperation) -> ASTNode
    {
        ASTNode { op, left: None, right: None }
    }
}

fn stash_prev_op(current_op: OperatorType, target_op: OperatorType) -> bool
{
    let op_priority: HashMap<OperatorType, u16> = HashMap::from([
        (OperatorType::NOT, 6),
        (OperatorType::AND, 4),
        (OperatorType::OR, 2),
    ]);

    return op_priority.get(&current_op).unwrap() < op_priority.get(&target_op).unwrap();
}

type NodeStack = Vec<Box<ASTNode>>;

fn create_operation_node(node_stack: &mut NodeStack, op: OperatorType) -> Result<ASTNode, ()>
{
    let mut node: ASTNode;

    match op
    {
        OperatorType::AND | OperatorType::OR => {
            node = ASTNode::create(NodeOperation::BinaryOperation(op));
            node.right = node_stack.pop();
            node.left = node_stack.pop();

            if let None = node.left     { return Err(()); };
            if let None = node.right    { return Err(()); };
        },
        OperatorType::NOT => {
            node = ASTNode::create(NodeOperation::UnaryOperation(op));
            node.left = node_stack.pop();
            if let None = node.left     { return Err(()); };
        }
    };

    return Ok(node);
}

fn peek_stack<'a, T>(stack: &'a Vec<T>) -> &'a T
{
    return &stack[stack.len() - 1];
}

pub struct CompiledSyntaxBTree<'a>
{
    pub error_token: Option<Token<'a>>,
    pub root: Option<Box<ASTNode>>,
    pub variables: Vec<String>
}

pub fn compile<'a>(tokens: &'a Vec<Token>) -> CompiledSyntaxBTree<'a>
{
    let mut node_stack: NodeStack = vec![];
    let mut operands_stack: Vec<TokenType> = vec![];

    let mut variables: Vec<String> = vec![];

    let mut error = false;
    let mut error_token_ref: &Token = &tokens[0];

    for token in tokens
    {
        if error {
            break;
        }

        match token.token_type
        {
            TokenType::Error => { error = true; error_token_ref = &token; break; },

            TokenType::LeftParen => { operands_stack.push(TokenType::LeftParen); },
            TokenType::LeftBrace => { operands_stack.push(TokenType::LeftBrace); },
            TokenType::Variable => {
                let location: usize;
                if let Some(pos) = variables.iter().position(|var| var == token.lexeme)
                {
                    location = pos;
                }
                else
                {
                    location = variables.len();
                    variables.push(String::from(token.lexeme));
                }

                let node = ASTNode::create(NodeOperation::VariableDeref(location));
                node_stack.push(Box::new(node));
            },
            TokenType::Literal(val) => {
                let node = ASTNode::create(NodeOperation::Literal(val));
                node_stack.push(Box::new(node));
            },
            TokenType::Operator(current_op) => {
                while !operands_stack.is_empty()
                {
                    if let TokenType::Operator(target_op) = peek_stack(&operands_stack)
                    {
                        if !stash_prev_op(current_op, *target_op)
                        {
                            break;
                        }

                        let node = create_operation_node(&mut node_stack, *target_op);
                        match node
                        {
                            Ok(n) => { node_stack.push(Box::new(n)); }
                            _ => { error = true; error_token_ref = &token; break; }
                        }
                        
                        operands_stack.pop();
                    }
                    else {
                        break;
                    }
                }

                operands_stack.push(TokenType::Operator(current_op));
            },
            TokenType::RightParen | TokenType::RightBrace | TokenType::EOF => {

                while !operands_stack.is_empty()
                {
                    let top_op = peek_stack(&operands_stack);

                    let paren_closing = token.token_type == TokenType::RightParen && *top_op == TokenType::LeftParen;
                    let brace_closing = token.token_type == TokenType::RightBrace && *top_op == TokenType::LeftBrace;

                    if let TokenType::Operator(op) = top_op
                    {
                        let node = create_operation_node(&mut node_stack, *op);
                        match node
                        {
                            Ok(n) => { node_stack.push(Box::new(n)); }
                            _ => { error = true; error_token_ref = &token; break; }
                        }
                        
                        operands_stack.pop();
                    }
                    else if paren_closing {
                        operands_stack.pop();
                        break;
                    }
                    else if brace_closing
                    {
                        operands_stack.pop();

                        // Prevent redundant nested groups
                        if node_stack.len() > 0 && node_stack[node_stack.len() - 1].op != NodeOperation::Subexpression 
                        {
                            let mut node = ASTNode::create(NodeOperation::Subexpression);
                            node.left = node_stack.pop();
                            node_stack.push(Box::new(node));
                        }
                        break;
                    }
                    else {
                        error = true;
                        error_token_ref = &token;
                        break;
                    }
                }
            },
            _ => ()
        }
    };

    let mut result = CompiledSyntaxBTree {
        error_token: if error { Some(error_token_ref.clone()) } else { None },
        root: None,
        variables
    };

    if !error
    {
        if operands_stack.len() == 0 && node_stack.len() == 1
        {
            result.root = node_stack.pop();
        }
        else
        {
            result.error_token = Some(Token { 
                lexeme: "<EOF>", token_type: TokenType::EOF 
            });
        }
    }

    return result;
}


