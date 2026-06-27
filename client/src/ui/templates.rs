#[derive(Clone)]
pub struct Template {
    pub name: &'static str,
    pub equation: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
}

pub const TEMPLATES: &[Template] = &[
    Template {
        name: "Sphere",
        equation: "sqrt(x*x + y*y + z*z) - 10.0",
        tag: "geometry",
        description: "Basic signed distance sphere",
    },
    Template {
        name: "Infinite Cylinder",
        equation: "sqrt(x*x + y*y) - 5.0",
        tag: "geometry",
        description: "Endless tube along the Z axis",
    },
    Template {
        name: "Sphere with Hole",
        equation: "Max(sqrt(x*x + y*y + z*z) - 10.0, -(sqrt(x*x + y*y) - 4.0))",
        tag: "geometry",
        description: "Sphere with a cylindrical void through center",
    },
    Template {
        name: "Gyroid",
        equation: "sin(x)*cos(y) + sin(y)*cos(z) + sin(z)*cos(x)",
        tag: "advanced",
        description: "Triply periodic minimal surface",
    },
    Template {
        name: "Torus",
        equation: "sqrt((sqrt(x*x + z*z) - 8.0)*(sqrt(x*x + z*z) - 8.0) + y*y) - 3.0",
        tag: "geometry",
        description: "Ring doughnut shape",
    },
    Template {
        name: "Orbital Tracking",
        equation: "sqrt((x - state.x)*(x - state.x) + (y - state.y)*(y - state.y) + (z - state.z)*(z - state.z)) - 5.0",
        tag: "physics",
        description: "Follows a moving entity with a spherical probe",
    },
];
