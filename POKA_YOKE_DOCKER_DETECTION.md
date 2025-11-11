# Poka-Yoke Analysis: Docker Detection Error Modes

## Step 1: Identify Error Modes

### Error Modes Inventory

#### Invalid State
- [ ] **Container can exist without Docker check** - Runtime check happens, but type doesn't enforce it
- [ ] **Container can be used after Docker stops** - No type-level guarantee Docker is still available
- [ ] **Exit code can be out of i32 range** - i64 to i32 conversion can fail (currently handled, but could be type-level)

#### Invalid Input
- [ ] **Empty image/tag strings** - Can pass empty strings, causes runtime error
- [ ] **Invalid port numbers** - Port 0 or >65535 possible (u16 prevents >65535, but 0 is valid u16)
- [ ] **Empty command/args** - Can pass empty command string

#### Invalid Operations
- [ ] **Calling exec() on stopped container** - No type-level guarantee container is running
- [ ] **Calling get_host_port() before port mapping** - No type-level guarantee port is mapped
- [ ] **Using container after Docker daemon stops** - No type-level tracking of Docker state

#### Resource Errors
- [ ] **Docker unavailable** - Currently runtime check, could be type-level
- [ ] **Container not found** - Runtime error, no type prevention

#### Logic Errors
- [ ] **Exit code conversion (i64 -> i32)** - Currently runtime check with try_into
- [ ] **Port number validation** - Port 0 is technically valid u16 but invalid for containers

## Step 2: Design Type-Level Prevention

### Proposed Type-Level Improvements

#### 1. Docker Availability State Machine
```rust
// Type-level state: DockerChecked vs DockerUnchecked
struct DockerChecked;
struct DockerUnchecked;

struct ContainerClient<State = DockerUnchecked> {
    _state: PhantomData<State>,
}

impl ContainerClient<DockerUnchecked> {
    fn new() -> Self {
        Self { _state: PhantomData }
    }
    
    fn check_docker(self) -> Result<ContainerClient<DockerChecked>, TestcontainersError> {
        check_docker_available()?;
        Ok(ContainerClient { _state: PhantomData })
    }
}

impl ContainerClient<DockerChecked> {
    fn create_container(&self, image: &str, tag: &str) -> TestcontainersResult<GenericContainer> {
        // Can only create container if Docker was checked
        GenericContainer::new(self, image, tag)
    }
}
```

**Poka-yoke**: Cannot create container without checking Docker first - compiler error!

#### 2. Non-Empty String Types
```rust
/// Non-empty string - prevents empty input errors
#[derive(Debug, Clone)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(s: String) -> Result<Self, TestcontainersError> {
        if s.is_empty() {
            Err(TestcontainersError::InvalidConfig("String cannot be empty".to_string()))
        } else {
            Ok(Self(s))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Usage: GenericContainer::new(client, NonEmptyString::new(image)?, NonEmptyString::new(tag)?)
```

**Poka-yoke**: Cannot pass empty string - compile-time guarantee!

#### 3. Valid Port Number Type
```rust
/// Valid container port (1-65535, excludes 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerPort(u16);

impl ContainerPort {
    pub fn new(port: u16) -> Result<Self, TestcontainersError> {
        if port == 0 {
            Err(TestcontainersError::InvalidConfig("Port cannot be 0".to_string()))
        } else {
            Ok(Self(port))
        }
    }
    
    pub fn value(&self) -> u16 {
        self.0
    }
}
```

**Poka-yoke**: Cannot use port 0 - type prevents it!

#### 4. Container State Machine
```rust
// Container states: Running vs Stopped
struct Running;
struct Stopped;

struct GenericContainer<State = Running> {
    container: Container<GenericImage>,
    _state: PhantomData<State>,
}

impl GenericContainer<Running> {
    fn exec(&self, command: &str, args: &[&str]) -> TestcontainersResult<ExecResult> {
        // Can only exec when running
    }
    
    fn stop(self) -> GenericContainer<Stopped> {
        // Consumes Running, returns Stopped
    }
}

impl GenericContainer<Stopped> {
    // Cannot exec - compiler error!
}
```

**Poka-yoke**: Cannot exec on stopped container - compiler error!

#### 5. Exit Code Type
```rust
/// Valid exit code (fits in i32 range)
#[derive(Debug, Clone, Copy)]
pub struct ExitCode(i32);

impl ExitCode {
    pub fn from_i64(code: i64) -> Result<Self, TestcontainersError> {
        code.try_into()
            .map(Self)
            .map_err(|_| TestcontainersError::ExitCodeFailed("Exit code out of range".to_string()))
    }
    
    pub fn value(&self) -> i32 {
        self.0
    }
}
```

**Poka-yoke**: Exit code conversion validated at construction - type guarantees valid range!

## Step 3: Current State Analysis

### What's Already Good (Type-Level Prevention)

✅ **Result<T, E> types** - Forces error handling, prevents unwrap() in production
✅ **u16 for ports** - Prevents >65535 ports (but allows 0)
✅ **Option<T> for nullable values** - Forces handling of None
✅ **Feature gates** - Compile-time feature detection

### What Could Be Improved

❌ **Runtime Docker checks** - Could be type-level state machine
❌ **Empty string validation** - Runtime check, could be NonEmptyString type
❌ **Port 0 validation** - Runtime check, could be ContainerPort type
❌ **Container state** - Runtime, could be type-level state machine
❌ **Exit code conversion** - Runtime try_into, could be ExitCode type

## Step 4: Priority Assessment

### High Priority (High Impact, Low Effort)

1. **NonEmptyString type** - Prevents empty image/tag errors
2. **ContainerPort type** - Prevents port 0 errors
3. **ExitCode type** - Prevents conversion errors

### Medium Priority (High Impact, Medium Effort)

4. **Container state machine** - Prevents exec on stopped container
5. **Docker availability state** - Prevents container creation without Docker check

### Low Priority (Nice to Have)

6. **Command/args validation** - Could use NonEmptyString for commands

## Step 5: Implementation Recommendations

### Immediate Improvements (80/20)

1. Add `NonEmptyString` type for image/tag validation
2. Add `ContainerPort` type for port validation  
3. Add `ExitCode` type for exit code conversion

### Future Enhancements

4. Add container state machine (Running/Stopped)
5. Add Docker availability state machine (Checked/Unchecked)

## Conclusion

**Current State**: Good error handling with Result types, but some validation is runtime.

**Recommended**: Add type-level validation for:
- Non-empty strings (image/tag)
- Valid port numbers (exclude 0)
- Valid exit codes (i32 range)

**Poka-yoke Principle**: "Make invalid states unrepresentable" - Use types to prevent errors at compile time!

