use crate::compiler::{NodeOperation, ASTNode};
use crate::scanner::OperatorType;

pub fn postorder_traversal_postfix(node: &Box<ASTNode>, output: &mut Vec<NodeOperation>, start_index: usize) -> usize
{   
    let mut w_left: usize   = 0;
    let mut w_right: usize  = 0;

    if let Some(child) = &node.left
    {
        w_left = postorder_traversal_postfix(child, output, start_index);

        if let Some(child) = &node.right
        {
            w_right = postorder_traversal_postfix(child, output, start_index + w_left);
        }
    }

    output.push(node.op);

    return w_left + w_right + 1;
}

/**
 * Returns number of operands required for a node 
**/
fn op_backtrack_size(op: &NodeOperation) -> u8
{
    return match op 
    {
        NodeOperation::Literal(_) 
        | NodeOperation::VariableDeref(_) 
        | NodeOperation::Subexpression
        | NodeOperation::IndexedSubexpression(_) => 0,

        NodeOperation::UnaryOperation(_) => 1,
        NodeOperation::BinaryOperation(_) => 2
    };
}

/** 
 * Calculates the length of smallest 'standalone' sub-list of nodes (subexpression that does not depend on
   neighbouring nodes) in the given postfix expression ending at the given subexpression marker
 * 
 * Example: Given the following postfix expression (with the subexpression marker at 2nd position from last)
 *      expr = [ var(p), var(q), op(or), var(p), var(q), op(and), op(not), SUB, op(and) ]
 *      (Postfix expression of `(p or q) and !(p and q)`)
 * 
 *  It returns the length of the sub-list from index=3 to index=6
 *                              |-----------------------------------|  
 *    [ var(p), var(q), op(or), | var(p), var(q), op(and), op(not), | SUB, op(and) ]
 *                              |-----------------------------------|  
 **/
fn subexpression_backtrack_size(pf_list: &Vec<NodeOperation>, sub_loc: usize) -> usize
{
    let mut remaining = 1;
    let mut consumed = 0;

    while remaining != 0
    {
        consumed += 1;
        remaining = remaining - 1 + op_backtrack_size(&pf_list[sub_loc - consumed]);
    }

    return consumed;
}

fn subexpression_groups_impl(pf_list: Vec<NodeOperation>, locations: Vec<usize>) -> Vec<Vec<NodeOperation>>
{
    let mut pf_list = pf_list;
    let mut groups = Vec::<Vec<NodeOperation>>::with_capacity(locations.len());

    let mut total_removed = 0;

    for (index, loc) in locations.into_iter().enumerate()
    {
        let loc = loc - total_removed;
        let mut grp = Vec::<NodeOperation>::with_capacity(loc);
        let backtrack_size = subexpression_backtrack_size(&pf_list, loc);
        for i in 0..backtrack_size
        {
            grp.push(pf_list[loc - backtrack_size + i]);
        }
        groups.push(grp);
        total_removed += pf_list.drain((loc - backtrack_size)..loc).count();
    }

    return groups;
}

pub fn subexpression_groups(node: &Box<ASTNode>) -> Vec<Vec<NodeOperation>>
{
    // 1 . First convert the expression tree to its postfix representation

    let mut as_list = Vec::<NodeOperation>::new();
    let count = postorder_traversal_postfix(node, &mut as_list, 0);

    let mut locations: Vec<usize> = vec![];
    for i in 0..count
    {
        if as_list[i] == NodeOperation::Subexpression
        {
            as_list[i] = NodeOperation::IndexedSubexpression(locations.len() as u32);
            locations.push(i);
        }
    }

    // 1.5 . Instead of making the final result an edge-case calculation, we can wrap the whole
    // expression in a subexpr group. This way it will get picked up and outputted just like
    // other groups.
    match locations.last()
    {
        // Do not add a redundant IndexedSubexpression on top of an existing one at the end
        Some(loc) if *loc == count - 1 => (),
        _ => {
            as_list.push(NodeOperation::IndexedSubexpression(locations.len() as u32));
            locations.push(count);
        }
    }

    // 2 . Then divide the list into subexpresion groups

    return subexpression_groups_impl(as_list, locations);
}

