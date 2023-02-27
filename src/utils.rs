pub const ST2084_Y_MAX: f64 = 10000.0;
pub const ST2084_M1: f64 = 2610.0 / 16384.0;
pub const ST2084_M2: f64 = (2523.0 / 4096.0) * 128.0;
pub const ST2084_C1: f64 = 3424.0 / 4096.0;
pub const ST2084_C2: f64 = (2413.0 / 4096.0) * 32.0;
pub const ST2084_C3: f64 = (2392.0 / 4096.0) * 32.0;

#[inline(always)]
pub fn pq_to_nits(x: f64) -> f64 {
    if x > 0.0 {
        let xpow = x.powf(1.0 / ST2084_M2);
        let num = (xpow - ST2084_C1).max(0.0);
        let den = (ST2084_C2 - ST2084_C3 * xpow).max(f64::NEG_INFINITY);

        (num / den).powf(1.0 / ST2084_M1) * ST2084_Y_MAX
    } else {
        0.0
    }
}

/// Helper function to calculate PQ codes from nits (cd/m2) values
#[inline(always)]
pub fn nits_to_pq(nits: f64) -> f64 {
    let y = nits / ST2084_Y_MAX;

    ((ST2084_C1 + ST2084_C2 * y.powf(ST2084_M1)) / (1.0 + ST2084_C3 * y.powf(ST2084_M1)))
        .powf(ST2084_M2)
}
