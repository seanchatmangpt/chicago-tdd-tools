//! Poka-Yoke Types for Testcontainers
//!
//! Provides type-level error prevention for container operations.
//! Uses Rust's type system to make invalid container states unrepresentable.
//!
//! **Poka-Yoke Principles**:
//! - **Make invalid states unrepresentable**: Use types to prevent errors
//! - **Type-level prevention**: Invalid container operations cannot be called
//! - **State machine**: Container lifecycle enforced by types
//! - **Docker availability**: Checked at compile-time where possible, runtime where necessary
//!
//! # Error Modes Prevented
//!
//! 1. **Exec on stopped container**: Cannot execute commands on stopped containers
//! 2. **Port mapping on stopped container**: Cannot get ports from stopped containers
//! 3. **Double start**: Cannot start already-started container
//! 4. **Docker unavailable**: Checked at client creation
//! 5. **Invalid configuration**: Validated at container creation

#[cfg(feature = "testcontainers")]
use std::marker::PhantomData;

#[cfg(feature = "testcontainers")]
/// Container state marker types
///
/// **Poka-yoke**: Phantom types prevent invalid operations at compile time.
/// A container is either `Stopped` or `Running` - cannot be both.
pub mod state {
    /// Container is stopped (initial state)
    pub struct Stopped;

    /// Container is running (can execute commands)
    pub struct Running;
}

#[cfg(feature = "testcontainers")]
/// Container with type-level state
///
/// **Poka-yoke**: Type parameter `S` prevents invalid operations.
/// - `Container<Stopped>`: Can only start, cannot exec
/// - `Container<Running>`: Can exec, can stop
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::testcontainers::poka_yoke::*;
///
/// // Create stopped container
/// let container: Container<Stopped> = Container::new(...)?;
///
/// // Start container (changes type to Running)
/// let container: Container<Running> = container.start()?;
///
/// // Can exec on running container
/// let result = container.exec("echo", &["hello"])?;
///
/// // Stop container (changes type back to Stopped)
/// let container: Container<Stopped> = container.stop()?;
///
/// // Compile error: Cannot exec on stopped container
/// // container.exec("echo", &["hello"])?; // ERROR!
/// ```
pub struct Container<S> {
    /// Container ID (internal)
    id: String,
    /// Client reference (internal)
    client: crate::testcontainers::ContainerClient,
    /// State marker (compile-time guarantee)
    _state: PhantomData<S>,
}

#[cfg(feature = "testcontainers")]
impl Container<state::Stopped> {
    /// Create a new stopped container
    ///
    /// **Poka-yoke**: Returns `Container<Stopped>` - cannot exec until started.
    ///
    /// # Errors
    ///
    /// This placeholder always returns `Ok` in the design demonstration.
    ///
    /// # Note
    ///
    /// This is a placeholder for poka-yoke design demonstration.
    #[allow(clippy::unnecessary_wraps)] // Placeholder - will be implemented later
    pub fn new(
        client: crate::testcontainers::ContainerClient,
        image: &str,
        tag: &str,
    ) -> crate::testcontainers::TestcontainersResult<Self> {
        // Docker availability already checked in ContainerClient::new()
        // Container creation happens here
        // For now, return placeholder - actual implementation would create container
        Ok(Self { id: format!("{image}:{tag}"), client, _state: PhantomData })
    }

    /// Start the container
    ///
    /// **Poka-yoke**: Changes type from `Container<Stopped>` to `Container<Running>`.
    /// After this call, container can execute commands.
    ///
    /// # Errors
    ///
    /// This placeholder always returns `Ok` in the design demonstration.
    ///
    /// # Note
    ///
    /// This is a placeholder for poka-yoke design demonstration.
    #[allow(clippy::unnecessary_wraps)] // Placeholder - will be implemented later
    pub fn start(self) -> crate::testcontainers::TestcontainersResult<Container<state::Running>> {
        // Start container logic here
        Ok(Container { id: self.id, client: self.client, _state: PhantomData })
    }
}

