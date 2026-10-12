//! Petri-net specification and compilation to packed 64-bit transition masks.
//!
//! A [`NetSpec`] is a name-based description (places and transitions reference
//! each other by name). [`NetSpec::compile`] resolves names to bit indices,
//! enforces structural invariants (size caps, uniqueness, no self-loops), and
//! packs each transition's pre/post sets into one `u64` word:
//!
//! ```text
//! words[i] = ((post_mask as u64 & 0xFFFF) << 16) | (pre_mask & 0xFFFF)
//! ```
//!
//! The transition id is implicit: `tid == i (mod 256)`.

use super::error::ConformanceError;
use super::state::Marking64;

/// Maximum places supported by the 64-bit marking encoding.
const MAX_PLACES: usize = 64;

/// Maximum transitions supported by the compiled word vector.
const MAX_TRANSITIONS: usize = 256;

/// A declared place in a [`NetSpec`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceDef {
    /// Unique place name.
    pub name: String,
    /// When true, any reachable marking containing this place is forbidden.
    pub forbidden: bool,
}

/// A declared transition in a [`NetSpec`]; pre/post arcs reference place names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionDef {
    /// Unique transition name.
    pub name: String,
    /// The activity label emitted when this transition fires.
    pub activity: String,
    /// Input place names (consumed tokens).
    pub pre: Vec<String>,
    /// Output place names (produced tokens).
    pub post: Vec<String>,
}

/// A validated, name-resolved net.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetSpec {
    places: Vec<PlaceDef>,
    transitions: Vec<TransitionDef>,
}

/// A compiled net: one packed `u64` word per transition.
///
/// `words[i] = ((post_mask & 0xFFFF) << 16) | (pre_mask & 0xFFFF)`; the
/// transition id is implicit as `i (mod 256)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledNet {
    /// Number of places (guaranteed `<= 64`).
    pub places: u8,
    /// Packed pre/post masks, one word per transition (index = tid mod 256).
    pub words: Vec<u64>,
}

impl NetSpec {
    /// Start building an empty net.
    #[must_use]
    pub const fn builder() -> NetBuilder {
        NetBuilder { spec: Self { places: Vec::new(), transitions: Vec::new() } }
    }

    /// The declared places, in declaration order (index = bit index).
    #[must_use]
    pub fn places(&self) -> &[PlaceDef] {
        &self.places
    }

    /// The declared transitions, in declaration order (index = tid mod 256).
    #[must_use]
    pub fn transitions(&self) -> &[TransitionDef] {
        &self.transitions
    }

    /// Compile the spec into packed transition masks.
    ///
    /// Refuses: more than [`MAX_PLACES`] places, more than
    /// [`MAX_TRANSITIONS`] transitions, duplicate place or transition names,
    /// references to undeclared places, and self-loops (a transition with a
    /// place in both its pre and post sets).
    ///
    /// # Errors
    /// Returns [`ConformanceError`] describing the first structural violation.
    pub fn compile(&self) -> Result<CompiledNet, ConformanceError> {
        if self.places.len() > MAX_PLACES {
            return Err(ConformanceError::TooManyPlaces { limit: MAX_PLACES });
        }
        if self.transitions.len() > MAX_TRANSITIONS {
            return Err(ConformanceError::ModelTooLarge {
                found: self.transitions.len(),
                limit: MAX_TRANSITIONS,
            });
        }

        // Name -> bit index resolution table.
        let mut index: std::collections::HashMap<&str, u32> =
            std::collections::HashMap::with_capacity(self.places.len());
        for (i, place) in self.places.iter().enumerate() {
            // JUSTIFICATION: place count is capped at MAX_PLACES = 64 before
            // this loop, so the index always fits a u32 bit position.
            #[allow(clippy::cast_possible_truncation)]
            let i = i as u32;
            if index.insert(place.name.as_str(), i).is_some() {
                return Err(ConformanceError::BcinrValidation {
                    message: format!("duplicate place name '{}'", place.name),
                });
            }
        }

        let mut seen_transitions: std::collections::HashSet<&str> =
            std::collections::HashSet::with_capacity(self.transitions.len());
        let mut words: Vec<u64> = Vec::with_capacity(self.transitions.len());

        for transition in &self.transitions {
            if !seen_transitions.insert(transition.name.as_str()) {
                return Err(ConformanceError::BcinrValidation {
                    message: format!("duplicate transition name '{}'", transition.name),
                });
            }

            let pre_mask = Self::resolve_set(&transition.name, &transition.pre, &index)?;
            let post_mask = Self::resolve_set(&transition.name, &transition.post, &index)?;

            if pre_mask & post_mask != 0 {
                return Err(ConformanceError::BcinrValidation {
                    message: format!(
                        "self-loop on transition '{}': a place appears in both pre and post",
                        transition.name
                    ),
                });
            }

            words.push(((u64::from(post_mask) & 0xFFFF) << 16) | u64::from(pre_mask));
        }

        Ok(CompiledNet {
            // JUSTIFICATION: place count is capped at MAX_PLACES = 64 above,
            // so it always fits a u8.
            #[allow(clippy::cast_possible_truncation)]
            places: self.places.len() as u8,
            words,
        })
    }

    /// Initial marking: every declared place holds a token.
    ///
    /// Starting fully marked lets forbidden-place policies detect violations
    /// from step zero.
    #[must_use]
    pub fn initial_marking(&self) -> Marking64 {
        let mut marking = 0u64;
        for i in 0..self.places.len().min(MAX_PLACES) {
            marking |= 1u64 << i;
        }
        Marking64(marking)
    }

