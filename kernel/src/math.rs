


#[macro_export]
macro_rules! lerp_int {
    ($a:expr, $b:expr, $t:expr) => {{
        let a = $a as u128;
        let b = $b as u128;
        let t = $t as u128;

        let t = if t > 255 {255} else {t};

        ((a * (255 - t) + b*t)/255) as u128
    }};
}

#[macro_export]
macro_rules! lerp_float {
    ($a:expr, $b:expr, $t:expr) => {{
        let a = $a as f64;
        let b = $b as f64;
        let t = $t as f64;

        let t = t.clamp(0.0, 1.0);

        (a * (1.0 - t) + b*t) as u128
    }};
}
