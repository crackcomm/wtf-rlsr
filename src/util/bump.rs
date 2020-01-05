//! Semver version bump utilities.

/// Possible bump selections.
pub static BUMPS: &'static [Bump] = &[Bump::Chore, Bump::Patch, Bump::Minor, Bump::Major];

/// Kind of a semver bump.
#[derive(Copy, Clone)]
pub enum Bump {
    Chore,
    Patch,
    Minor,
    Major,
}

impl Bump {
    /// Returns true if bump is a `Chore`.
    pub fn is_chore(&self) -> bool {
        match self {
            Bump::Chore => true,
            _ => false,
        }
    }

    /// Returns true if bump is a `Patch`.
    pub fn is_patch(&self) -> bool {
        match self {
            Bump::Patch => true,
            _ => false,
        }
    }

    /// Returns true if bump is a `Minor`.
    pub fn is_minor(&self) -> bool {
        match self {
            Bump::Minor => true,
            _ => false,
        }
    }

    /// Returns true if bump is a `Major`.
    pub fn is_major(&self) -> bool {
        match self {
            Bump::Major => true,
            _ => false,
        }
    }
}

/// Version bump extension.
pub trait BumpExt {
    /// Bumps semver version with a given bump kind.
    fn bump(&self, bump: &Bump) -> Self;
}

impl BumpExt for semver::Version {
    fn bump(&self, bump: &Bump) -> semver::Version {
        let mut ver = self.clone();
        match bump {
            Bump::Chore => {}
            Bump::Patch => ver.increment_patch(),
            Bump::Minor => ver.increment_minor(),
            Bump::Major => ver.increment_major(),
        }
        ver
    }
}
