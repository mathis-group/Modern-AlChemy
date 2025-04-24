use async_std::task::{block_on, spawn};
use futures::{stream::FuturesUnordered, StreamExt};
use lambda_calculus::reduction::Order::HAP;
use lambda_calculus::{
    abs, app,
    combinators::{I, K, S},
    data::{
        boolean::{self, and},
        num::church::{add, eq, succ},
    },
    parse,
    term::Notation::Classic,
    IntoChurchNum,
    Term::{self, Var},
};
use rand::random;

use crate::{
    config::{self, ConfigSeed},
    generators::BTreeGen,
    lambda::recursive::{has_two_args, is_truthy, uses_both_arguments, LambdaSoup},
    utils::{dump_series_to_file, read_inputs},
};

fn experiment_soup(seed: ConfigSeed) -> LambdaSoup {
    LambdaSoup::from_config(&config::Reactor {
        rules: vec![String::from("\\x.\\y.x y")],
        discard_copy_actions: false,
        discard_identity: false,
        discard_free_variable_expressions: true,
        maintain_constant_population_size: true,
        discard_parents: false,
        reduction_cutoff: 8000,
        size_cutoff: 1000,
        seed,
    })
}

pub fn coadd() -> Term {
    abs!(2, app!(Var(2), succ(), Var(1)))
}

pub fn addtwo() -> Term {
    let mut comp = app!(succ(), succ());
    comp.reduce(HAP, 0);
    comp
}

// Triplet permutation combinators
fn p123() -> Term {
    abs!(3, app!(Var(1), Var(2), Var(3)))
}

fn p132() -> Term {
    abs!(3, app!(Var(1), Var(3), Var(2)))
}

fn p213() -> Term {
    abs!(3, app!(Var(2), Var(1), Var(3)))
}

fn p231() -> Term {
    abs!(3, app!(Var(2), Var(3), Var(1)))
}

fn p312() -> Term {
    abs!(3, app!(Var(3), Var(1), Var(2)))
}

fn p321() -> Term {
    abs!(3, app!(Var(3), Var(2), Var(1)))
}

pub(super) fn test_add(a: usize, b: usize) -> Term {
    let mut test = parse(r"\eq. \a. \b. \ab. \f. (eq (f a b) ab)", Classic).unwrap();
    test = app!(
        test,
        eq(),
        a.into_church(),
        b.into_church(),
        (a + b).into_church()
    );
    // `test` has type (church -> church -> church) -> bool
    test.reduce(lambda_calculus::HAP, 0);
    test
}

fn test_add_seq(pairs: impl Iterator<Item = (usize, usize)>) -> Term {
    let mut test = parse(r"\f. \a. \b. a", Classic).unwrap();
    for (u, v) in pairs {
        let gut = parse(
            r"\and. \test. \testadd. \f. and (test f) (testadd f)",
            Classic,
        )
        .unwrap();
        test = app!(gut, and(), test, test_add(u, v));
    }
    test.reduce(lambda_calculus::HAP, 0);
    let mut comp = app!(test.clone(), add());
    comp.reduce(lambda_calculus::HAP, 0);
    assert!(comp.is_isomorphic_to(&boolean::tru()));
    test
}

pub(super) fn test_succ(a: usize) -> Term {
    let mut test = parse(r"\eq. \a. \asucc. \f. (eq (f a) asucc)", Classic).unwrap();
    test = app!(test, eq(), a.into_church(), (a + 1).into_church());
    // `test` has type (church -> church) -> bool
    test.reduce(lambda_calculus::HAP, 0);
    test
}

pub(super) fn test_succ_seq(nums: impl Iterator<Item = usize>) -> Term {
    let mut test = parse(r"\f. \a. \b. a", Classic).unwrap();
    for u in nums {
        let gut = parse(
            r"\and. \test. \testscc. \f. and (test f) (testscc f)",
            Classic,
        )
        .unwrap();
        test = app!(gut, and(), test, test_succ(u));
    }
    test.reduce(lambda_calculus::HAP, 0);
    let mut comp = app!(test.clone(), succ());
    comp.reduce(lambda_calculus::HAP, 0);
    assert!(comp.is_isomorphic_to(&boolean::tru()));
    test
}

