use nalgebra_glm::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

impl BoundingSphere {
    pub fn new() -> Self {
        return BoundingSphere {
            center: Vec3::zeros(),
            radius: 0.0,
        }
    }

    pub fn from_points(points: Vec<Vec3>) -> Self {
        if points.len() == 0 {
            return BoundingSphere {
                center: Vec3::zeros(),
                radius: 0.0,
            }
        }

        // From "Real-Time Collision Detection" (Page 89)
        let mut minx = Vec3::new(std::f32::MAX, std::f32::MAX, std::f32::MAX);
        let mut maxx = -minx;
        let mut miny = minx;
        let mut maxy = -minx;
        let mut minz = minx;
        let mut maxz = -minx;

        // Find the most extreme points along the principle axis.
        let mut num_points = 0;           
        for pt in points.iter() {
            num_points += 1;

            if pt.x < minx.x {
                minx = pt.clone();
            }
            if pt.x > maxx.x {
                maxx = pt.clone();
            }
            if pt.y < miny.y {
                miny = pt.clone();
            }
            if pt.y > maxy.y {
                maxy = pt.clone();
            }
            if pt.z < minz.z {
                minz = pt.clone();
            }
            if pt.z > maxz.z {
                maxz = pt.clone();
            }
        }

        if num_points == 0 {
            panic!("You should have at least one point in points.");
        }

        let sq_dist_x = nalgebra_glm::distance2(&maxx, &minx);
        let sq_dist_y = nalgebra_glm::distance2(&maxy, &miny);
        let sq_dist_z = nalgebra_glm::distance2(&maxz, &minz);

        // Pick the pair of most distant points.
        let mut min = minx;
        let mut max = maxx;
        if sq_dist_y > sq_dist_x && sq_dist_y > sq_dist_z
        {
            max = maxy;
            min = miny;
        }
        if sq_dist_z > sq_dist_x && sq_dist_z > sq_dist_y
        {
            max = maxz;
            min = minz;
        }
        
        let mut center = (min + max) * 0.5;
        let mut radius = nalgebra_glm::distance(&max, &center);
        
        // Test every point and expand the sphere.
        // The current bounding sphere is just a good approximation and may not enclose all points.            
        // From: Mathematics for 3D Game Programming and Computer Graphics, Eric Lengyel, Third Edition.
        // Page 218
        let mut sq_radius = radius * radius;
        for pt in points.iter() {
            let diff: Vec3 = pt - center;
            let sq_dist: f32 = diff.magnitude_squared();
            if sq_dist > sq_radius
            {
                let distance = sq_dist.sqrt(); // equal to diff.Length();
                let direction = diff / distance;
                let g = center - radius * direction;
                center = (g + pt) / 2.0;
                radius = nalgebra_glm::distance(pt, &center);
                sq_radius = radius * radius;
            }
        }

        return BoundingSphere {
            center,
            radius
        }
    }

    // Creates the smallest BoundingSphere that can contain a Vec<BoundingSphere>
    pub fn from_bounding_spheres(mut spheres: Vec<&BoundingSphere>) -> Self {
        let mut current = spheres.remove(0).clone();
        for sphere in spheres {
            current = current.merge(sphere);
        }

        current
    }

    pub fn merge(&self, additional: &BoundingSphere) -> Self {
        let mut ocenter_to_acenter: Vec3 = additional.center - self.center;
        let distance = ocenter_to_acenter.magnitude();
        if distance <= self.radius + additional.radius
        {
            if distance <= self.radius - additional.radius //original contain additional
            {
                return self.clone();
            }
            if distance <= additional.radius - self.radius //additional contain original
            {
                return additional.clone();
            }
        }
        //else find center of new sphere and radius
        let left_radius = (self.radius - distance).max(additional.radius);
        let right_radius = (self.radius + distance).max(additional.radius);
        ocenter_to_acenter = ocenter_to_acenter + (((left_radius - right_radius) / (2.0 * ocenter_to_acenter.magnitude())) * ocenter_to_acenter);

        BoundingSphere {
            center: self.center + ocenter_to_acenter,
            radius: (left_radius + right_radius) / 2.0,
        }
    }

    pub fn intersects_sphere(&self, other: &BoundingSphere) -> bool {
        let sq_dist = nalgebra_glm::distance2(&self.center, &other.center);
        let sphere_radius_combined = other.radius + self.radius;
        let sphere_radius_subtracted = self.radius - other.radius;

        if sq_dist > sphere_radius_combined * sphere_radius_combined {
            return false;
        } else if sq_dist <= sphere_radius_subtracted * sphere_radius_subtracted {
            return true;
        } else {
            return true;
        }
    }

        
}