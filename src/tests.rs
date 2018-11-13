use std::collections::HashMap;

lazy_static! {
    pub static ref EVENT_FREQS: Vec<HashMap<usize, usize>> = {
        let mut m1 = HashMap::new();
        m1.insert(1, 3);
        m1.insert(3, 4);
        m1.insert(5, 5);

        let mut m2 = HashMap::new();
        m2.insert(2, 4);
        m2.insert(4, 7);

        let mut v = Vec::new();
        v.push(m1);
        v.push(m2);
        v
    };
    pub static ref JOINT_FREQS: HashMap<[usize; 2], usize> = {
        let mut v = HashMap::new();
        v.insert([1, 1], 1);
        v.insert([1, 2], 2);
        v.insert([1, 3], 1);
        v.insert([1, 4], 7);
        v
    };
}
