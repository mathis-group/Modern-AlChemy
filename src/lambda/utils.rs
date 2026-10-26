
// Global Imports
use lambda_calculus::Term;

// Package Imports
use crate::lambda::result::LambdaCollisionError;

pub fn has_two_args(expr: &Term) -> bool {
    if let Term::Abs(body) = expr {
        if let Term::Abs(_) = **body {
            return true;
        }
    }
    false
}

// Check if expr has the form \x1. ... \xn. var for n >= 2
pub fn is_truthy(expr: &Term) -> bool {
    if let Term::Abs(body) = expr {
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
        Term::Abs(boxed) => uses_both_arguments_helper(boxed, depth + 1),
        Term::App(boxed) => {
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
/// Reduce `expr` to normal form under the reactor's cutoffs. Fails if a step
/// grows the expression past `slimit`, or if all `rlimit` steps are used.
/// Shared by the collider (for products) and the generator (for seeds), so the
/// two always apply the same rule
pub fn reduce_to_normal_form(
    expr: &mut Term,
    rlimit: usize,
    slimit: usize,
) -> Result<usize, LambdaCollisionError> {
    let n = reduce_with_limit(expr, rlimit, slimit)?;
    if n == rlimit {
        return Err(LambdaCollisionError::ExceedsReductionLimit);
    }
    Ok(n)
}

/// Close `expr` by binding each distinct free variable with a new outer
/// abstraction, one per variable. The free variable met first (function before
/// argument) gets the innermost new abstraction. A closed `expr` is unchanged
pub fn close_free_variables(expr: Term) -> Term {
    // `depth` is the number of abstractions above the current node, so a
    // De Bruijn index above it is free. `free` lists free indices (relative to
    // the root) in order of first use
    fn bind(expr: Term, depth: usize, free: &mut Vec<usize>) -> Term {
        match expr {
            // A free variable: its index points past the top of the term
            Term::Var(i) if i > depth => {
                // Which free variable this is, independent of how deep it sits
                let root_index = i - depth;

                // Its position in `free`, adding it to the end if this is its first use
                let free_index = match free.iter().position(|&f| f == root_index) {
                    Some(free_index) => free_index,
                    None => {
                        free.push(root_index);
                        free.len() - 1
                    }
                };

                // Point past the `depth` abstractions above it to its new outer
                // abstraction. The first free variable gets the innermost one;
                // `+ 1` because De Bruijn indices start at 1
                Term::Var(depth + free_index + 1)
            }
            // A bound variable is left as it is
            Term::Var(i) => Term::Var(i),
            Term::Abs(body) => Term::Abs(Box::new(bind(*body, depth + 1, free))),
            Term::App(pair) => {
                let (left, right) = *pair;
                Term::App(Box::new((bind(left, depth, free), bind(right, depth, free))))
            }
        }
    }

    let mut free = Vec::new();
    let mut closed = bind(expr, 0, &mut free);
    // Add one outer abstraction per distinct free variable found
    for _ in 0..free.len() {
        closed = Term::Abs(Box::new(closed));
    }
    closed
}