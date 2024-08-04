use crate::body::Body;

const G: f64 = 6.67430e-11;


pub fn calculate_forces(mut bodies: Vec<Body>) {
    for i in 0..bodies.len() {
        for j in (i+1)..bodies.len() {
            let r = (bodies[j].x - bodies[i].x, bodies[j].y - bodies[i].y, bodies[j].z - bodies[i].z);
            let m1 = bodies[i].mass;
            let m2 = bodies[j].mass;

            let f = calculate_force_vector(r, m1, m2);
            println!("Body {} to Body {} has force vector ({},{},{})", bodies[i].name, bodies[j].name, f.0, f.1, f.2);
        }
    }
}

fn calculate_force_vector(r: (f64, f64, f64), m1:f64, m2: f64) -> (f64, f64, f64) {
    let mag = (r.0.powi(2) + r.1.powi(2) + r.2.powi(2)).sqrt();
    let r_hat = (r.0/mag, r.1/mag, r.2/mag);
    let f = G * m1 * m2 / mag.powi(2);
    return (f*r_hat.0, f*r_hat.1, f*r_hat.2);
}
