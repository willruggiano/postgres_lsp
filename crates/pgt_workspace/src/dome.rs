use pgt_fs::PgTPath;
use std::collections::BTreeSet;
use std::collections::btree_set::Iter;
use std::iter::{FusedIterator, Peekable};

/// A type that holds the evaluated paths, and provides an iterator to extract
/// specific paths like configuration files, manifests and more.
#[derive(Debug, Default)]
pub struct Dome {
    paths: BTreeSet<PgTPath>,
}

impl Dome {
    pub fn with_path(mut self, path: impl Into<PgTPath>) -> Self {
        self.paths.insert(path.into());
        self
    }

    pub fn new(paths: BTreeSet<PgTPath>) -> Self {
        Self { paths }
    }

    pub fn iter(&self) -> DomeIterator {
        DomeIterator {
            iter: self.paths.iter().peekable(),
        }
    }

    pub fn to_paths(self) -> BTreeSet<PgTPath> {
        self.paths
    }
}

pub struct DomeIterator<'a> {
    iter: Peekable<Iter<'a, PgTPath>>,
}

impl<'a> DomeIterator<'a> {
    pub fn next_config(&mut self) -> Option<&'a PgTPath> {
        if let Some(path) = self.iter.peek() {
            if path.is_config() {
                self.iter.next()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn next_ignore(&mut self) -> Option<&'a PgTPath> {
        if let Some(path) = self.iter.peek() {
            if path.is_ignore() {
                self.iter.next()
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a> Iterator for DomeIterator<'a> {
    type Item = &'a PgTPath;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl FusedIterator for DomeIterator<'_> {}
