//! Semver version bump utilities.

/// Possible bump selections.
pub static UPDATES: &'static [Update] = &[
    Update::Docs,
    Update::Chore,
    Update::Patch,
    Update::Minor,
    Update::Major,
];

/// Kind of update.
#[derive(Copy, Clone)]
pub enum Update {
    Docs,
    Chore,
    Patch,
    Minor,
    Major,
}

impl Update {
    /// Returns update commit type.
    pub fn commit_type(&self) -> &'static str {
        match self {
            Update::Docs => "docs",
            Update::Chore => "chore",
            Update::Patch => "fix",
            Update::Minor => "feat",
            Update::Major => "feat",
        }
    }

    /// Returns update commit description.
    pub fn commit_description(&self) -> &'static str {
        match self {
            Update::Docs => "docs update",
            Update::Chore => "cleanup",
            Update::Patch => "patch",
            Update::Minor => "minor update",
            Update::Major => "major update",
        }
    }

    /// Returns update bump kind.
    pub fn as_bump(&self) -> Option<Bump> {
        match self {
            Update::Docs => None,
            Update::Chore => None,
            Update::Patch => Some(Bump::Patch),
            Update::Minor => Some(Bump::Minor),
            Update::Major => Some(Bump::Major),
        }
    }

    /// Returns update bump kind.
    pub fn bump(&self, ver: &semver::Version) -> semver::Version {
        if let Some(bump) = self.as_bump() {
            ver.bump(bump)
        } else {
            ver.clone()
        }
    }

    /// Formats version transition according to update.
    pub fn transition(&self, version: &semver::Version) -> String {
        if let Some(bump) = self.as_bump() {
            format!("v{} → {}", version, version.bump(bump))
        } else {
            format!("v{}", version)
        }
    }
}

/// Kind of a semver bump.
#[derive(Copy, Clone)]
pub enum Bump {
    Patch,
    Minor,
    Major,
}

impl Bump {
    /// Returns dependency bump.
    pub fn dependency(&self, changed: bool, commit: bool) -> Self {
        if !changed {
            match self {
                Bump::Major => Bump::Minor,
                Bump::Minor | Bump::Patch => Bump::Patch,
            }
        } else if !commit {
            match self {
                Bump::Major | Bump::Minor => Bump::Minor,
                Bump::Patch => *self,
            }
        } else {
            *self
        }
    }
}

/// Version bump extension.
pub trait BumpExt {
    /// Bumps semver version with a given bump kind.
    fn bump(&self, bump: Bump) -> Self;
}

impl BumpExt for semver::Version {
    fn bump(&self, bump: Bump) -> semver::Version {
        let mut ver = self.clone();
        match bump {
            Bump::Patch => ver.increment_patch(),
            Bump::Minor => ver.increment_minor(),
            Bump::Major => ver.increment_major(),
        }
        ver
    }
}
