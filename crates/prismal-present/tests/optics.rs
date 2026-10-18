//! Geometric optics (D-103): the first-party `optics` module, written in Prismal: reflection,
//! Snell's law and total internal reflection, intersections with planes and spheres, thin
//! lenses and images, dispersion.

mod common;
use common::*;
use prismal_runtime::{run, Config, RunStatus};

const SRC: &str = r#"use optics

space Plane = euclidean(2)

model Rays in Plane {
  param {
    n_water: Real   = 1.33
    f:       Length = 10 cm
    R:       Length = 1 m
    n_glass: Real   = 1.5
  }
  derived {
    into_water: Vector<1> = refract((1, -1), (0, 1), 1, n_water)
    out_of_water: Vector<1> = refract((sqrt(3), 1), (0, 1), n_water, 1)
    steep_out: Vector<1> = refract((1, 2), (0, 1), n_water, 1)
    critical: Angle = critical_angle(n_water, 1)
    mirror: Vector<1> = reflect((1, -1), (0, 1))
    seen: Angle = incidence((1, -1), (0, -1))
    on_line: Point = hit_plane(origin + (0 m, 1 m), (1, -1), origin, (0, 1))
    high: Point = origin + (-1 m, 2 cm)
    at_lens: Point = hit_plane(high, (1, 0), origin, (1, 0))
    after_lens: Vector<1> = thin_lens((1, 0), at_lens, origin, (1, 0), f)
    on_axis: Point = hit_plane(at_lens, after_lens, origin, (0, 1))
    image: Length = image_distance(30 cm, f)
    scale: Real = magnification(30 cm, f)
    entry: Point = hit_sphere(origin + (-3 m, 0.5 m), (1, 0), origin, R)
    inside: Vector<1> = refract((1, 0), sphere_normal(entry, origin), 1, n_glass)
    exit: Point = hit_sphere(entry, inside, origin, R)
    misses: Boolean = meets_sphere(origin + (-3 m, 2 m), (1, 0), origin, R)
    blue: Real = cauchy(1.5046, 4200 nm^2, 400 nm)
    red: Real = cauchy(1.5046, 4200 nm^2, 700 nm)
    focal: Length = lensmaker(1.5, 20 cm, -20 cm)
  }
}
"#;

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-9, "{what}: {a} vs {b}");
}

#[test]
fn rays_by_the_optics_module() {
    let prog = program(SRC);
    let cm = prog.model("Rays");
    let r = run(cm, Config::until(0.0));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let s = r.state_at(0.0);
    let v = |n: &str| {
        let mut a = [0.0; 2];
        s[cm.idx(&format!("Rays.{n}"))].write_to(&mut a);
        a
    };
    let x = |n: &str| s[cm.idx(&format!("Rays.{n}"))].num();
    // Into water at 45 degrees: sin θ2 = sin 45° / 1.33.
    let s2 = (0.5f64).sqrt() / 1.33;
    let d = v("into_water");
    close(d[0], s2, "sin of the refracted angle");
    close(d[1], -(1.0 - s2 * s2).sqrt(), "cos of the refracted angle");
    // Out of water at 60 degrees: 1.33 sin 60° > 1, totally reflected.
    let d = v("out_of_water");
    close(d[0], 0.75f64.sqrt(), "reflected x");
    close(d[1], -0.5, "reflected y");
    // Out of water at atan(1/2) = 26.57 degrees, below the critical angle: it leaves.
    let s1 = 1.0 / 5f64.sqrt();
    let d = v("steep_out");
    close(d[0], 1.33 * s1, "sin out of water");
    close(d[1], (1.0 - (1.33 * s1).powi(2)).sqrt(), "cos out of water");
    close(x("critical"), (1.0f64 / 1.33).asin(), "critical angle");
    close(v("mirror")[1], (0.5f64).sqrt(), "mirror");
    close(x("seen"), std::f64::consts::FRAC_PI_4, "angle of incidence");
    close(v("on_line")[0], 1.0, "where the ray meets the line");
    // A ray parallel to the axis, 2 cm above it, crosses the axis at the focus.
    close(v("on_axis")[0], 0.10, "focus");
    close(x("image"), 0.15, "1/30 + 1/v = 1/10");
    close(x("scale"), -0.5, "magnification");
    // A ray 0.5 m above the centre of a glass ball of radius 1 m: it enters at 30 degrees of
    // incidence, `sin r = 0.5 / 1.5`, and leaves where the chord of that angle ends.
    let e = v("entry");
    close(e[0], -(0.75f64).sqrt(), "entry x");
    close(e[1], 0.5, "entry y");
    let (sr, i) = (0.5f64 / 1.5, (0.5f64).asin());
    let r_ang = sr.asin();
    // The chord turns the radius by π - 2r; the entry radius is at angle π - i from +x.
    let a_exit = std::f64::consts::PI - i - (std::f64::consts::PI - 2.0 * r_ang);
    let q = v("exit");
    close(q[0], a_exit.cos(), "exit x");
    close(q[1], a_exit.sin(), "exit y");
    assert!(!s[cm.idx("Rays.misses")].boolean());
    assert!(x("blue") > x("red"), "blue is bent more");
    close(x("blue"), 1.5046 + 4200.0 / 160000.0, "Cauchy at 400 nm");
    close(x("focal"), 0.20, "a biconvex lens of 20 cm faces in glass of 1.5");
}
