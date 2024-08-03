use crate::body::Body;


pub fn calculate_forces(mut bodies: Vec<Body>) {
    for body in bodies {
        let (x, y, z) = (body.x, body.y, body.z);
        println!("Body: {} has position vector ({},{},{})", body.name, x, y, z);
    }
}