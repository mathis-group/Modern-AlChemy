/// Default constructor trait, implemented by all reactor configs.
///
/// Reactor configs are expression-specific and are selected through
/// `config::expression::Expression`, so there is no reactor enum here.
pub trait ReactorConfig {
    fn new() -> Self;
}
