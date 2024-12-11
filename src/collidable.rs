pub trait Particle {
    fn compose(&self, other: &Self) -> Self;

    fn is_isomorphic_to(&self, other: &Self) -> bool;
}

pub trait Collider<P, T, E>
where
    P: Particle,
{
    fn collide(&self, left: P, right: P) -> Result<T, E>;
}