#[cfg(feature = "testcontainers")]
impl Container<state::Running> {
    /// Execute a command in the running container
    ///
    /// **Poka-yoke**: Only available on `Container<Running>`.
    /// Compiler prevents calling this on stopped containers.
    ///
    /// # Errors
    ///
    /// Returns error if command execution fails or container stops unexpectedly.
    ///
    /// # Note
    ///
    /// This is a placeholder for poka-yoke design demonstration.
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)] // Placeholder - will be implemented later
    pub fn exec(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> crate::testcontainers::TestcontainersResult<crate::testcontainers::exec::ExecResult> {
        // Exec logic here - only works on running containers
        // This is a placeholder - actual implementation would execute command
        Err(crate::testcontainers::TestcontainersError::OperationFailed(
            "Not implemented - placeholder for poka-yoke design".to_string(),
        ))
    }

    /// Get host port for container port
    ///
    /// **Poka-yoke**: Only available on `Container<Running>`.
    /// Compiler prevents calling this on stopped containers.
    ///
    /// # Errors
    ///
    /// Returns error if port mapping fails.
    ///
    /// # Note
    ///
    /// This is a placeholder for poka-yoke design demonstration.
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)] // Placeholder - will be implemented later
    pub fn get_host_port(
        &self,
        _container_port: u16,
    ) -> crate::testcontainers::TestcontainersResult<u16> {
        // Port mapping logic here - only works on running containers
        Err(crate::testcontainers::TestcontainersError::OperationFailed(
            "Not implemented - placeholder for poka-yoke design".to_string(),
        ))
    }

    /// Stop the container
    ///
    /// **Poka-yoke**: Changes type from `Container<Running>` to `Container<Stopped>`.
    /// After this call, container cannot execute commands.
    ///
    /// # Errors
    ///
    /// This placeholder always returns `Ok` in the design demonstration.
    ///
    /// # Note
    ///
    /// This is a placeholder for poka-yoke design demonstration.
    #[allow(clippy::unnecessary_wraps)] // Placeholder - will be implemented later
    pub fn stop(self) -> crate::testcontainers::TestcontainersResult<Container<state::Stopped>> {
        // Stop container logic here
        Ok(Container { id: self.id, client: self.client, _state: PhantomData })
    }
}

#[cfg(feature = "testcontainers")]
/// Validated container configuration
///
/// **Poka-yoke**: Newtype prevents invalid configurations.
/// The type system makes invalid container configs impossible.
#[derive(Debug, Clone)]
pub struct ValidContainerConfig {
    /// Image name (always non-empty)
    image: String,
    /// Image tag (always non-empty)
    tag: String,
    /// Command (optional) - currently unused, reserved for future use
    _command: Option<String>,
    /// Command args (optional) - currently unused, reserved for future use
    _args: Option<Vec<String>>,
}

#[cfg(feature = "testcontainers")]
impl ValidContainerConfig {
    /// Create a new validated container configuration
    ///
    /// **Poka-yoke**: Returns `Option` to prevent invalid configs:
    /// - Empty image: Returns `None`
    /// - Empty tag: Returns `None`
    ///
    /// # Errors
    ///
    /// Returns `None` if image or tag is empty.
    #[must_use]
    pub fn new(image: &str, tag: &str) -> Option<Self> {
        if image.trim().is_empty() || tag.trim().is_empty() {
            return None;
        }

        Some(Self { image: image.to_string(), tag: tag.to_string(), _command: None, _args: None })
    }

    /// Get image name
    #[must_use]
    pub fn image(&self) -> &str {
        &self.image
    }

    /// Get image tag
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }
}

#[cfg(all(test, feature = "testcontainers"))]
mod tests {
    use super::*;

    #[test]
    fn test_valid_container_config() {
        let config = ValidContainerConfig::new("alpine", "latest");
        assert!(config.is_some());
        if let Some(c) = config {
            assert_eq!(c.image(), "alpine");
            assert_eq!(c.tag(), "latest");
        }
    }

    #[test]
    fn test_invalid_container_config_empty_image() {
        let config = ValidContainerConfig::new("", "latest");
        assert!(config.is_none()); // Type prevents empty image
    }

    #[test]
    fn test_invalid_container_config_empty_tag() {
        let config = ValidContainerConfig::new("alpine", "");
        assert!(config.is_none()); // Type prevents empty tag
    }
}
