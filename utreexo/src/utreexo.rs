use super::error::UError;
use super::hash::Hash;
use super::proof::{Proof, ProofStep};
use super::update::Update;

use std::collections::HashMap;

pub type HashFn = fn(&Hash, &Hash) -> Hash;

#[derive(Clone)]
pub struct Utreexo {
    pub roots: Vec<Option<Hash>>,
    pub hasher: HashFn,
}

struct Worktree {
    heights: Vec<Vec<Hash>>,
    roots: HashMap<Hash, usize>,
}

impl Utreexo {
    pub fn new(hasher: HashFn) -> Utreexo {
        return Utreexo {
            roots: Vec::new(),
            hasher: hasher,
        };
    }

    pub fn update(&mut self, deletions: &[Proof], insertions: &[Hash]) -> Result<Update, UError> {
        let mut w = Worktree {
            heights: Vec::new(),
            roots: HashMap::new(),
        };
        let mut update = Update::new(self.clone());

        for d in deletions {
            let i: usize;
            let j: usize;
            match self.del_helper(&w, &d.leaf, &d.steps, 0, None) {
                Ok(pair) => {
                    i = pair.0;
                    j = pair.1
                }
                Err(e) => return Err(e),
            }
            w.roots.remove(&w.heights[i][j]);
            w.heights[i].swap_remove(j);

            for (k, s) in d.steps.iter().enumerate() {
                w.heights[k].push(s.h.clone());
            }
        }

        if w.heights.is_empty() {
            w.heights.push(Vec::new());
        }
        w.heights[0].extend_from_slice(insertions);

        let mut i = 0;
        while i < w.heights.len() {
            while w.heights[i].len() > 1 {
                let b = w.heights[i].pop().unwrap();
                let a = w.heights[i].pop().unwrap();
                let h = (self.hasher)(&a, &b);

                if w.heights.len() <= i + 1 {
                    w.heights.push(Vec::new());
                }
                w.heights[i + 1].push(h);
                update.add(&a, &b);
            }
            i += 1;
        }

        i = w.heights.len();
        while i > 0 {
            i -= 1;
            if !w.heights[i].is_empty() {
                break;
            }
            w.heights.pop();
        }

        for (i, h) in w.heights.iter().enumerate() {
            if self.roots.len() <= i {
                self.roots.push(None);
            }
            self.roots[i] = if h.is_empty() {
                None
            } else {
                Some(h[0].clone())
            }
        }

        self.roots.truncate(w.heights.len());

        Ok(update)
    }

    fn del_helper(
        &self,
        w: &Worktree,
        item: &Hash,
        steps: &[ProofStep],
        height: usize,
        j: Option<usize>,
    ) -> Result<(usize, usize), UError> {
        if steps.is_empty() {
            if height >= self.roots.len() {
                return Err(UError::Invalid);
            }
            match &self.roots[height] {
                None => return Err(UError::Invalid),
                Some(h) => {
                    if item != h {
                        return Err(UError::Invalid);
                    }
                }
            }
            if w.heights.is_empty() {
                return Err(UError::Invalid);
            }
            match j {
                None => match find_root(item, &w.heights[0][..]) {
                    None => return Err(UError::Invalid),
                    Some(jj) => return Ok((height, jj)),
                },
                Some(jj) => return Ok((height, jj)),
            }
        }

        let new_item = if steps[0].left {
            (self.hasher)(&steps[0].h, item)
        } else {
            (self.hasher)(item, &steps[0].h)
        };

        let jj = match (j, w.roots.get(&new_item)) {
            (None, Some(h)) => match find_root(&new_item, &w.heights[*h][..]) {
                None => return Err(UError::Invalid),
                Some(k) => Some(k),
            },
            _ => j,
        };

        self.del_helper(w, &new_item, &steps[1..], height + 1, jj)
    }
}

fn find_root(h: &Hash, hashes: &[Hash]) -> Option<usize> {
    for (i, hh) in hashes.iter().enumerate() {
        if h == hh {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utreexo() {
        let items: [Hash; 12]; // xxx initialize with hashes
                               // xxx define hashfn
        let mut u = utreexo::new(hashfn);

        // Try to delete from an empty tree, should give an invalid-proof error.
        let p1 = Proof {
            leaf: items[0].clone(),
            steps: vec![],
        };
        match u.update(&[p1], &[]) {
            Err(uerr) => {
                if uerr != UError::Invalid {
                    // xxx unexpected error
                }
            }
            _ => {
                // xxx unexpected success
            }
        }

        // Add 11 leaves.
        let mut proofs: [Proof; 11];
        match u.update(&[], &items[..11]) {
            Err(uerr) => {
                // xxx unexpected error
            }
            Ok(upd) => {
                for i in 0..11 {
                    proof[i] = upd.proof(&items[i]);
                }
            }
        }

        // Remove one of them.
        match u.update(&proofs[..10], &[]) {
            Err(uerr) => {
                // xxx unexpected error
            }
            Ok(upd) => {
                for i in 0..10 {
                    if let Err(uerr) = proofs[i].update(upd) {
                        // xxx unexpected error
                    }
                }
                let p10 = proofs[10].clone();
                match p10.update(upd) {
                    Err(uerr) => {
                        if uerr != UError::Invalid {
                            // xxx unexpected error
                        }
                    }
                    _ => {
                        // xxx unexpected success
                    }
                }
            }
        }

        let saved = u.clone();
        match u.update(&[proofs[10]], &[]) {
            Err(uerr) => {
                if uerr != UError::Invalid {
                    // xxx unexpected error
                }
            }
            _ => {
                // xxx unexpected success
            }
        }
        assert!(saved == u);
    }
}
