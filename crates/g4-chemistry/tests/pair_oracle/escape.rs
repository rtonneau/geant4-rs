//! Continuum upper bound on return to a unit contact sphere after escape.
//! This bounds conditional expectations, not individual trajectory outcomes;
//! Monte Carlo uncertainty and numerical crossing errors must be added separately.
pub fn return_probability(charge: f64, radius: f64) -> f64 {
    assert!(charge.is_finite() && radius.is_finite() && radius >= 1.0);
    if charge == 0.0 {
        1.0 / radius
    } else if charge > 0.0 {
        (charge / radius - charge).exp() * (-(charge / radius)).exp_m1() / (-charge).exp_m1()
    } else {
        (charge / radius).exp_m1() / charge.exp_m1()
    }
}

pub fn reaction_interval(
    reacted: usize,
    escaped: usize,
    unresolved: usize,
    n: usize,
    return_bound: f64,
) -> (f64, f64) {
    assert!(n > 0 && reacted + escaped + unresolved == n);
    assert!((0.0..=1.0).contains(&return_bound));
    (
        reacted as f64 / n as f64,
        (reacted as f64 + unresolved as f64 + escaped as f64 * return_bound) / n as f64,
    )
}
