use std::fmt;
use std::io::{self, Write};
use std::rc::Rc;
use std::time::Instant;

#[derive(PartialEq)]
enum NodeValue {
    Operator(char),
    Index(usize),
}

struct Node {
    value: NodeValue,
    left: Option<Rc<Node>>,
    right: Option<Rc<Node>>,
    id: u32,
}

impl Node {
    fn new(value: NodeValue, id: u32) -> Self {
        Node {
            value,
            left: None,
            right: None,
            id,
        }
    }
    fn left(mut self, node: &Rc<Node>) -> Self {
        self.left = Some(node.clone());
        self
    }
    fn right(mut self, node: &Rc<Node>) -> Self {
        self.right = Some(node.clone());
        self
    }
    fn run(&self, env: &Vec<i32>) -> f64 {
        match self.value {
            NodeValue::Index(index) => env[index] as f64,
            NodeValue::Operator(op) => {
                let left = self.left.as_ref().unwrap().run(env);
                let right = self.right.as_ref().unwrap().run(env);
                match op {
                    '+' => left + right,
                    '-' => left - right,
                    '*' => left * right,
                    '/' => {
                        if left.is_finite() && right.is_normal() {
                            left / right
                        } else {
                            std::f64::NAN
                        }
                    }
                    _ => panic!("Undefined Operator: {}", op),
                }
            }
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            NodeValue::Index(index) => write!(f, "[{}]", index.to_string()),
            NodeValue::Operator(op) => {
                let left = self.left.as_ref().unwrap();
                let right = self.right.as_ref().unwrap();
                let left_cap = "*/".find(op).is_some()
                    && (left.value == NodeValue::Operator('+')
                        || left.value == NodeValue::Operator('-'));
                let right_cap = '/' == op
                    && match right.value {
                        NodeValue::Index(_) => false,
                        NodeValue::Operator(_) => true,
                    }
                    || ("*-".find(op).is_some()
                        && match self.right.as_ref().unwrap().value {
                            NodeValue::Index(_) => false,
                            NodeValue::Operator(op) => "+-".find(op).is_some(),
                        });
                if left_cap {
                    f.write_str("(").and(left.fmt(f).and(f.write_str(")")))
                } else {
                    left.fmt(f)
                }
                .and(op.fmt(f).and(if right_cap {
                    f.write_str("(").and(right.fmt(f).and(f.write_str(")")))
                } else {
                    right.fmt(f)
                }))
            }
        }
    }
}

fn gen_operators(left: &Rc<Node>, right: &Rc<Node>, id: u32) -> Vec<Rc<Node>> {
    let mut results = Vec::new();
    for op in "+*".chars() {
        if left.value != NodeValue::Operator(op)
            && (right.value != NodeValue::Operator(op) || left.id < right.left.as_ref().unwrap().id)
        {
            results.push(Rc::new(
                Node::new(NodeValue::Operator(op), id).left(left).right(right),
            ));
        }
    }
    for op in "-/".chars() {
        results.push(Rc::new(
            Node::new(NodeValue::Operator(op), id).left(left).right(right),
        ));
        results.push(Rc::new(
            Node::new(NodeValue::Operator(op), id).left(right).right(left),
        ));
    }
    results
}

fn dfs(trees: Vec<Rc<Node>>, minj: usize) -> Vec<Rc<Node>> {
    if trees.len() == 1 {
        return vec![trees[0].clone()];
    }
    let mut results = Vec::new();
    for j in minj..trees.len() {
        for i in 0..j {
            let new_trees: Vec<Rc<Node>> = (0..trees.len())
                .filter(|&k| k != i && k != j)
                .map(|k| trees[k].clone())
                .collect();
            for node in gen_operators(&trees[i], &trees[j], trees.last().unwrap().id + 1) {
                let mut new_trees = new_trees.clone();
                new_trees.push(node);
                results.append(&mut dfs(new_trees, j - 1));
            }
        }
    }
    results
}

fn check(expressions: &Vec<Rc<Node>>, env: &Vec<i32>, target: i32) -> Vec<Rc<Node>> {
    let mut results = Vec::new();
    for exp in expressions {
        if (target as f64 - exp.as_ref().run(env)).abs() < 0.01 {
            results.push(exp.clone());
        }
    }
    results
}

fn main() -> Result<(), io::Error> {
    println!("Generating Expressions...");
    let start = Instant::now();
    let expressions = dfs((0..4)
        .map(|i| Rc::new(Node::new(NodeValue::Index(i), i as u32)))
        .collect(), 1);
    println!("expressions len: {}", expressions.len());
    println!("{:?}", start.elapsed());
    let mut env = vec![0; 4];
    loop {
        print!("Input 4 Numbers: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let mut input = input.split_whitespace();
        || -> Option<()> {
            env[0] = input.next()?.parse().ok()?;
            env[1] = input.next()?.parse().ok()?;
            env[2] = input.next()?.parse().ok()?;
            env[3] = input.next()?.parse().ok()?;

            let solutions = check(&expressions, &env, 24);
            if solutions.is_empty() {
                println!("No Solution.");
            } else {
                for exp in solutions {
                    println!(
                        "{}",
                        format!("{}", exp)
                            .replace("[0]", &env[0].to_string())
                            .replace("[1]", &env[1].to_string())
                            .replace("[2]", &env[2].to_string())
                            .replace("[3]", &env[3].to_string())
                    );
                }
            }
            None
        }();
    }
}
