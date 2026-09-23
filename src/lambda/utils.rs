
// Global Imports
use lambda_calculus::Term;

// Package Imports
use crate::lambda::result::LambdaCollisionError;

pub fn has_two_args(expr: &Term) -> bool {
    if let Term::Abs(ref body) = expr {
        if let Term::Abs(_) = **body {
            return true;
        }
    }
    false
}

// Check if expr has the form \x1. ... \xn. var for n >= 2
pub fn is_truthy(expr: &Term) -> bool {
    if let Term::Abs(ref body) = expr {
        // Hopefully if let chaining becomes stable someday
        if let Term::Abs(ref var) = **body {
            if let Term::Var(_) = **var {
                return true;
            }
        }
        return is_truthy(body);
    }
    false
}

fn uses_both_arguments_helper(expr: &Term, depth: usize) -> (bool, bool) {
    match expr {
        Term::Abs(ref boxed) => uses_both_arguments_helper(boxed, depth + 1),
        Term::App(ref boxed) => {
            let (ref left, ref right) = **boxed;
            let (l0, l1) = uses_both_arguments_helper(left, depth);
            let (r0, r1) = uses_both_arguments_helper(right, depth);
            (l0 || r0, l1 || r1)
        }
        Term::Var(n) => (*n == depth, *n == depth - 1),
    }
}

pub fn uses_both_arguments(expr: &Term) -> bool {
    let (left, right) = uses_both_arguments_helper(expr, 0);
    left && right
}

pub fn reduce_with_limit(
    expr: &mut Term,
    rlimit: usize,
    slimit: usize,
) -> Result<usize, LambdaCollisionError> {
    let mut n = 0;
    for _ in 0..rlimit {
        if expr.reduce(lambda_calculus::HAP, 1) == 0 {
            break;
        }

        // WARNING: This is EXTREMELY expensive. Calling max_depth is log(depth), and is done
        // per reduction step. Remove when possible.
        let depth = expr.size();
        if depth > slimit {
            return Err(LambdaCollisionError::ExceedsDepthLimit);
        }
        n += 1;
    }
    Ok(n)
}