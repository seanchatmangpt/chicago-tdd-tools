//! Capability 03 — builders and fixtures on Chatman scenarios:
//! `GenericTestDataBuilder` for scenario structs, a `FixtureProvider`
//! implementation for a profile + symbol-table fixture, and
//! `ScopedMetadata` RAII on `TestFixture`.
//!
//! Contract note: the original brief asked for `#[derive(TestBuilder)]`.
//! chicago-tdd-tools ships no such derive macro (checked src/ — the builder
//! surface is `TestDataBuilder` / `GenericTestDataBuilder` in
//! `core::builders`), so this file demonstrates the builder capability the
//! framework actually exports.

mod chatman_common;

use chicago_tdd_tools::core::builders::GenericTestDataBuilder;
use chicago_tdd_tools::core::fixture::{FixtureError, FixtureProvider, TestFixture};
use chicago_tdd_tools::test;
use praxis_graphlaw::chatman::abi::{ProfileId, Refusal};
use praxis_graphlaw::chatman::triple8::{ProfileSymbolTable, Term8};

/// A profile + frozen symbol-table fixture, provided through the framework's
/// `FixtureProvider` GAT trait.
struct ChatmanProfileProvider {
    profile: ProfileId,
    terms: Vec<String>,
}

impl FixtureProvider for ChatmanProfileProvider {
    type Fixture<'a> = TestFixture<ProfileSymbolTable>;
    type Error = FixtureError;

    fn create_fixture(&self) -> Result<Self::Fixture<'_>, Self::Error> {
        let table = ProfileSymbolTable::build(self.profile.clone(), self.terms.clone())
            .map_err(|refusal| FixtureError::CreationFailed(refusal.to_string()))?;
        Ok(TestFixture::with_data(table))
    }
}

test!(fixture_provider_builds_profile_symbol_table, {
    // Arrange: a provider carrying the profile-scoped term universe.
    let provider = ChatmanProfileProvider {
        profile: ProfileId::new("profile:fixtures"),
        terms: vec!["<urn:s>".to_string(), "<urn:p>".to_string(), "<urn:o>".to_string()],
    };

    // Act
    let fixture = provider
        .create_fixture()
        .map_err(|e| Refusal::ValidationFailed(e.to_string()))?;

    // Assert: the fixture wraps a frozen 3-term universe.
    assert_eq!(fixture.inner().len(), 3);
    assert_eq!(fixture.inner().resolve("<urn:o>"), Ok(Term8(0)));
    Ok::<(), Refusal>(())
});

test!(generic_builder_assembles_invocation_scenario, {
    // Arrange: a fluent scenario map (identity fields for one invocation).
    let scenario = GenericTestDataBuilder::<String, String>::new()
        .with_var("invocation_id".to_string(), "inv:builder".to_string())
        .with_var("snapshot_id".to_string(), "snap:builder".to_string())
        .with_var("profile_id".to_string(), "profile:builder".to_string())
        .with_var("operator_id".to_string(), "op:builder".to_string());

    // Act
    let data = scenario.build();

    // Assert
    assert_eq!(data.get("invocation_id").map(String::as_str), Some("inv:builder"));
    assert_eq!(data.len(), 4);
});

test!(scoped_metadata_raii_expires_on_drop, {
    // Arrange: a plain fixture; metadata scoped to the "act" phase.
    let mut fixture = TestFixture::<()>::new()
        .map_err(|e| Refusal::ValidationFailed(e.to_string()))?;

    {
        // Act: metadata alive only inside this scope.
        let _scope = fixture.with_scoped_metadata("phase", "act");
    }

    // Assert: RAII removed the key at scope exit.
    assert_eq!(fixture.get_metadata("phase"), None);
    Ok::<(), Refusal>(())
});