    /// Resolve a set of place names to a bit mask, refusing unknown names.
    fn resolve_set(
        transition: &str,
        names: &[String],
        index: &std::collections::HashMap<&str, u32>,
    ) -> Result<u16, ConformanceError> {
        let mut mask = 0u16;
        for name in names {
            let bit = index.get(name.as_str()).ok_or_else(|| ConformanceError::UnknownNodeRef {
                side: "place",
                id: format!("{name} (referenced by transition '{transition}')"),
            })?;
            mask |= 1u16 << bit;
        }
        Ok(mask)
    }
}

/// Fluent builder for [`NetSpec`].
#[derive(Debug, Clone, Default)]
pub struct NetBuilder {
    spec: NetSpec,
}

impl NetBuilder {
    /// Declare a place (not forbidden).
    #[must_use]
    pub fn place(mut self, name: impl Into<String>) -> Self {
        self.spec.places.push(PlaceDef { name: name.into(), forbidden: false });
        self
    }

    /// Declare a forbidden place: any marking containing it is a violation.
    #[must_use]
    pub fn forbidden_place(mut self, name: impl Into<String>) -> Self {
        self.spec.places.push(PlaceDef { name: name.into(), forbidden: true });
        self
    }

    /// Declare a transition; subsequent [`add_pre`](Self::add_pre) /
    /// [`add_post`](Self::add_post) calls attach to it.
    #[must_use]
    pub fn transition(mut self, name: impl Into<String>, activity: impl Into<String>) -> Self {
        self.spec.transitions.push(TransitionDef {
            name: name.into(),
            activity: activity.into(),
            pre: Vec::new(),
            post: Vec::new(),
        });
        self
    }

    /// Add an input arc from `place` to the most recently declared transition.
    #[must_use]
    pub fn add_pre(mut self, place: impl Into<String>) -> Self {
        if let Some(t) = self.spec.transitions.last_mut() {
            t.pre.push(place.into());
        }
        self
    }

    /// Add an output arc from the most recently declared transition to `place`.
    #[must_use]
    pub fn add_post(mut self, place: impl Into<String>) -> Self {
        if let Some(t) = self.spec.transitions.last_mut() {
            t.post.push(place.into());
        }
        self
    }

    /// Finish building.
    #[must_use]
    pub fn build(self) -> NetSpec {
        self.spec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Happy path: two places, one transition; verify packed word layout.
    #[test]
    fn compile_happy_path_packs_pre_and_post_masks() {
        let spec = NetSpec::builder()
            .place("p0")
            .place("p1")
            .transition("t0", "submit")
            .add_pre("p0")
            .add_post("p1")
            .build();

        let compiled = spec.compile().expect("compiles");
        assert_eq!(compiled.places, 2);
        assert_eq!(compiled.words.len(), 1);
        // pre bit 0 => low half 0b01; post bit 1 => high half 0b01 << 16.
        assert_eq!(compiled.words[0], (1u64 << 17) | 1);
    }

    /// More than 64 places is refused with `TooManyPlaces`.
    #[test]
    fn compile_refuses_more_than_64_places() {
        let mut builder = NetSpec::builder();
        for i in 0..65 {
            builder = builder.place(format!("p{i}"));
        }
        let err = builder.build().compile().expect_err("must refuse");
        assert_eq!(err, ConformanceError::TooManyPlaces { limit: 64 });
    }

    /// More than 256 transitions is refused with `ModelTooLarge`.
    #[test]
    fn compile_refuses_more_than_256_transitions() {
        let mut builder = NetSpec::builder().place("p0");
        for i in 0..257 {
            builder = builder.transition(format!("t{i}"), "act");
        }
        let err = builder.build().compile().expect_err("must refuse");
        assert_eq!(err, ConformanceError::ModelTooLarge { found: 257, limit: 256 });
    }

    /// Duplicate place names are refused.
    #[test]
    fn compile_refuses_duplicate_place_name() {
        let spec = NetSpec::builder()
            .place("p0")
            .place("p0")
            .transition("t0", "act")
            .add_pre("p0")
            .build();
        let err = spec.compile().expect_err("must refuse");
        assert!(err.to_string().contains("duplicate place name 'p0'"), "unexpected error: {err:?}");
    }

    /// Duplicate transition names are refused.
    #[test]
    fn compile_refuses_duplicate_transition_name() {
        let spec = NetSpec::builder()
            .place("p0")
            .transition("t0", "act")
            .add_pre("p0")
            .transition("t0", "act-again")
            .build();
        let err = spec.compile().expect_err("must refuse");
        assert!(
            err.to_string().contains("duplicate transition name 't0'"),
            "unexpected error: {err:?}"
        );
    }

    /// An arc referencing an undeclared place is refused.
    #[test]
    fn compile_refuses_unknown_place_reference() {
        let spec = NetSpec::builder()
            .place("p0")
            .transition("t0", "act")
            .add_pre("p0")
            .add_post("ghost")
            .build();
        let err = spec.compile().expect_err("must refuse");
        assert!(err.to_string().contains("ghost"), "unexpected error: {err:?}");
    }

    /// A transition consuming and producing the same place (self-loop) is refused.
    #[test]
    fn compile_refuses_self_loop() {
        let spec = NetSpec::builder()
            .place("p0")
            .transition("t0", "act")
            .add_pre("p0")
            .add_post("p0")
            .build();
        let err = spec.compile().expect_err("must refuse");
        assert!(err.to_string().contains("self-loop"), "unexpected error: {err:?}");
    }

    /// Initial marking sets one bit per declared place.
    #[test]
    fn initial_marking_marks_all_declared_places() {
        let spec = NetSpec::builder()
            .place("a")
            .place("b")
            .place("c")
            .transition("t", "act")
            .add_pre("a")
            .add_post("b")
            .build();
        assert_eq!(spec.initial_marking().0, 0b111);
    }
}