pub fn test_addtwo(a: usize) -> Term {
    let mut test = parse(r"\eq. \a. \asucc. \f. (eq (f a) asucc)", Classic).unwrap();
    test = app!(test, eq(), a.into_church(), (a + 2).into_church());
    // `test` has type (church -> church) -> bool
    test.reduce(lambda_calculus::HAP, 0);
    test
}

fn generate_sample_for_addsearch(seed: ConfigSeed) -> Vec<Term> {
    let mut sample = vec![S(); 200];
    sample.append(&mut vec![K(); 100]);
    sample.append(&mut vec![I(); 100]);
    for size in 5..12 {
        let mut gen = BTreeGen::from_config(&config::BTreeGen {
            size,
            freevar_generation_probability: 0.2,
            standardization: crate::generators::Standardization::Prefix,
            n_max_free_vars: 6,
            seed,
        });
        let n_samples = match size {
            5..=7 => 800,
            8..=10 => 400,
            _ => 200,
        };
        sample.append(&mut gen.generate_n(n_samples))
    }
    sample
}

pub(super) fn asymmetric_skip_sample() -> Vec<Term> {
    let mut sample = vec![];
    sample.append(&mut vec![S(); 10]);
    sample.append(&mut vec![K(); 10]);
    sample.append(&mut vec![I(); 10]);
    sample.append(&mut vec![p132(); 1]);
    sample
}

pub(super) fn symmetric_skip_sample() -> Vec<Term> {
    vec![S(), K(), I(), p132()]
}

pub(super) fn ski_sample() -> Vec<Term> {
    vec![S(), K(), I()]
}

fn dump_sample(sample: &Vec<Term>) {
    for expr in sample {
        if expr.is_isomorphic_to(&succ()) {
            println!("successor: {expr}");
        }
        println!(
            "{expr}, {:?}, {} {} {}",
            expr,
            !is_truthy(expr),
            uses_both_arguments(expr),
            has_two_args(expr)
        );
    }
}

async fn add_magic_tests(
    sample: impl Iterator<Item = Term>,
    tests: impl Iterator<Item = Term>,
    id: usize,
    run_length: usize,
    polling_interval: usize,
) -> (usize, Vec<(usize, usize, usize)>) {
    let mut soup = experiment_soup(ConfigSeed::new([id as u8; 32]));
    soup.add_lambda_expressions(sample);
    soup.add_test_expressions(tests);
    let mut populations = Vec::new();
    for i in 0..10 {
        let pops = soup.simulate_and_poll(run_length / 10, polling_interval, false, |s| {
            (
                s.expressions().filter(|e| e.is_recursive()).count(),
                s.population_of(&succ()),
                s.population_of(&add()),
            )
        });
        populations.extend(pops);
        let n_remaining = 1000 - soup.expressions().filter(|e| e.is_recursive()).count();
        let tests = [
            || test_succ(random::<usize>() % 20),
            || test_add(random::<usize>() % 20, random::<usize>() % 20),
        ]
        .into_iter()
        .map(|f| f())
        .cycle()
        .take(n_remaining);
        soup.perturb_test_expressions(n_remaining, tests);
        let skips = asymmetric_skip_sample();
        soup.perturb_lambda_expressions(200, skips);

        println!("Soup {id} {}0% done", i + 1);
    }
    (id, populations)
}

