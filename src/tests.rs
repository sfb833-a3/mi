use std::collections::HashMap;

lazy_static! {
    //pub static ref EVENT_FREQS: Vec<HashMap<usize, usize>> = vec![[(1, 3), (3, 4), (5, 5)].iter().cloned().collect(), [(2, 4), (4, 7)].iter().cloned().collect()];

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
}