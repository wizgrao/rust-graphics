use graphics::math::*;
use graphics::path_tracer::bvh::*;
use graphics::path_tracer::primitives::*;
use graphics::path_tracer::Object;
use std::sync::Arc;
fn main() {
    let t1 = Triangle::new(v(-200., 0., 1.), v(-201., 1., 1.), v(-202., 0., 1.));
    let t2 = Triangle::new(v(0., 0., 1.), v(1., 1., 1.), v(2., 0., 1.));
    let tt1 = Solid {
        bsdf: Arc::new(Emissive { emission: O }),
        intersectable: Arc::new(t1),
    };
    let tt2 = Solid {
        bsdf: Arc::new(Emissive { emission: O }),
        intersectable: Arc::new(t2),
    };
    let bvh = BVHNode::new(vec![tt1, tt2], 1);
    dbg!(&bvh);
    dbg!(
        bvh.intersect(&Ray {
            x: O,
            d: normalize(&v(-201., 0.5, 1.))
        })
        .unwrap()
        .0
    );
}
