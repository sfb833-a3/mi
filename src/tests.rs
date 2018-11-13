use std::collections::HashMap;

lazy_static! {
    pub static ref EVENT_FREQS: Vec<HashMap<usize, usize>> = vec![
        hashmap!{
            1 => 3,
            3 => 4,
            5 => 5,
        },
        hashmap!{
            2 => 4,
            4 => 7,
        },
    ];
    pub static ref JOINT_FREQS: HashMap<[usize; 2], usize> = hashmap! {
        [1, 1] => 1,
        [1, 2] => 2,
        [1, 3] => 1,
        [1, 4] => 7,
    };
}