async fn succ_magic_tests(
    sample: impl Iterator<Item = Term>,
    tests: impl Iterator<Item = Term>,
    id: usize,
    run_length: usize,
    polling_interval: usize,
) -> (usize, Vec<(usize, usize, usize)>) {
    let mut soup = experiment_soup(ConfigSeed::new([id as u8; 32]));
    soup.add_lambda_expressions(sample);
    soup.add_test_expressions(tests);
    let mut populations = Vec::new();
    for i in 0..10 {
        let pops = soup.simulate_and_poll(run_length / 10, polling_interval, false, |s| {
            (
                s.expressions().filter(|e| e.is_recursive()).count(),
                s.population_of(&succ()),
                s.population_of(&add()),
            )
        });
        populations.extend(pops);
        let n_remaining = 1000 - soup.expressions().filter(|e| e.is_recursive()).count();
        let tests = [|| test_succ(random::<usize>() % 20)]
            .into_iter()
            .map(|f| f())
            .cycle()
            .take(n_remaining);
        soup.perturb_test_expressions(n_remaining, tests);
        let skips = asymmetric_skip_sample();
        soup.perturb_lambda_expressions(200, skips);

        println!("Soup {id} {}0% done", i + 1);
    }
    (id, populations)
}

async fn simulate_additive_murder(
    sample: impl Iterator<Item = Term>,
    id: usize,
    run_length: usize,
    polling_interval: usize,
) -> (usize, Vec<usize>) {
    let mut soup = experiment_soup(ConfigSeed::new([0; 32]));
    soup.add_lambda_expressions(sample);
    let check_series =
        soup.simulate_and_poll_with_killer(run_length, polling_interval, false, |s| {
            (
                s.collisions(),
                s.expressions()
                    .any(|e| e.get_underlying_term().is_isomorphic_to(&add())),
            )
        });
    (id, check_series)
}

pub fn add_search_no_test() {
    let mut futures = FuturesUnordered::new();
    let run_length = 1000000;
    let polling_interval = 1000;
    let sample = read_inputs().collect::<Vec<Term>>();
    for i in 0..1000 {
        futures.push(spawn(simulate_additive_murder(
            sample.clone().into_iter().cycle().take(10000),
            i,
            run_length,
            polling_interval,
        )));
    }

    print!("Soup, ");
    println!();
    while let Some((id, series)) = block_on(futures.next()) {
        print!("{}, ", id);
        for i in series {
            print!("{:?}, ", i)
        }
        println!();
    }
}

pub fn add_search_with_test() {
    let mut futures = FuturesUnordered::new();
    let run_length = 100000;
    let polling_interval = 1000;
    for i in 0..16 {
        let sample = asymmetric_skip_sample();
        dump_sample(&sample);

        let distribution = sample.clone().into_iter().cycle().take(5000);
        let tests = [
            || test_succ(random::<usize>() % 20),
            || test_add(random::<usize>() % 20, random::<usize>() % 20),
        ]
        .into_iter()
        .map(|f| f())
        .cycle()
        .take(1000);
        futures.push(spawn(add_magic_tests(
            distribution,
            tests,
            i,
            run_length,
            polling_interval,
        )));
    }

    let fname = "add-search-output";
    while let Some((id, series)) = block_on(futures.next()) {
        dump_series_to_file(fname, &series, &[id]).expect("Cannot write to file");
    }
}

pub fn succ_search_with_test() {
    let mut futures = FuturesUnordered::new();
    let run_length = 100000;
    let polling_interval = 1000;
    for i in 0..16 {
        let sample = asymmetric_skip_sample();
        dump_sample(&sample);

        let distribution = sample.clone().into_iter().cycle().take(5000);
        let tests = [|| test_succ(random::<usize>() % 20)]
            .into_iter()
            .map(|f| f())
            .cycle()
            .take(1000);
        futures.push(spawn(succ_magic_tests(
            distribution,
            tests,
            i,
            run_length,
            polling_interval,
        )));
    }

    let fname = "scc-search-output";
    while let Some((id, series)) = block_on(futures.next()) {
        dump_series_to_file(fname, &series, &[id]).expect("Cannot write to file");
    }
}
