extern crate nalgebra as na;

use na::{SMatrix, Matrix4, Vector3, RowVector4};

struct ViewPort {
    u_min: f64,
    v_min: f64,
    u_max: f64,
    v_max: f64,
}

struct Window {
    x_min: f64,
    y_min: f64,
    x_max: f64,
    y_max: f64,
}

struct AspectRatio {
    su: f64,
    sv: f64,
    n: f64,
    f: f64,
}

fn main() {
    let points = SMatrix::<f64, 5, 4>::new
                        (-2.0,	-1.0,	4.0,	1.0,
                        3.0,	-2.0,	5.0,	1.0,
                        4.0,	-1.0,	-2.0,	1.0,
                        -1.0,	0.0,	-3.0,	1.0,
                        1.0,	6.0,	1.0,	1.0);

    let vrp = Vector3::new(30.0, 40.0, 100.0);
    let view_up = Vector3::new(0.0, 1.0, 0.0);
    let p = Vector3::new(1.0, 2.0, 1.0);

    let vp = ViewPort {
        u_min: 100.0,
        v_min: 300.0,
        u_max: 1000.0,
        v_max: 900.0,
    };  

    let window = Window {
        x_min: -10.0,
        y_min: -8.0,
        x_max: 10.0,
        y_max: 8.0,
    };

    let aspect_ratio = AspectRatio {
        su: 10.0,
        sv: 8.0,
        n: 20.0,
        f: 120.0,
    };

    let dp = 50.0;
    let cu = 0.0;
    let cv = 0.0;

    let zmin = 0.0;
    let zmax = 65535.0;

    let n = p - vrp;
    let n = n.normalize();

    let v = view_up - n * (view_up.dot(&n));
    let v = v.normalize();

    let u = n.cross(&v);

    println!("n: {:?}", n);
    println!("v: {:?}", v);
    println!("u: {:?}", u);

    // let mut a = Matrix4::<f64>::identity();
    // a.set_row(3, &RowVector4::new(-vrp.x, -vrp.y, -vrp.z, 1.0));

    // let mut b = Matrix4::<f64>::identity();

}

