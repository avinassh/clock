use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Clone, Default, Debug)]
pub struct VersionVector {
    vector:HashMap<String, i64>,
    // TODO(kavi): Add support mutex for thread-safe?
}

#[derive(Clone, Debug)]
pub struct Dot (String, i64);

impl VersionVector {
    pub fn new() -> VersionVector {
	VersionVector{
	    vector: HashMap::new(),
	}
    }

    pub fn inc(mut self, node_id:&str) -> Self{
	self.vector.entry(node_id.to_string()).and_modify(|e| *e += 1).or_insert(1);
	self
    }

    pub fn descends(&self, w:&VersionVector) -> bool {
	let keys = VersionVector::all_keys(&[&self.vector, &w.vector]);
	// All the keys from 'self' should be greater than or equal to same key from 'w'.
	// So, now if both self and w are same, then it descends(v, v) => true
	for k in keys.iter() {
	    let v1 = match self.vector.get(k) {
		None => 0,
		Some(v) => *v
	    };
	    let v2 = match w.vector.get(k) {
		None => 0,
		Some(v) => *v
	    };
	    if v1 < v2 {
		return false
	    }
	}
	true
    }

    pub fn concurrent(&self, w:&VersionVector) -> bool {
	// if neither of them descends from another, then they are concurrent
	!(self.descends(w) || w.descends(self))
    }

    pub fn descends_dot(&self, w:&Dot) -> bool {
	let v = match self.vector.get(&w.0) {
	    None => 0,
	    Some(v) => *v
	};
	v >= w.1
    }

    /// merges the two given vectors via point-wise max.
    pub fn merge(&self, w:&VersionVector) -> VersionVector {
	let slice = vec![&self.vector, &w.vector];
	let keys = VersionVector::all_keys(&slice[..]);
	let mut res:HashMap<String, i64> = HashMap::new();
	for k in keys.iter() {
	    let e1 = match self.vector.get(k) {
		None => 0,
		Some(v) => *v
	    };
	    let e2 = match w.vector.get(k) {
		None => 0,
		Some(v) => *v,
	    };

	    res.insert(k.to_string(), std::cmp::max(e1, e2));
	}
	
	VersionVector{
	    vector: res,
	}
    }
    
    pub fn get_dot(&self, node_id:&str) -> Dot {
	let count = match self.vector.get(node_id) {
	    None => 0,
	    Some(v) => *v
	};
	Dot(node_id.to_string(), count)
    }

    fn all_keys(clocks: &[&HashMap<String, i64>]) -> HashSet<String> {
	let mut res = HashSet::new();

	for clock in clocks {
	    for (k, _) in clock.iter() {
		res.insert(k.to_string());
	    }
	}
	res
    }
}


impl Dot {
    pub fn descends_vv(&self, w:&VersionVector) -> bool {
	let v = match w.vector.get(&self.0) {
	    None => 0,
	    Some(v) => *v
	};

	self.1 >= v
    }
    pub fn descends(&self, w:&Dot) -> bool {
	self.0 == w.0 && self.1 >= w.1
    }
}


#[test]
fn test_vv_new() {
    let mut vv = VersionVector::new();
    vv = vv.inc("A").inc("B");

    assert_eq!(vv.vector.get("A").unwrap(), &1_i64);
    assert_eq!(vv.vector.get("B").unwrap(), &1_i64);

    vv = vv.inc("A").inc("C");

    assert_eq!(vv.vector.get("A").unwrap(), &2_i64);
    assert_eq!(vv.vector.get("C").unwrap(), &1_i64);
}

#[test]
fn test_vv_merge() {
    // [2, 1]
    let v1 = VersionVector::new()
	.inc("A")
	.inc("A")
	.inc("B");
    // [1, 2]
    let v2 = VersionVector::new()
	.inc("B")
	.inc("B")
	.inc("A");

    let v3 = v1.merge(&v2);

    // [2, 2]
    assert_eq!(v3.vector.get("A").unwrap(), &2_i64);
    assert_eq!(v3.vector.get("B").unwrap(), &2_i64);
}

#[test]
fn test_vv_descends() {
    // Case 0: v2 descends v1
    // [2, 3, 2]
    let v1 = VersionVector::new()
	.inc("A")
	.inc("A")
	.inc("B")
	.inc("B")
	.inc("B")
	.inc("C")
	.inc("C");
	
    // [3, 4, 2]
    let v2 = VersionVector::new()
	.inc("A")
	.inc("B")
	.inc("B")
	.inc("C");
    
    assert!(v1.descends(&v2));
    assert!(!v2.descends(&v1));
	
    // Case 1: Concurrent
    // [2, 3, 2]
    let v1 = VersionVector::new()
	.inc("A")
	.inc("A")
	.inc("B")
	.inc("B")
	.inc("B")
	.inc("C")
	.inc("C");

    // [1, 4, 1]
    let v2 = VersionVector::new()
	.inc("A")
	.inc("B")
	.inc("B")
	.inc("B")
	.inc("B")
	.inc("C");
    
    assert!(!v1.descends(&v2));
    assert!(!v2.descends(&v1)); // neither v2 descends Case
}

#[test]
fn test_vv_concurrent() {
    // Case 0: not concurrent
    // [2, 3, 2]
    let v1 = VersionVector::new().
	inc("A")
	.inc("A")
	.inc("B")
	.inc("B")
	.inc("B")
	.inc("C")
	.inc("C");
	
    // [3, 4, 2]
    let v2 = VersionVector::new()
	.inc("A")
	.inc("B")
	.inc("B")
	.inc("C");
    
    assert!(!v1.concurrent(&v2));
    assert!(!v2.concurrent(&v1));
	
    // Case 1: Concurrent
    // [2, 3, 2]
    let v1 = VersionVector::new()
	.inc("A")
	.inc("A")
	.inc("B")
	.inc("B")
	.inc("B")
	.inc("C")
	.inc("C");

    // [1, 4, 1]
    let v2 = VersionVector::new().
	inc("A").
	inc("B").
	inc("B").
	inc("B").
	inc("B").
	inc("C");
    assert!(v1.concurrent(&v2));
    assert!(v2.concurrent(&v1));
}

#[test]
fn test_get_dot() {
    let v = VersionVector::new().inc("A").inc("B");
    let dot = v.get_dot("A");

    assert_eq!("A", dot.0);
    assert_eq!(1, dot.1);
}

#[test]
fn test_descends_dot() {
    let v = VersionVector::new()
	.inc("A")
	.inc("A")
	.inc("B");

    let dot = Dot("A".to_string(), 3);

    assert!(dot.descends_vv(&v));
    assert!(!v.descends_dot(&dot));

    let v = VersionVector::new()
	.inc("A")
	.inc("A")
	.inc("B");

    let dot = Dot("A".to_string(), 1);
    assert!(!dot.descends_vv(&v));
    assert!(v.descends_dot(&dot));
    
}