pub fn evaluate(groups: &Vec<Vec<NodeOperation>>, values: &[bool], out_eval: &mut [bool])
{
    let mut operands_stack = Vec::<bool>::with_capacity(100);
    let mut index = 0;
    for grp in groups
    {
        operands_stack.clear();

        for op in grp
        {
            match op
            {
                NodeOperation::Literal(val) => { operands_stack.push(*val); },
                NodeOperation::VariableDeref(loc) => { operands_stack.push(values[*loc]); },
                NodeOperation::IndexedSubexpression(sub_loc) => {
                    operands_stack.push(out_eval[*sub_loc as usize]);
                },
                NodeOperation::BinaryOperation(op_type) => {
                    let right = operands_stack.pop().expect("Operand not found");
                    let left = operands_stack.pop().expect("Operand not found");

                    let result = match *op_type
                    {
                        OperatorType::AND => left && right,
                        OperatorType::OR => left || right,
                        _ => { panic!("Unhandled binary operation"); }
                    };

                    operands_stack.push(result)
                },
                NodeOperation::UnaryOperation(op_type) => {
                    let left = operands_stack.pop().expect("Operand not found");

                    let result = match *op_type
                    {
                        OperatorType::NOT => !left,
                        _ => { panic!("Unhandled unary operation"); }
                    };

                    operands_stack.push(result)
                },
                _ => ()
            }
        }

        let result = operands_stack.pop().expect("Broken expression");
        out_eval[index] = result;
        index += 1;
    }
}

const SYMBOL_TRUE: &'static str = "<T>";
const SYMBOL_FALSE: &'static str = "<F>";
const SYMBOL_AND: &'static str = " & ";
const SYMBOL_OR: &'static str = " | ";
const SYMBOL_NOT: &'static str = "!";
const SYMBOL_LEFT_PAREN: &'static str = "(";
const SYMBOL_RIGHT_PAREN: &'static str = ")";

// TODO: Optimize. Horribly slow at the moment
pub fn groups_to_string(groups: &Vec<Vec<NodeOperation>>, variables: &Vec<String>) -> Vec<String>
{
    let mut operands_stack = Vec::<String>::with_capacity(100);
    let mut result = Vec::<String>::with_capacity(groups.len());

    for grp in groups
    {
        operands_stack.clear();

        for op in grp
        {
            match op
            {
                NodeOperation::Literal(val) => { 
                    operands_stack.push(String::from( if *val { SYMBOL_TRUE } else { SYMBOL_FALSE } ));
                },
                NodeOperation::VariableDeref(loc) => { 
                    operands_stack.push(variables[*loc].clone());
                },
                NodeOperation::IndexedSubexpression(sub_loc) => {
                    operands_stack.push(result[*sub_loc as usize].clone());
                },
                NodeOperation::BinaryOperation(op_type) => {
                    let right = operands_stack.pop().unwrap();
                    let left = operands_stack.pop().unwrap();

                    let symbol = match *op_type {
                        OperatorType::AND => SYMBOL_AND,
                        OperatorType::OR => SYMBOL_OR,
                        _ => ""
                    };

                    operands_stack.push([
                        String::from(SYMBOL_LEFT_PAREN),
                        left, 
                        String::from(symbol),
                        right,
                        String::from(SYMBOL_RIGHT_PAREN),
                    ].join(""));
                },
                NodeOperation::UnaryOperation(op_type) => {
                    let left = operands_stack.pop().unwrap();

                    let symbol = match *op_type {
                        OperatorType::NOT => SYMBOL_NOT,
                        _ => ""
                    };

                    operands_stack.push([
                        String::from(symbol),
                        String::from(SYMBOL_LEFT_PAREN),
                        left,
                        String::from(SYMBOL_RIGHT_PAREN),
                    ].join(""));
                },
                _ => ()   
            }
        }

        result.push(operands_stack.pop().expect("Broken expression"));
    }

    return result;
}
