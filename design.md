# **Complete compiler design: master specification and production roadmap**

**Source language and frontend**

Lexing · Parsing · Resolution · Types · Ownership · Effects

**IR 1: Effect-typed Sea of Nodes**

Language semantics · Control · Data · Memory · Effects

**IR 2: Heterogeneous Sea of Nodes**

Representation selection · Specialization · Allocation elimination · Optimization

**IR 3: Machine IR**

Instruction selection · Registers · Scheduling · Target-specific lowering

**Object generation and executable**

Assembler · Relocations · Object files · Linker integration

There will also be temporary representations and analysis structures inside these stages. For example, the frontend needs control-flow information for borrow checking. That does not mean we need another permanent optimizing IR.

## **How to read this design**

Every item below is something that needs an explicit definition, implementation contract, or validation strategy.

I distinguish three types of work:

- **Language specification:** What programs mean.
- **Compiler specification:** How the implementation represents and transforms programs.
- **Production specification:** How the compiler is built, distributed, tested, maintained, and used.

A feature is not considered finished merely because syntax exists for it.

# **Volume I: Language foundation**

## **1. Language goals and implementation contract**

Before defining more language features, establish the product contract.

These are the questions the entire project must answer.

### **1.1. Language identity**

- Language name and versioning.
- Language design principles.
- Intended programming domains.
- Supported programming paradigms.
- Primary implementation goals.
- Non-goals.
- Compatibility philosophy.
- Relationship between language and standard library.
- Relationship between language and compiler implementation.

### **1.2. Performance contract**

- Compilation-time objectives.
- Runtime performance objectives.
- Memory consumption objectives.
- Code-size objectives.
- Startup-time objectives.
- Allocation behavior.
- Zero-cost abstraction definition.
- Runtime overhead policy.
- Performance predictability.
- Compile-time versus runtime computation.
- Optimization guarantees versus optimization hints.

Do not promise that every abstraction is free. Define exactly what costs the language guarantees and what costs depend on optimization.

### **1.3. Safety contract**

- Memory safety.
- Type safety.
- Initialization safety.
- Ownership safety.
- Borrowing safety.
- Data-race freedom.
- Pointer validity.
- Integer overflow behavior.
- Undefined behavior categories.
- Unsafe-code responsibilities.
- FFI safety boundaries.
- Compiler trust boundaries.

### **1.4. Platform contract**

- Supported architectures.
- Supported operating systems.
- Supported object formats.
- Supported executable formats.
- Supported ABIs.
- Cross-compilation.
- Endianness.
- Pointer widths.
- Atomic capabilities.
- SIMD capabilities.
- Operating-system interfaces.
- Hosted and freestanding environments.

### **1.5. Compatibility policy**

- Source compatibility.
- Binary compatibility.
- Standard library compatibility.
- Compiler version compatibility.
- Package compatibility.
- ABI stability.
- Deprecation rules.
- Edition changes.
- Language feature stabilization.
- Breaking-change policy.

**Deliverable:** Language Charter and Implementation Requirements Specification.

# **Volume II: Lexical and syntactic specification**

## **2. Lexical structure**

Your existing lexical draft is a starting point. It needs a complete normative definition.

### **2.1. Source files**

- Source encoding.
- Unicode version.
- Byte-order mark handling.
- Line endings.
- Invalid encoding behavior.
- Source locations.
- Column numbering.
- Tabs and indentation.
- File inclusion rules.

### **2.2. Tokens**

- Identifier grammar.
- Unicode identifier rules.
- Identifier normalization.
- Reserved keywords.
- Future-reserved keywords.
- Context-sensitive restrictions, if any.
- Integer literals.
- Floating-point literals.
- Literal suffixes.
- Character literals.
- String literals.
- Byte strings.
- Raw strings.
- Escape sequences.
- Comments.
- Documentation comments.
- Attributes.
- Punctuation.
- Operators.
- Delimiters.
- Tokenization precedence.

### **2.3. Literal semantics**

- Integer literal interpretation.
- Arbitrary-precision literal handling during type checking.
- Integer range checking.
- Floating-point rounding.
- NaN and infinity representation.
- Character validity.
- Unicode scalar values.
- UTF-8 validation.
- Byte literal ranges.
- Compile-time literal evaluation.
- Invalid literal diagnostics.

### **2.4. Operator specification**

For every operator:

- Syntax.
- Precedence.
- Associativity.
- Operand types.
- Result type.
- Overloadability.
- Evaluation order.
- Overflow semantics.
- Panic behavior.
- Effects.
- Constant evaluation.
- IR lowering.

### **2.5. Grammar**

Define a complete grammar for:

- Compilation units.
- Modules.
- Declarations.
- Generic parameters.
- Type expressions.
- Function declarations.
- Structs.
- Enums.
- Unions.
- Traits.
- Implementations.
- Attributes.
- Statements.
- Expressions.
- Patterns.
- Closures.
- Async blocks.
- Unsafe blocks.
- Inline assembly.
- Macros, if supported.
- Error recovery.

### **2.6. Parsing behavior**

- Ambiguity resolution.
- Generic argument parsing.
- Type versus expression disambiguation.
- Operator precedence parsing.
- Trailing commas.
- Semicolon rules.
- Block expressions.
- Parser recovery.
- Malformed input handling.
- Error synchronization.
- Incremental parsing.

**Deliverables:** Lexical Specification, Grammar Specification, Operator Specification, Parser Contract.

# **Volume III: Names, modules, and declarations**

## **3. Name resolution**

This deserves its own specification. It cannot just be a side effect of parsing.

Define:

- Identifier namespaces.
- Lexical scopes.
- Module scopes.
- Type namespaces.
- Value namespaces.
- Trait namespaces.
- Macro namespaces, if applicable.
- Generic parameter scopes.
- Shadowing.
- Name visibility.
- Name imports.
- Wildcard imports.
- Re-exports.
- Glob ambiguity.
- Qualified paths.
- Relative paths.
- Absolute paths.
- `self`, `Self`, `super`, and `crate`.
- Name hygiene.
- Unresolved names.
- Ambiguous names.
- Recursive declarations.
- Forward declarations.

## **4. Module and package system**

Define the complete compilation unit model.

- Source modules.
- Nested modules.
- External modules.
- Packages or crates.
- Package roots.
- Module ownership.
- Public exports.
- Private declarations.
- Dependency graphs.
- Circular dependency handling.
- Compilation order.
- Separate compilation.
- Module initialization.
- Package versioning.
- Conditional modules.
- Platform-specific modules.
- Module-level generic declarations.
- Interface metadata.
- Compilation cache boundaries.

## **5. Declaration semantics**

Specify:

- Variables.
- Constants.
- Static variables.
- Functions.
- Generic functions.
- Structs.
- Tuple structs.
- Unit structs.
- Enums.
- Unions.
- Traits.
- Implementations.
- Type aliases.
- Associated constants.
- Associated types.
- Foreign declarations.
- External statics.
- Methods.
- Inherent implementations.
- Trait implementations.
- Visibility.
- Declaration conflicts.
- Duplicate definitions.
- Recursive types.
- Incomplete declarations.

**Deliverables:** Name Resolution Specification, Module System Specification, Declaration Semantics Specification.

# **Volume IV: Complete type system**

## **6. Type system specification**

This will likely be one of the largest documents in the entire project.

### **6.1. Type categories**

Define the exact semantics of:

- Integer types.
- Floating-point types.
- Boolean.
- Character.
- Unit.
- Never.
- Tuples.
- Arrays.
- Slices.
- Structs.
- Enums.
- Unions.
- References.
- Raw pointers.
- Function pointers.
- Closures.
- Trait objects.
- Opaque types.
- Generic types.
- Dynamically sized types.
- SIMD types.
- Foreign types.
- Zero-sized types.

### **6.2. Type identity**

- Nominal type identity.
- Structural type identity.
- Type aliases.
- Newtype identity.
- Generic type identity.
- Instantiated type identity.
- Associated type identity.
- Opaque type identity.
- Function type identity.
- Closure type identity.
- Dynamic type identity.

### **6.3. Type compatibility**

- Exact type equality.
- Type unification.
- Coercion.
- Implicit conversions.
- Explicit conversions.
- Numeric conversions.
- Pointer conversions.
- Reference conversions.
- Array-to-slice conversions.
- Function coercions.
- Trait object coercions.
- Unsizing.
- Subtyping, if supported.
- Never-type coercion.
- Lifetime compatibility.

### **6.4. Type inference**

Define:

- Local variable inference.
- Numeric literal inference.
- Function argument inference.
- Return type inference.
- Generic inference.
- Closure parameter inference.
- Closure return inference.
- Trait constraint inference.
- Associated type inference.
- Lifetime inference.
- Effect inference.
- Const generic inference.
- Ambiguity handling.
- Inference failure diagnostics.
- Explicit type constraints.

### **6.5. Generic system**

- Type parameters.
- Const parameters.
- Lifetime parameters.
- Effect parameters, if supported.
- Generic bounds.
- Multiple bounds.
- Associated types.
- Associated constants.
- Generic defaults.
- Parameter ordering.
- Generic inference.
- Instantiation identity.
- Recursive generic instantiation.
- Monomorphization.
- Partial monomorphization.
- Erased polymorphism.
- Hybrid specialization.
- Specialization legality.
- Specialization coherence.
- Code-size control.
- Instantiation caching.

### **6.6. Trait system**

- Trait declarations.
- Required methods.
- Default methods.
- Associated types.
- Associated constants.
- Generic traits.
- Supertraits.
- Trait implementation.
- Blanket implementations.
- Orphan rules.
- Coherence.
- Overlap detection.
- Method resolution.
- Trait selection.
- Recursive trait solving.
- Auto traits.
- Unsafe traits.
- Negative implementations.
- Object safety.
- Dynamic dispatch.
- Trait upcasting.
- Vtable semantics.
- Trait implementation visibility.

### **6.7. Capabilities**

Define each capability precisely:

- Copy.
- Clone.
- Drop.
- Send.
- Sync.
- Sized.
- Unpin.
- Freeze.
- Any additional ownership or representation capabilities.

For every capability:

- Semantic meaning.
- Automatic derivation conditions.
- Explicit implementation permissions.
- Generic constraint behavior.
- Interaction with other capabilities.
- Unsafe implementation obligations.
- Compiler enforcement.

### **6.8. Variance**

- Covariance.
- Contravariance.
- Invariance.
- Lifetime variance.
- Function variance.
- Mutable-reference variance.
- Generic container variance.
- Trait object variance.
- Inference.
- Variance validation.

### **6.9. Dynamic types**

- Trait object construction.
- Trait object layout.
- Vtables.
- Associated type bindings.
- Dynamic method dispatch.
- Fat pointers.
- Trait object ownership.
- Trait object lifetimes.
- Auto trait combinations.
- Dynamic conversion.
- Object safety.

### **6.10. Type-level programming**

- Const generics.
- Compile-time type computations.
- Type reflection.
- Layout reflection.
- Type predicates.
- Type-level constants.
- Compile-time trait queries.
- Recursive type computations.
- Evaluation limits.
- Termination requirements.

**Deliverable:** Complete Type System Specification, including typing and constraint rules for every construct.

# **Volume V: Value, memory, and object semantics**

## **7. Value semantics and object identity**

This is the specification you've been developing.

It must contain:

### **7.1. Values and objects**

- Definition of value.
- Definition of object.
- Semantic identity.
- Object identity.
- Storage identity.
- Value equivalence.
- Equality semantics.
- Address observability.
- Address stability.
- Object creation.
- Object destruction.
- Virtual objects.
- Materialized objects.

### **7.2. Initialization**

- Uninitialized storage.
- Initialization.
- Full initialization.
- Partial initialization.
- Reinitialization.
- Initialization through references.
- Initialization through pointers.
- Initialization of arrays.
- Initialization of enums.
- Initialization of unions.
- Initialization of dynamically sized types.
- Initialization failure.
- Initialization tracking.

### **7.3. Copy and move**

- Copy semantics.
- Copy capability.
- Implicit copying.
- Explicit cloning.
- Move semantics.
- Move source invalidation.
- Partial moves.
- Aggregate moves.
- Array element moves.
- Moves through references.
- Moves involving pinned objects.
- Relocation.
- Ownership transfer.

### **7.4. Destruction**

- Destructor invocation.
- Destructor eligibility.
- Drop order.
- Field destruction.
- Partial destruction.
- Destructor effects.
- Destructor panic.
- Unwinding.
- Abort behavior.
- Double panic.
- Drop flags.
- Manual destruction.
- Explicit resource release.
- Deliberate leaking.
- Interaction with FFI.

### **7.5. Layout**

- Size.
- Alignment.
- Field offsets.
- Padding.
- Enum discriminants.
- Niche representations.
- Zero-sized types.
- Aggregate packing.
- Explicit alignment.
- Packed structures.
- Transparent structures.
- C layout.
- SIMD layout.
- Foreign layout.
- Layout reflection.
- Layout compatibility.

### **7.6. Identity and optimization**

- Identity preservation.
- Address escape.
- Pointer identity.
- Pointer comparison.
- Storage reuse.
- Storage lifetime.
- Object replacement.
- Aggregate virtualization.
- Scalar replacement.
- Object merging legality.
- Allocation elimination legality.
- Identity-sensitive operations.

## **8. Memory model**

This must be one of the most precisely written volumes.

### **8.1. Memory regions**

- Stack storage.
- Heap storage.
- Static storage.
- Thread-local storage.
- Foreign storage.
- Memory-mapped storage.
- Allocator-managed storage.
- Temporary storage.
- Storage lifetime.

### **8.2. Pointers**

- Pointer types.
- Pointer formation.
- Null pointers.
- Non-null pointers.
- Pointer provenance.
- Provenance creation.
- Provenance preservation.
- Provenance exposure.
- Pointer arithmetic.
- Pointer subtraction.
- Pointer comparison.
- Pointer-to-integer conversion.
- Integer-to-pointer conversion.
- One-past pointers.
- Dangling pointers.
- Pointer invalidation.
- Pointer equality.
- Pointer ordering.
- Pointer reconstruction.

### **8.3. Memory access**

- Read access.
- Write access.
- Initialization requirements.
- Alignment requirements.
- Access width.
- Valid memory ranges.
- Mutability.
- Aliasing.
- Overlapping accesses.
- Invalid accesses.
- Unaligned access.
- Volatile access.
- Atomic access.
- Foreign memory access.

### **8.4. Aliasing model**

- Reference aliasing.
- Raw pointer aliasing.
- Mutable alias restrictions.
- Shared alias rules.
- Interior mutability.
- Provenance-based alias analysis.
- No-alias guarantees.
- Alias invalidation.
- Unknown alias regions.
- Compiler alias assumptions.

### **8.5. Allocation**

- Allocation APIs.
- Allocation layouts.
- Alignment.
- Allocator identity.
- Allocation failure.
- Allocation limits.
- Deallocation.
- Reallocation.
- Placement construction.
- Zero-sized allocations.
- Allocation lifetime.
- Foreign allocation.
- Allocation ownership.

### **8.6. Undefined behavior**

Create a complete UB catalog.

At minimum, define:

- Reading uninitialized values.
- Invalid references.
- Invalid pointer dereferences.
- Misaligned accesses.
- Out-of-bounds access.
- Invalid enum representations.
- Invalid Boolean representations.
- Use-after-free.
- Double-free.
- Violated aliasing rules.
- Data races.
- Invalid function calls.
- Invalid calling conventions.
- Invalid inline assembly contracts.
- Invalid union access.
- Invalid destruction.
- Invalid atomic operations.
- Invalid provenance operations.

For each UB category, document:

1. The triggering condition.
2. Whether the condition is statically detectable.
3. Whether unsafe code is needed to trigger it.
4. Compiler assumptions permitted by the condition.
5. Expected diagnostic behavior, where applicable.

### **8.7. Memory ordering**

- Ordinary reads and writes.
- Compiler reordering.
- Hardware reordering.
- Atomic access.
- Fences.
- Synchronization.
- Volatile.
- Thread communication.
- Data-race definition.
- Memory consistency guarantees.

**Deliverables:** Value Semantics Specification, Object and Layout Specification, Memory Model Specification, UB Catalog.

# **Volume VI: Ownership and borrow checker**

## **9. Ownership system**

### **9.1. Ownership model**

- Unique ownership.
- Borrowed ownership.
- Shared ownership.
- Static ownership.
- Thread-local ownership.
- Foreign ownership.
- Manually managed ownership.
- Ownership transfer.
- Ownership boundaries.
- Ownership graphs.

### **9.2. Place semantics**

- Place definition.
- Base places.
- Local places.
- Field projections.
- Tuple projections.
- Array projections.
- Index projections.
- Dereference projections.
- Downcasts.
- Dynamic projections.
- Place overlap.
- Place disjointness.
- Unknown projections.

### **9.3. Move analysis**

- Move creation.
- Move legality.
- Source invalidation.
- Destination initialization.
- Partial moves.
- Reinitialization.
- Move through branches.
- Move through loops.
- Move through closures.
- Move through function calls.
- Move through pattern matching.
- Move involving destructors.
- Move involving pinned data.

### **9.4. Borrow semantics**

- Shared references.
- Exclusive references.
- Loan creation.
- Loan uses.
- Loan termination.
- Reborrowing.
- Nested borrowing.
- Borrow splitting.
- Field-sensitive borrowing.
- Array borrowing.
- Slice borrowing.
- Borrowing temporaries.
- Borrowing statics.
- Borrowing foreign data.
- Escaping references.

### **9.5. Lifetime system**

- Lifetime regions.
- Lifetime inference.
- Lifetime annotations.
- Lifetime parameters.
- Lifetime bounds.
- Lifetime subtyping.
- Lifetime variance.
- Region constraints.
- Lifetime containment.
- Lifetime escape.
- Return lifetime relationships.
- Closure lifetime capture.
- Async lifetime capture.
- Lifetime termination.

### **9.6. Borrow checker analysis**

- Control-flow graph construction.
- Initialization dataflow.
- Move dataflow.
- Loan generation.
- Loan liveness.
- Region inference.
- Conflict detection.
- Place overlap analysis.
- Escape analysis.
- Destruction analysis.
- Edge-sensitive loan termination.
- Loop handling.
- Call-boundary handling.
- Recursive function handling.
- Diagnostics.

### **9.7. Borrow checking corner cases**

- Partial initialization.
- Partial moves.
- Conditional initialization.
- Loops with moves.
- Multiple exits.
- Reborrowing.
- Closures.
- Captured references.
- Recursive aggregates.
- Mutable slices.
- Self-referential structures.
- Pinning.
- Async suspension.
- Foreign pointers.
- Interior mutability.

### **9.8. Unsafe integration**

- Unsafe authority.
- Raw-pointer access.
- Manual initialization.
- Manual destruction.
- Manual allocation.
- Manual deallocation.
- Unsafe trait obligations.
- Unsafe function contracts.
- Borrow checker bypass boundaries.
- Safety invariants.
- Unsafe diagnostics and auditing.

### **9.9. Pinning**

- Pin semantics.
- Pinned storage.
- `Unpin`.
- Address stability.
- Pinned references.
- Pinned ownership.
- Pin projection.
- Self-referential objects.
- Async state machines.
- Destruction of pinned values.
- Unsafe pinning contracts.

**Deliverable:** Ownership and Borrow Checker Specification.

This volume needs both semantic rules and an explicit algorithm for enforcing them. Saying "use NLL" is not a complete borrow-checker design.

# **Volume VII: Effects, control flow, and execution**

## **10. Effect system**

### **10.1. Effect categories**

Define the full set of effects:

- Read.
- Write.
- Allocation.
- Deallocation.
- IO.
- Atomic.
- Synchronization.
- Panic.
- Suspension.
- FFI.
- Any additional required external effects.

### **10.2. Effect semantics**

- Observable versus non-observable effects.
- Memory-region effects.
- Effect identities.
- Effect dependencies.
- Effect composition.
- Effect inference.
- Effect propagation.
- Effect constraints.
- Effect subtyping.
- Effect polymorphism, if supported.
- Effect specialization.
- Effect erasure.
- Unknown effect handling.

### **10.3. Function effects**

- Function declarations.
- Function types.
- Closure effect types.
- Generic effect contracts.
- Trait method effects.
- Dynamic dispatch effect contracts.
- Foreign function effects.
- Async latent effects.
- Constructor effects.
- Destructor effects.
- Intrinsic effects.

### **10.4. Optimization semantics**

- Reordering.
- Dead-effect elimination.
- Effect-preserving inlining.
- Call summaries.
- Memory dependency partitioning.
- Effect independence.
- Unknown effect barriers.
- Atomic ordering.
- IO ordering.

## **11. Evaluation and control-flow semantics**

### **11.1. Expression evaluation**

Every expression needs:

- Evaluation order.
- Operand evaluation order.
- Result type.
- Result category.
- Ownership behavior.
- Borrowing behavior.
- Effects.
- Panic conditions.
- Divergence conditions.
- Constant-evaluation rules.

### **11.2. Control flow**

- Blocks.
- If expressions.
- If statements.
- Match expressions.
- Match statements.
- While loops.
- For loops.
- Infinite loops.
- Break.
- Continue.
- Return.
- Yield.
- Early exits.
- Labeled control flow, if supported.
- Unreachable paths.

### **11.3. Pattern matching**

- Literal patterns.
- Identifier patterns.
- Wildcards.
- Reference patterns.
- Tuple patterns.
- Struct patterns.
- Enum patterns.
- Slice patterns.
- Or patterns.
- Binding patterns.
- Guards.
- Move bindings.
- Copy bindings.
- Reference bindings.
- Exhaustiveness.
- Reachability.
- Irrefutability.
- Pattern compilation.

### **11.4. Operator semantics**

- Arithmetic.
- Comparisons.
- Logical operations.
- Bitwise operations.
- Assignment.
- Compound assignment.
- Ranges.
- Casting.
- Dereferencing.
- Borrow operators.
- Operator overloading.
- Short-circuiting.
- Overflow behavior.
- Division-by-zero behavior.
- Floating-point edge cases.

### **11.5. Function-call semantics**

- Argument evaluation.
- Argument ownership transfer.
- Parameter initialization.
- Return-value semantics.
- Stack frame semantics.
- Recursive calls.
- Tail calls.
- Indirect calls.
- Generic calls.
- Dynamic calls.
- Variadic calls.
- Function pointer calls.
- Panic propagation.

### **11.6. Panic and error handling**

- Result values.
- Optional values.
- Propagation.
- Panic.
- Unwind.
- Abort.
- Panic hooks.
- Destructor behavior during unwinding.
- Error conversion.
- Error traits.
- Panic boundaries.
- FFI unwind behavior.

**Deliverables:** Effect System Specification, Operational Semantics Specification, Control-Flow Specification, Error and Panic Specification.

# **Volume VIII: Concurrency and asynchronous execution**

## **12. Concurrency semantics**

### **12.1. Thread model**

- Thread creation.
- Thread termination.
- Thread ownership.
- Thread-local data.
- Thread migration.
- Thread joining.
- Thread detachment.
- Thread lifecycle.
- Stack allocation.
- Thread failure.

### **12.2. Send and Sync**

- Capability definitions.
- Automatic derivation.
- Explicit implementations.
- Generic propagation.
- Unsafe implementations.
- Trait object behavior.
- Foreign type behavior.
- Thread-transfer validation.

### **12.3. Synchronization**

- Mutexes.
- Read-write locks.
- Condition variables.
- Semaphores.
- Barriers.
- Channels.
- Atomics.
- Fences.
- Lock-free structures.
- Wait-free structures.
- Memory ordering.
- Deadlock considerations.

### **12.4. Atomic model**

- Supported atomic widths.
- Atomic types.
- Atomic operations.
- Compare-and-swap.
- Exchange.
- Fetch operations.
- Memory ordering.
- Atomic alignment.
- Target feature requirements.
- Unsupported atomic behavior.
- Compiler lowering.

## **13. Async system**

### **13.1. Future model**

- Future type.
- Future construction.
- Lazy execution.
- Polling.
- Completion.
- Pending state.
- Waking.
- Scheduling.
- Future composition.
- Future destruction.
- Cancellation.

### **13.2. Async lowering**

- State-machine construction.
- Suspension points.
- Captured variables.
- State fields.
- Resume dispatch.
- Drop paths.
- Error paths.
- Panic paths.
- Lifetime storage.
- Pin requirements.

### **13.3. Executors**

- Executor interfaces.
- Single-threaded executors.
- Multi-threaded executors.
- Work stealing.
- Task queues.
- Scheduling policy.
- Wake queues.
- Cancellation propagation.
- Task priority, if supported.
- Executor-independent futures.

### **13.4. Generators and yield**

If `yield` exists, separately define:

- Generator values.
- Generator state.
- Yielded values.
- Resumption.
- Generator destruction.
- Generator borrowing.
- Generator pinning.
- Async-generator interaction.

**Deliverables:** Concurrency Memory Model, Async Semantics Specification, Executor Interface Specification.

# **Volume IX: Compile-time execution and language extensibility**

## **14. Compile-time programming**

### **14.1. Constant evaluation**

- Constant expressions.
- `const` declarations.
- `const fn`.
- Evaluation order.
- Permitted operations.
- Forbidden operations.
- Pointer restrictions.
- Allocation restrictions.
- Effects during evaluation.
- Recursion.
- Termination.
- Evaluation resource limits.
- Diagnostics.
- Constant caching.

### **14.2. Static initialization**

- Static values.
- Constant initialization.
- Runtime initialization, if supported.
- Initialization order.
- Dependency cycles.
- Initialization failure.
- Destructor behavior.
- Thread-local initialization.

### **14.3. Reflection**

- Type reflection.
- Field reflection.
- Method reflection.
- Trait reflection.
- Layout reflection.
- Generic reflection.
- Compile-time queries.
- Visibility restrictions.
- Reflection stability.
- Runtime metadata policy.

### **14.4. Generated code**

- Compile-time code generation.
- Generated declarations.
- Type generation.
- Implementation generation.
- Syntax tree construction.
- Generated source locations.
- Diagnostics.
- Hygiene.
- Generated-code safety.
- Determinism.

### **14.5. Macros**

You need to explicitly choose which macro systems exist.

Possible categories:

- Declarative macros.
- Procedural macros.
- Attribute macros.
- Derive macros.
- Built-in compiler macros.

For each:

- Invocation syntax.
- Expansion timing.
- Expansion inputs.
- Expansion outputs.
- Hygiene.
- Name resolution.
- Recursion.
- Error diagnostics.
- Build reproducibility.
- Sandbox restrictions.
- Dependency handling.
- Compiler-version compatibility.

### **14.6. Compile-time dependencies**

- Host execution.
- Target execution restrictions.
- Host versus target types.
- Cross-compilation.
- Build scripts.
- Compile-time library dependencies.
- Caching.
- Sandboxing.
- Deterministic evaluation.

**Deliverables:** Constant Evaluation Specification, Reflection Specification, Macro System Specification, Compile-time Execution Contract.

# **Volume X: Attributes, intrinsics, and low-level interfaces**

## **15. Attributes and compiler directives**

Define every supported attribute and its semantics.

### **15.1. Layout attributes**

- `repr(C)`.
- `repr(transparent)`.
- `repr(packed)`.
- `repr(align(N))`.
- `repr(simd)`.
- Enum representation.
- Discriminant selection.

### **15.2. Optimization attributes**

- Inline hints.
- No-inline.
- Cold and hot hints.
- Branch prediction hints.
- Optimization level hints.
- Target features.
- Link-time optimization controls.
- Code-generation hints.

Make clear which attributes are guarantees and which are advisory.

### **15.3. Semantic attributes**

- Derive.
- Must-use.
- Deprecation.
- Diagnostics.
- Conditional compilation.
- Testing.
- Foreign linkage.
- Export control.
- Initialization.
- Destruction.
- Safety requirements.

### **15.4. Attribute infrastructure**

- Attribute grammar.
- Attribute argument types.
- Attribute validation.
- Attribute expansion.
- Attribute conflicts.
- Unknown attributes.
- Stability.
- Built-in versus library-defined attributes.

## **16. Intrinsics and inline assembly**

### **16.1. Compiler intrinsics**

- Intrinsic namespace.
- Type requirements.
- Target requirements.
- Semantic contracts.
- Constant evaluation.
- Effect classification.
- Unsafe requirements.
- Lowering.
- Fallback implementations.

### **16.2. Inline assembly**

Define:

- Assembly syntax.
- Template substitutions.
- Input operands.
- Output operands.
- In/out operands.
- Register classes.
- Fixed registers.
- Clobber lists.
- Condition codes.
- Memory effects.
- Stack effects.
- Control-flow effects.
- Volatility.
- Unwinding.
- Debug information.
- Target-specific constraints.
- Compiler optimization barriers.

### **16.3. SIMD**

- Vector types.
- Vector lane types.
- Vector widths.
- Masks.
- Arithmetic.
- Comparisons.
- Shuffles.
- Reductions.
- Memory loads.
- Memory stores.
- Alignment.
- Masked operations.
- Floating-point semantics.
- Saturating operations.
- Target feature detection.
- Portable fallback behavior.

## **17. FFI and ABI**

### **17.1. ABI specification**

- Calling conventions.
- Argument classification.
- Return-value classification.
- Register usage.
- Stack layout.
- Alignment.
- Structure passing.
- Aggregate return.
- Variadic arguments.
- Unwinding.
- Thread-local ABI.
- Symbol naming.
- Export visibility.

### **17.2. Foreign data**

- C-compatible integers.
- Foreign floating-point types.
- Opaque types.
- C-compatible structs.
- C-compatible unions.
- Enum interoperability.
- Function pointers.
- Variadic types.
- Foreign allocation.
- Foreign ownership.

### **17.3. FFI safety**

- Imported function safety.
- Exported function safety.
- Pointer validity.
- Ownership crossing the boundary.
- Lifetime crossing.
- Callback lifetime.
- Callback threading.
- Panic crossing.
- Exception interoperability.
- Foreign allocator pairing.
- String interoperability.
- ABI mismatch detection.

### **17.4. Linkage**

- Static linkage.
- Dynamic linkage.
- Weak symbols.
- Symbol visibility.
- Symbol versioning.
- Import libraries.
- Export libraries.
- Foreign libraries.
- Runtime library dependencies.

**Deliverables:** Attribute Specification, Intrinsic Registry, Assembly Contract, ABI Specification, FFI Safety Specification.

# **Volume XI: Frontend architecture**

Now we move from defining the language to defining the actual compiler that accepts it.

## **18. Frontend architecture**

The frontend must transform source text into a semantically validated program.

It needs the following subsystems.

### **18.1. Compiler driver**

- Command-line interface.
- Input file handling.
- Multiple input files.
- Compilation modes.
- Target selection.
- Optimization level.
- Debug information options.
- Output selection.
- Diagnostic configuration.
- Feature selection.
- Build configuration.
- Compiler configuration files.
- Environment variables.
- Exit codes.
- Cancellation.
- Compiler invocation protocol.

### **18.2. Source management**

- Source file storage.
- Source identifiers.
- Source locations.
- Source spans.
- Source maps.
- Included files.
- Generated source.
- Macro expansion locations.
- File caching.
- Incremental invalidation.

### **18.3. Lexer**

- Token generation.
- Token positions.
- Literal decoding.
- Keyword recognition.
- Error tokens.
- Lexical diagnostics.
- Incremental lexing.
- Malformed input recovery.

### **18.4. Parser**

- Grammar implementation.
- AST construction.
- Operator precedence.
- Generic parsing.
- Error recovery.
- Partial AST construction.
- Parser diagnostics.
- Incremental parsing.
- Syntax tree ownership.
- AST validation.

### **18.5. AST**

Define schemas for:

- Modules.
- Declarations.
- Types.
- Expressions.
- Statements.
- Patterns.
- Attributes.
- Generic parameters.
- Effect clauses.
- Function bodies.
- Error nodes.

Also define:

- Node IDs.
- Source locations.
- Parent-child relationships.
- Node ownership.
- AST mutation rules.
- AST traversal interfaces.
- Serialization, if required.

### **18.6. Desugaring**

Decide which language constructs lower into simpler constructs.

Examples:

- Compound assignments.
- For loops.
- Method calls.
- Async functions.
- Closures.
- Error propagation.
- Pattern matching.
- Range expressions.
- Derive-generated implementations.
- Other syntactic sugar.

For every desugaring:

- Source construct.
- Lowered form.
- Semantic preservation rules.
- Ownership preservation.
- Effect preservation.
- Diagnostic source mapping.

Desugaring must not erase information needed by ownership checking or diagnostics.

### **18.7. Name resolution**

- Scope creation.
- Declaration collection.
- Import resolution.
- Visibility validation.
- Generic resolution.
- Method lookup.
- Trait lookup.
- Associated item lookup.
- Module dependency resolution.
- Ambiguity reporting.

### **18.8. Type checking**

- Expression type checking.
- Declaration type checking.
- Function checking.
- Generic checking.
- Trait constraint collection.
- Type inference.
- Coercion checking.
- Pattern type checking.
- Return-type verification.
- Closure type inference.
- Trait object validation.
- Type normalization.
- Type error recovery.

### **18.9. Trait solver**

This should be a distinct subsystem.

Define:

- Goal representation.
- Constraint generation.
- Implementation candidate lookup.
- Candidate selection.
- Generic substitution.
- Associated type normalization.
- Recursive goals.
- Cyclic goals.
- Ambiguous goals.
- Overflow handling.
- Coherence validation.
- Specialization selection.
- Caching.
- Incremental invalidation.

Do not bury the entire trait solver inside type checking. It has sufficiently complex semantics to justify its own design.

### **18.10. Constant evaluator**

- Constant expression evaluation.
- Compile-time function execution.
- Generic constant evaluation.
- Static initializer evaluation.
- Type-level computation.
- Resource limits.
- Evaluation caching.
- Error reporting.
- Cross-target evaluation.

### **18.11. Exhaustiveness analysis**

- Match exhaustiveness.
- Unreachable patterns.
- Variant analysis.
- Integer range coverage.
- Guard handling.
- Nested patterns.
- Slice patterns.
- Or patterns.
- Generic enum patterns.
- Diagnostic suggestions.

### **18.12. Frontend control-flow analysis**

Before borrow checking, construct a temporary control-flow representation.

Define:

- Basic blocks.
- Edges.
- Branch conditions.
- Loop structure.
- Returns.
- Breaks.
- Continues.
- Panic exits.
- Unreachable blocks.
- Exceptional edges.
- Await suspension points.
- Yield suspension points.
- Terminators.
- Dominance information.

This representation is an analysis structure. It does not replace your three principal optimizing IRs.

### **18.13. Ownership and effect validation**

- Initialization dataflow.
- Move analysis.
- Borrow checking.
- Lifetime inference.
- Loan conflicts.
- Drop analysis.
- Effect inference.
- Effect contract validation.
- Unsafe authority validation.
- Async borrowing validation.
- Send and Sync checking.

### **18.14. Typed semantic boundary**

This is the point where the frontend must produce a validated semantic program.

Define a formal handoff containing:

- Resolved names.
- Fully determined semantic types.
- Generic constraints.
- Ownership facts.
- Lifetime facts.
- Initialization facts.
- Effect summaries.
- Control-flow facts.
- Destructor information.
- Unsafe requirements.
- ABI requirements.
- Source mappings.

**Deliverable:** Frontend Architecture Specification and Frontend Semantic Handoff Contract.

# **Volume XII: Compiler-wide infrastructure**

## **19. Core compiler infrastructure**

These systems are often overlooked until they become painful to retrofit.

### **19.1. Type representation infrastructure**

- Type interning.
- Type IDs.
- Type canonicalization.
- Generic substitution.
- Type normalization.
- Type equality.
- Type hashing.
- Recursive type handling.
- Lifetime representation.
- Effect representation.
- Type serialization.

### **19.2. Symbol infrastructure**

- Symbol IDs.
- Symbol tables.
- Mangling.
- Demangling.
- Symbol visibility.
- Linkage names.
- Generic instance symbols.
- Incremental symbol identity.

### **19.3. Graph infrastructure**

Required by both Sea of Nodes implementations.

- Node IDs.
- Edge representation.
- Input slots.
- Output values.
- Use lists.
- Def-use relationships.
- Graph ownership.
- Graph mutation.
- Node replacement.
- Node deletion.
- Graph cloning.
- Graph traversal.
- Graph serialization.
- Graph verification.
- Graph visualization.
- Graph hashing.

### **19.4. Memory infrastructure**

- Arena allocation for compiler data structures.
- Interning storage.
- Temporary allocation.
- Memory reclamation.
- Compilation-unit ownership.
- Parallel-safe allocation.
- Memory profiling.
- Peak-memory tracking.

### **19.5. Diagnostics infrastructure**

- Diagnostic IDs.
- Severity.
- Source spans.
- Secondary labels.
- Notes.
- Suggestions.
- Fix-its.
- Diagnostic deduplication.
- Diagnostic ordering.
- Macro expansion tracebacks.
- Internal compiler errors.
- Machine-readable diagnostic output.
- Localization, if required.

### **19.6. Pass infrastructure**

- Pass registration.
- Pass dependencies.
- Pass ordering.
- Analysis invalidation.
- Cached analyses.
- Pass configuration.
- Pass instrumentation.
- Pass verification.
- Pass profiling.
- Pass debugging.
- Pass serialization.
- Failure reporting.

### **19.7. Metadata infrastructure**

- Debug metadata.
- Profiling metadata.
- Source locations.
- Optimization annotations.
- Safety metadata.
- Effect summaries.
- Ownership metadata.
- Target metadata.
- ABI metadata.
- Coverage metadata.

### **19.8. Compiler internal APIs**

- AST API.
- Semantic type API.
- Frontend analysis API.
- IR 1 API.
- IR 2 API.
- Machine IR API.
- Target API.
- Pass API.
- Diagnostic API.
- Metadata API.

**Deliverable:** Compiler Core Infrastructure Specification.

# **Volume XIII: IR 1, Effect-typed Sea of Nodes**

## **20. IR 1 complete specification**

This is your first major optimizing representation.

Its job is to preserve source-language semantics while exposing data, control, memory, and effect dependencies.

It should not make premature representation decisions.

### **20.1. IR 1 philosophy**

Define:

- Purpose.
- Semantic guarantees.
- Supported operations.
- Type preservation.
- Graph model.
- Control model.
- Memory model.
- Effect model.
- Ownership metadata.
- Transformation permissions.
- Transformation prohibitions.
- Target independence.

### **20.2. Graph representation**

- Graph ownership.
- Node ownership.
- Node identity.
- Input edges.
- Output edges.
- Value edges.
- Control edges.
- Effect edges.
- Memory edges.
- Dependency ordering.
- Graph entry.
- Graph exit.
- Dead nodes.
- Unreachable nodes.
- Graph regions.

### **20.3. Control nodes**

Specify every control node individually:

- Start.
- If.
- True projection.
- False projection.
- Switch.
- Merge.
- Region.
- Loop.
- Loop header.
- Loop backedge.
- Loop exit.
- Return.
- Unreachable.
- Panic.
- Exceptional exit.
- Invoke, if needed.

For each node, define:

- Inputs.
- Outputs.
- Semantics.
- Dominance behavior.
- Control projection behavior.
- Legal graph transformations.

### **20.4. Value nodes**

- Constants.
- Parameters.
- Arithmetic.
- Comparisons.
- Bitwise operations.
- Conversions.
- Aggregate construction.
- Aggregate extraction.
- Aggregate insertion.
- Enum construction.
- Discriminant extraction.
- Tuple operations.
- Array operations.
- Range operations.
- Function values.
- Closure values.
- Function calls.
- Intrinsics.

### **20.5. Memory nodes**

- Load.
- Store.
- Raw load.
- Raw store.
- Initialization.
- Copy memory.
- Move memory.
- Memory clear.
- Memory allocation.
- Memory deallocation.
- Address formation.
- Pointer offset.
- Pointer comparison.
- Volatile load.
- Volatile store.
- Atomic load.
- Atomic store.
- Atomic read-modify-write.
- Compare-and-swap.
- Memory fence.

### **20.6. Memory SSA**

This needs a full independent specification.

Define:

- Memory state.
- Memory definitions.
- Memory uses.
- Memory merge nodes.
- Memory loop-carried states.
- Memory partitions.
- Alias classes.
- Read dependencies.
- Write dependencies.
- Unknown memory state.
- Memory state splitting.
- Memory state merging.
- Memory dependency pruning.
- Memory state verification.

### **20.7. Effect nodes**

- Effect definitions.
- Effect uses.
- Effect merges.
- Effect partitions.
- Effect dependencies.
- IO.
- Allocation.
- Deallocation.
- Synchronization.
- Atomic operations.
- Panic.
- Suspension.
- Foreign calls.
- Unknown effects.

Define exactly how effect tokens are propagated, combined, and removed.

### **20.8. Type representation**

Every value-producing node needs:

- Semantic type.
- Generic substitution.
- Known value constraints, where available.
- Integer range information, where available.
- Nullability information, where applicable.
- Enum variant information.
- Ownership category.
- Relevant lifetime metadata.
- Representation-independent metadata.

### **20.9. Call nodes**

- Direct calls.
- Indirect calls.
- Generic calls.
- Intrinsics.
- Foreign calls.
- Trait calls.
- Closure calls.
- Recursive calls.
- Calls with panic edges.
- Calls with allocation effects.
- Calls with unknown effect summaries.

### **20.10. Ownership information**

IR 1 should retain information needed to enforce the frontend's semantic guarantees:

- Ownership transitions.
- Move origins.
- Borrow origins.
- Lifetime regions.
- Initialization states.
- Destruction obligations.
- Pinning constraints.
- Address observability facts.
- Object identity facts.

Not every ownership fact needs to become an executable node. Static metadata and graph annotations may be more appropriate.

### **20.11. IR 1 verification**

Define a verifier for:

- Graph structure.
- Node input types.
- Node output types.
- Control-flow validity.
- Dominance.
- Merge validity.
- Memory SSA consistency.
- Effect SSA consistency.
- Call signatures.
- Return types.
- Initialization constraints.
- Ownership constraints.
- Safety metadata.
- Unreachable control.
- Graph termination rules.

### **20.12. IR 1 transformations**

Create a complete pass catalog.

At minimum:

**Simplification**

- Constant folding.
- Constant propagation.
- Identity elimination.
- Algebraic simplification.
- Canonicalization.
- Redundant conversion elimination.

**Value optimization**

- Common subexpression elimination.
- Global value numbering.
- Conditional constant propagation.
- Redundant computation elimination.
- Value numbering through control flow.

**Control optimization**

- Branch simplification.
- Branch elimination.
- Unreachable block elimination.
- Merge simplification.
- Loop simplification.
- Control dependency reduction.
- If conversion, when legal.

**Memory optimization**

- Load elimination.
- Store elimination.
- Store-to-load forwarding.
- Dead store elimination.
- Memory dependence refinement.
- Alias analysis.
- Memory state simplification.
- Redundant initialization elimination.

**Effect optimization**

- Effect dependency simplification.
- Dead effect elimination.
- Effect-aware code motion.
- Call effect propagation.
- Effect summary refinement.

**Loop optimization**

- Loop invariant code motion.
- Induction-variable analysis.
- Strength reduction.
- Loop peeling.
- Loop unrolling.
- Loop unswitching.
- Loop fusion, if legal.
- Loop vectorization preparation.

**Function optimization**

- Inlining.
- Inlining cost models.
- Devirtualization.
- Tail-call recognition.
- Call graph optimization.
- Interprocedural propagation.

### **20.13. IR 1 optimization legality**

For every pass:

- Preconditions.
- Required analyses.
- Semantic preservation conditions.
- Effect preservation.
- Memory preservation.
- Ownership preservation.
- Panic preservation.
- Termination implications.
- Debug information preservation.
- Expected benefit.
- Cost model.
- Verification strategy.

**Deliverables:** IR 1 Semantics Specification, Node Schema, Memory SSA Specification, Effect Graph Specification, Pass Catalog, Verifier Specification.

# **Volume XIV: IR 2, Heterogeneous Sea of Nodes**

## **21. IR 2 complete specification**

This is arguably the most important optimization layer in your architecture.

IR 1 says what the program means.

IR 2 decides how that meaning should be represented and optimized.

This is where you pursue specialization, representation selection, allocation elimination, and aggressive object optimization.

### **21.1. IR 2 purpose and boundaries**

Define:

- Semantic information inherited from IR 1.
- Representation information introduced in IR 2.
- Target-independent versus target-aware decisions.
- Permitted representation transformations.
- Semantic preservation requirements.
- When layout decisions become fixed.
- When physical storage becomes mandatory.
- Which invariants must survive lowering.

### **21.2. Heterogeneous graph model**

- Multiple value representations.
- Multiple aggregate representations.
- Different memory representations.
- Virtual objects.
- Materialized objects.
- Scalar values.
- Vector values.
- Address-bearing values.
- Representation conversions.
- Representation-specific operations.
- Representation compatibility.
- Representation merging at control-flow joins.

### **21.3. Representation type system**

Define internal representation categories:

- Immediate scalar.
- Multi-scalar aggregate.
- Packed scalar.
- Register aggregate.
- Vector.
- Tagged value.
- Niche-encoded value.
- Pointer representation.
- Slice representation.
- Trait-object representation.
- Closure environment.
- Virtual object.
- Stack object.
- Heap object.
- Address-stable object.
- Foreign-layout object.

For each representation, define:

- Supported operations.
- Size.
- Alignment.
- Field access.
- Copy behavior.
- Move behavior.
- Destruction behavior.
- Conversion rules.
- Materialization requirements.

### **21.4. Representation selection**

This subsystem must be specified independently.

Inputs:

- Semantic type.
- Target architecture.
- Usage patterns.
- Escape information.
- Address observability.
- Object identity.
- Layout requirements.
- Register pressure estimates.
- Allocation behavior.
- ABI restrictions.
- Alignment.
- Access frequency.

Outputs:

- Selected representation.
- Materialization requirements.
- Conversion plan.
- Layout decision.
- Specialization decision.

Define selection cost models and how decisions are revised after subsequent optimization.

### **21.5. Aggregate optimization**

- Scalar replacement.
- Aggregate decomposition.
- Field promotion.
- Field elimination.
- Field splitting.
- Aggregate reconstruction.
- Aggregate propagation.
- Aggregate merging, when legal.
- Aggregate materialization.
- Aggregate virtualization.
- Aggregate copy elimination.
- Partial aggregate optimization.

### **21.6. Object optimization**

- Escape analysis.
- Object lifetime analysis.
- Object identity analysis.
- Address observability analysis.
- Virtual object construction.
- Virtual object mutation.
- Virtual object merging, if legal.
- Object materialization.
- Object elimination.
- Object replacement.
- Identity preservation.
- Address-stability handling.
- Pin-aware optimization.

### **21.7. Allocation optimization**

- Allocation elimination.
- Stack promotion.
- Stack allocation.
- Arena allocation.
- Allocation sinking.
- Allocation hoisting.
- Allocation coalescing.
- Reallocation elimination.
- Allocation size specialization.
- Dead allocation elimination.
- Lifetime shortening.
- Allocator-specific optimization.
- Allocation failure preservation.

### **21.8. Type representation optimization**

- Niche optimization.
- Enum layout selection.
- Tagged representation.
- Untagged representation where proven valid.
- Integer representation.
- Pointer representation.
- Optional value representation.
- Boolean representation.
- String and slice representations.
- Trait object representation.
- Closure representation.
- Generic representation.
- Packed representations.

### **21.9. Generic specialization**

- Monomorphization.
- Partial monomorphization.
- Hybrid monomorphization.
- Erased polymorphism.
- Constant specialization.
- Type specialization.
- Layout specialization.
- Function specialization.
- Call-site specialization.
- Specialization caching.
- Specialization recursion.
- Code growth management.
- Specialization profitability.
- ABI interaction.

### **21.10. Closure optimization**

- Capture analysis.
- Environment construction.
- Environment decomposition.
- Environment lifetime.
- Capture ownership.
- Capture borrowing.
- Closure representation.
- Closure allocation elimination.
- Closure specialization.
- Closure conversion.
- Function pointer conversion.

### **21.11. Dynamic dispatch optimization**

- Devirtualization.
- Vtable constant propagation.
- Trait object simplification.
- Dispatch elimination.
- Interface specialization.
- Dynamic dispatch preservation.
- Indirect call optimization.
- Object safety constraints.

### **21.12. Escape and lifetime optimization**

- Local escape analysis.
- Interprocedural escape analysis.
- Return escape.
- Heap escape.
- Thread escape.
- Foreign escape.
- Address escape.
- Lifetime shortening.
- Ownership transfer tracking.
- Reference analysis.

### **21.13. Vectorization and data layout**

- Automatic vectorization.
- Explicit SIMD lowering.
- Data layout specialization.
- Structure-of-arrays transformation.
- Array-of-structures transformation.
- Loop vectorization.
- Vector memory operations.
- Alignment analysis.
- Alias restrictions.
- Vector legality.
- Target feature specialization.

### **21.14. IR 2 optimization passes**

Create a pass catalog for:

- Representation selection.
- Scalar replacement.
- Aggregate decomposition.
- Aggregate reconstruction.
- Escape analysis.
- Allocation elimination.
- Stack promotion.
- Layout optimization.
- Niche selection.
- Boxing elimination.
- Unboxing.
- Closure conversion.
- Closure specialization.
- Generic specialization.
- Devirtualization.
- Constant specialization.
- Object virtualization.
- Object materialization.
- SIMD transformation.
- Loop transformation.
- Interprocedural specialization.

### **21.15. IR 2 verification**

This is essential.

The verifier must check:

- Every representation is legal for its semantic type.
- Representation conversions preserve values.
- Ownership remains valid.
- Destruction remains correct.
- Object identity is preserved where observable.
- Address observability requirements are respected.
- Pinning restrictions are preserved.
- Layout contracts are satisfied.
- Memory dependencies remain correct.
- Effect dependencies remain correct.
- ABI requirements remain satisfied.
- No illegal representation is passed to Machine IR.

### **21.16. Optimization profitability**

You need explicit cost models for:

- Code growth.
- Compilation time.
- Runtime performance.
- Register pressure.
- Instruction-cache pressure.
- Allocation reduction.
- Memory footprint.
- Cache locality.
- Branch behavior.
- Specialization count.
- Inlining.
- Loop transformations.

The compiler must not assume that a transformation is good merely because it removes an operation.

For example, aggressive inlining can reduce call overhead while increasing instruction-cache pressure.

**Deliverables:** IR 2 Semantics Specification, Representation System Specification, Specialization Specification, Object Optimization Specification, Pass Catalog, Verifier.

# **Volume XV: Machine IR**

## **22. Machine IR specification**

This is where the compiler stops representing abstract source-level semantics and starts representing actual hardware execution.

I recommend distinguishing the IR's universal machine concepts from target-specific instruction descriptions.

### **22.1. Machine IR model**

- Basic blocks.
- Machine instructions.
- Virtual registers.
- Physical registers.
- Register classes.
- Stack slots.
- Machine operands.
- Control-flow edges.
- Machine memory operands.
- Labels.
- Calls.
- Returns.
- Branches.
- Terminators.
- Inline assembly.
- Target intrinsics.

### **22.2. Register model**

- Register identities.
- Register classes.
- Register widths.
- Register aliases.
- Subregisters.
- Register pairs.
- Fixed registers.
- Reserved registers.
- Caller-saved registers.
- Callee-saved registers.
- Special-purpose registers.
- Register constraints.

### **22.3. Instruction model**

Every target instruction needs metadata:

- Opcode.
- Operand count.
- Operand types.
- Operand constraints.
- Definitions.
- Uses.
- Register effects.
- Flag effects.
- Memory effects.
- Latency model.
- Throughput model.
- Encoding requirements.
- Feature requirements.
- Scheduling properties.

### **22.4. Machine memory operands**

- Base registers.
- Index registers.
- Scale.
- Displacement.
- Addressing modes.
- Alignment.
- Memory width.
- Volatility.
- Atomic ordering.
- Address-space information.
- Alias metadata.
- Stack references.
- TLS references.

### **22.5. Machine-level control flow**

- Conditional branches.
- Unconditional branches.
- Indirect branches.
- Calls.
- Returns.
- Switch lowering.
- Jump tables.
- Branch probabilities.
- Tail calls.
- Exception edges.
- Unreachable instructions.

### **22.6. Machine-level operations**

- Integer arithmetic.
- Floating-point arithmetic.
- Vector arithmetic.
- Comparisons.
- Conversions.
- Bit operations.
- Shifts.
- Loads.
- Stores.
- Atomic operations.
- Fences.
- Calls.
- Stack operations.
- Target-specific intrinsics.

### **22.7. Machine IR invariants**

- Virtual registers are correctly defined.
- Uses are valid.
- Physical register constraints are legal.
- Instructions use supported operand forms.
- Control flow is valid.
- Memory operations preserve required semantics.
- Calling conventions are respected.
- Stack alignment is correct.
- Special registers are handled.
- No unlowered abstract operation remains.

### **22.8. Machine IR optimization**

- Machine constant propagation.
- Machine copy propagation.
- Machine CSE.
- Redundant move elimination.
- Peephole optimization.
- Instruction combining.
- Branch simplification.
- Register coalescing.
- Machine-level dead-code elimination.
- Instruction scheduling.
- Target-specific optimization.

### **22.9. Machine IR verification**

Provide a verifier after every major lowering stage.

At minimum:

- Pre-register-allocation verifier.
- Post-register-allocation verifier.
- Post-scheduling verifier.
- Pre-emission verifier.

**Deliverables:** Machine IR Specification, Register Model Specification, Instruction Description Format, Machine Verifier.

# **Volume XVI: Backend and target architecture**

## **23. Target architecture system**

You need a formal target description system rather than hardcoding assumptions throughout the backend.

### **23.1. Target description**

For every supported target define:

- Architecture name.
- Subarchitecture.
- Instruction set.
- Register file.
- Register classes.
- Instruction encodings.
- Addressing modes.
- Calling conventions.
- Stack rules.
- Data layout.
- Endianness.
- Pointer widths.
- Atomic capabilities.
- SIMD capabilities.
- Feature dependencies.
- ABI requirements.

### **23.2. Initial architecture targets**

I would initially support:

- x86-64.
- AArch64.
- RISC-V 64-bit.

Each needs a separate architecture specification.

Do not assume that an instruction available on one target has equivalent behavior on another.

### **23.3. Instruction selection**

Define:

- Selection rules.
- Pattern matching.
- Legalization.
- Instruction costs.
- Target-specific lowering.
- Complex addressing selection.
- SIMD selection.
- Atomic selection.
- Inline assembly integration.
- Unsupported instruction handling.

### **23.4. Register allocation**

This deserves its own specification.

- Liveness analysis.
- Live intervals.
- Live-range splitting.
- Register classes.
- Register constraints.
- Register allocation algorithm.
- Spill decisions.
- Spill slot allocation.
- Register coalescing.
- Rematerialization.
- Caller-saved handling.
- Callee-saved handling.
- Fixed-register handling.
- Register pressure.
- Allocation verification.

Possible algorithms include graph coloring and linear scan. The compiler should be able to use different strategies for different optimization modes or function characteristics.

### **23.5. Instruction scheduling**

- Dependency construction.
- Data dependencies.
- Memory dependencies.
- Control dependencies.
- Register pressure.
- Instruction latency.
- Throughput.
- Execution ports.
- Scheduling regions.
- Loop scheduling.
- Target-specific models.

### **23.6. Stack and frame lowering**

- Stack frame layout.
- Local stack objects.
- Spill slots.
- Alignment.
- Stack arguments.
- Saved registers.
- Red zones.
- Frame pointers.
- Dynamic stack allocation.
- Stack probing.
- Stack protection.
- Stack unwinding.
- Frame elimination.

### **23.7. Prologue and epilogue generation**

- Register saving.
- Register restoration.
- Stack allocation.
- Stack deallocation.
- Frame pointer setup.
- Return value handling.
- Calling convention compliance.
- Exception-related cleanup.
- Platform-specific entry requirements.

### **23.8. Target-specific optimization**

- Instruction combining.
- Addressing-mode folding.
- Branch optimization.
- Vector instruction optimization.
- Multiply/divide lowering.
- Bit manipulation.
- Atomic instruction selection.
- Cache-related hints.
- Architectural instruction idioms.
- Microarchitecture-specific tuning.

### **23.9. Machine code emission**

- Instruction encoding.
- Instruction sizing.
- Branch relaxation.
- Alignment.
- Relocation emission.
- Symbol references.
- Literal pools.
- Constant islands.
- Section placement.
- Debug information.
- Code metadata.
- Emission validation.

**Deliverables:** Target Description Specification, Instruction Selection Specification, Register Allocation Specification, Scheduling Specification, Code Generation Specification.

# **Volume XVII: Object files, linker, and executable generation**

## **24. Object generation**

The backend does not finish when it produces instruction bytes.

A production compiler must produce correctly structured object files and executables.

### **24.1. Object formats**

Initially define support for:

- ELF.
- PE/COFF.
- Mach-O.

Potentially add WebAssembly modules as a separate target format.

For each format specify:

- Sections.
- Symbols.
- Relocations.
- Alignment.
- Symbol visibility.
- Weak symbols.
- TLS.
- Debug information.
- Unwind metadata.
- Architecture metadata.
- Object versioning.
- Format-specific limitations.

### **24.2. Relocations**

- Absolute relocations.
- Relative relocations.
- PC-relative relocations.
- GOT relocations.
- PLT relocations.
- TLS relocations.
- Architecture-specific relocations.
- Relocation overflow.
- Relocation relaxation.
- Symbol resolution.
- Relocation validation.

### **24.3. Linker integration**

Decide whether to:

- Invoke platform linkers.
- Implement an internal linker.
- Support both.

A mature compiler can integrate with existing platform linkers while optionally providing specialized linking facilities.

Define:

- Static linking.
- Dynamic linking.
- Shared libraries.
- Executable linking.
- Library search paths.
- Symbol resolution.
- Library ordering.
- Dead-section elimination.
- Link-time optimization.
- Symbol versioning.
- Runtime library selection.
- Cross-linking.

### **24.4. Link-time optimization**

If supported:

- Link-time IR representation.
- Module summaries.
- Cross-module analysis.
- Cross-module inlining.
- Cross-module specialization.
- Symbol visibility analysis.
- Whole-program optimization.
- Code elimination.
- Cache behavior.
- LTO failure recovery.
- Thin versus full LTO.

### **24.5. Executable generation**

- Executable entry point.
- Startup code.
- Runtime initialization.
- Static initialization.
- TLS initialization.
- Runtime linking.
- Process termination.
- Exit codes.
- Program metadata.
- Platform-specific entry requirements.

**Deliverables:** Object Format Specification, Relocation Specification, Linker Interface Specification, Executable Format Contract.

# **Volume XVIII: Runtime and platform support**

## **25. Runtime architecture**

A systems language needs a deliberate runtime policy.

Your current objective is no mandatory garbage collector, but that does not mean there is no runtime support.

### **25.1. Runtime modes**

Define:

- No-runtime mode.
- Freestanding mode.
- Minimal hosted mode.
- Full hosted mode.
- Custom runtime mode.

Specify which language features require which runtime components.

### **25.2. Runtime initialization**

- Program entry.
- Static initialization.
- TLS initialization.
- Argument handling.
- Environment access.
- Standard streams.
- Runtime startup.
- Runtime shutdown.
- Exit processing.

### **25.3. Memory runtime**

- Default allocator.
- Allocation interfaces.
- Deallocation interfaces.
- Allocator initialization.
- Allocation failure behavior.
- Memory alignment.
- Custom allocators.
- Arena interfaces.
- Stack allocation.
- Runtime allocation diagnostics.

### **25.4. Panic and unwinding runtime**

- Panic entry.
- Panic payload.
- Panic reporting.
- Unwinding mechanism.
- Exception metadata.
- Stack unwinding.
- Destructor execution.
- Abort behavior.
- Double panic.
- Foreign unwind boundaries.
- Runtime configuration.

### **25.5. Concurrency runtime**

- Thread support.
- Thread-local storage.
- Synchronization primitives.
- Atomic support.
- Scheduler support.
- Async executor interfaces.
- Task allocation.
- Wake mechanisms.
- Cancellation.

### **25.6. Runtime instrumentation**

- Stack overflow detection.
- Profiling hooks.
- Allocation tracking.
- Sanitizer hooks.
- Coverage instrumentation.
- Runtime assertions.
- Backtrace support.
- Debugger integration.

### **25.7. Runtime ABI**

- Runtime entry points.
- Runtime data structures.
- Unwind ABI.
- Allocator ABI.
- Panic ABI.
- Thread ABI.
- TLS ABI.
- Version compatibility.

**Deliverable:** Runtime Architecture and Runtime ABI Specification.

# **Volume XIX: Standard library**

## **26. Standard library specification**

A language is not attractive to systems programmers merely because its compiler is impressive.

Its standard library determines whether writing ordinary software is pleasant or an exercise in building everything from scratch.

I'd divide the standard library into the following modules.

### **26.1. Core**

- Primitive traits.
- Copy.
- Clone.
- Drop.
- Default.
- Equality.
- Ordering.
- Hashing.
- Formatting.
- Debug formatting.
- Conversion.
- Iterator interfaces.
- Numeric traits.
- Error interfaces.
- Option.
- Result.
- Range types.
- Phantom types.
- Manually managed storage.

### **26.2. Memory**

- Box-like owned pointers.
- Reference-counted pointers.
- Thread-safe reference counting.
- Non-null pointers.
- Maybe-uninitialized storage.
- Manually dropped values.
- Pinning.
- Allocator interfaces.
- Arena interfaces.
- Region interfaces.
- Raw allocation APIs.

### **26.3. Collections**

- Dynamic arrays.
- Fixed arrays.
- Slices.
- Strings.
- Byte buffers.
- Hash maps.
- Ordered maps.
- Hash sets.
- Ordered sets.
- Linked lists.
- Intrusive lists.
- Deques.
- Ring buffers.
- Binary heaps.
- Trees.
- Bitsets.
- Specialized numeric containers.

### **26.4. Text and formatting**

- UTF-8 strings.
- String builders.
- Character operations.
- Unicode operations.
- Formatting.
- Parsing.
- Numeric formatting.
- Text encoding.
- Text decoding.
- Byte-oriented APIs.

### **26.5. Numeric library**

- Integer utilities.
- Floating-point utilities.
- Mathematical functions.
- SIMD abstractions.
- Numeric conversions.
- Saturating arithmetic.
- Wrapping arithmetic.
- Checked arithmetic.
- Big integers, if provided.

### **26.6. IO**

- Standard input.
- Standard output.
- Standard error.
- File operations.
- Buffered IO.
- Unbuffered IO.
- Filesystem operations.
- Directory operations.
- Streams.
- Sockets.
- Asynchronous IO interfaces.

### **26.7. Operating system**

- Processes.
- Environment variables.
- Filesystem.
- Memory mapping.
- Signals.
- Threads.
- System information.
- Timers.
- Synchronization.
- Platform APIs.

### **26.8. Concurrency**

- Thread abstraction.
- Mutex.
- Read-write lock.
- Condition variable.
- Semaphore.
- Atomic types.
- Channels.
- Scoped threads.
- Task interfaces.
- Executor interfaces.

### **26.9. Networking**

- TCP.
- UDP.
- Socket interfaces.
- Address parsing.
- DNS integration.
- Networking protocols.
- Async networking.
- Buffer management.

### **26.10. Testing and diagnostics**

- Assertions.
- Unit testing.
- Test discovery.
- Test reporting.
- Benchmarking.
- Property-based testing.
- Fuzzing integration.
- Diagnostic formatting.

### **26.11. Cryptography and security**

Either provide a carefully maintained library or specify supported external ecosystem interfaces.

- Secure random generation.
- Hash functions.
- Cryptographic primitives.
- Constant-time operations.
- Secret storage.
- Secure memory clearing.
- Cryptographic API contracts.

### **26.12. Standard library guarantees**

For every library module:

- Public API.
- Type signatures.
- Effects.
- Ownership behavior.
- Allocation behavior.
- Thread safety.
- Panic behavior.
- Performance expectations.
- ABI guarantees.
- Platform support.
- Version stability.
- Documentation.
- Tests.

**Deliverable:** Standard Library API and Behavioral Specification.

# **Volume XX: Package manager and build system**

## **27. Package management**

### **27.1. Package model**

- Package identity.
- Package versions.
- Package manifests.
- Dependencies.
- Dependency constraints.
- Feature selection.
- Optional dependencies.
- Platform-specific dependencies.
- Development dependencies.
- Build dependencies.
- Dependency resolution.
- Lockfiles.
- Registry support.
- Git dependencies.
- Local dependencies.

### **27.2. Dependency security**

- Dependency authenticity.
- Package checksums.
- Signature verification.
- Registry trust.
- Dependency auditing.
- Vulnerability reporting.
- Dependency replacement.
- Supply-chain policy.
- Reproducible dependency retrieval.

### **27.3. Build system**

- Build targets.
- Compilation units.
- Build graph.
- Dependency graph.
- Build ordering.
- Incremental builds.
- Parallel builds.
- Build caching.
- Remote caching.
- Cross-compilation.
- Build profiles.
- Debug and release modes.
- Feature configurations.
- Conditional compilation.
- Generated source.
- Static library generation.
- Shared library generation.
- Executable generation.

### **27.4. Build reproducibility**

- Deterministic compiler output.
- Deterministic dependency resolution.
- Environment isolation.
- Toolchain identification.
- Build manifests.
- Build provenance.
- Timestamp handling.
- File path handling.
- Compiler configuration capture.
- Artifact verification.

### **27.5. Incremental compilation**

This needs particularly careful design.

- Dependency tracking.
- Source invalidation.
- Type invalidation.
- Generic instance invalidation.
- Trait implementation invalidation.
- Constant evaluation invalidation.
- Macro expansion invalidation.
- IR cache invalidation.
- Cross-module invalidation.
- Cache serialization.
- Cache compatibility.
- Cache corruption recovery.
- Compilation restart behavior.

### **27.6. Compiler caching**

- Frontend cache.
- Generic specialization cache.
- IR 1 cache.
- IR 2 cache.
- Machine IR cache.
- Object cache.
- Build result cache.
- Target-specific cache.
- Cache keys.
- Cache invalidation.
- Cache storage.
- Cache eviction.

**Deliverables:** Package Manager Specification, Build System Specification, Incremental Compilation Specification, Reproducible Builds Specification.

# **Volume XXI: Tooling and developer experience**

## **28. Toolchain ecosystem**

This is a production requirement, not optional decoration.

### **28.1. Language server**

- Completion.
- Hover information.
- Go to definition.
- Find references.
- Rename.
- Type information.
- Diagnostics.
- Code actions.
- Import suggestions.
- Refactoring.
- Semantic highlighting.
- Macro expansion inspection.
- Borrow explanation.
- Generic constraint explanation.
- Effect inspection.
- Incremental analysis.

### **28.2. Formatter**

- Formatting rules.
- Stability.
- Configuration.
- Comments.
- Attributes.
- Macro formatting.
- Generated code behavior.
- Error recovery.
- Version compatibility.

### **28.3. Documentation system**

- API documentation.
- Module documentation.
- Code examples.
- Cross-references.
- Generic documentation.
- Trait documentation.
- Effect documentation.
- Safety documentation.
- Example execution.
- Search.
- Versioned documentation.

### **28.4. Debugger support**

- Debug symbols.
- Variable locations.
- Source mapping.
- Type inspection.
- Generic type inspection.
- Trait object inspection.
- Closure inspection.
- Optimized variable tracking.
- Async state inspection.
- Stack unwinding.
- Breakpoints.
- Watchpoints.
- Expression evaluation.

### **28.5. Profiling**

- CPU profiling.
- Sampling.
- Instrumentation.
- Allocation profiling.
- Memory profiling.
- Call graphs.
- Branch profiling.
- Hardware counters.
- Async task profiling.
- Compiler pass profiling.
- Optimization remarks.

### **28.6. Testing tools**

- Unit tests.
- Integration tests.
- Compile-fail tests.
- Runtime tests.
- Benchmark tests.
- Documentation tests.
- Fuzz tests.
- Sanitizer execution.
- Test filtering.
- Parallel execution.
- Test discovery.

### **28.7. Developer commands**

A complete toolchain should define commands for:

- Building.
- Testing.
- Formatting.
- Documentation.
- Benchmarking.
- Checking without code generation.
- Cleaning.
- Inspecting dependencies.
- Inspecting compiler diagnostics.
- Emitting IR 1.
- Emitting IR 2.
- Emitting Machine IR.
- Disassembling.
- Inspecting assembly.
- Inspecting optimization decisions.
- Inspecting layout.
- Inspecting type information.

**Deliverable:** Developer Tooling Specification.

# **Volume XXII: Testing, validation, and formal verification**

## **29. Compiler correctness strategy**

A production compiler must have an explicit correctness strategy.

"All tests pass" is not a correctness argument.

You need multiple layers of validation.

### **29.1. Language conformance tests**

- Lexical tests.
- Parsing tests.
- Grammar tests.
- Name resolution tests.
- Type checking tests.
- Generic tests.
- Trait resolution tests.
- Inference tests.
- Ownership tests.
- Borrow checking tests.
- Lifetime tests.
- Effect tests.
- Pattern tests.
- Constant evaluation tests.
- Async tests.
- FFI tests.
- Unsafe tests.
- ABI tests.

### **29.2. Compile-pass and compile-fail tests**

Every semantic rule needs positive and negative examples.

For example:

- Valid move.
- Invalid double move.
- Valid shared borrows.
- Invalid conflicting mutable borrow.
- Valid partial initialization.
- Invalid use of uninitialized fields.
- Valid reborrow.
- Invalid reference escape.
- Valid effect contract.
- Invalid effect contract.
- Valid trait implementation.
- Invalid coherence overlap.

Diagnostics should also be tested, not just whether compilation succeeds.

### **29.3. IR verification tests**

For every IR:

- Valid graph tests.
- Invalid graph tests.
- Type consistency tests.
- Control-flow tests.
- Memory SSA tests.
- Effect dependency tests.
- Ownership metadata tests.
- Representation consistency tests.
- Machine constraint tests.
- Serialization tests.

### **29.4. Optimization correctness tests**

Every optimization needs:

- A precondition test.
- A successful optimization test.
- A case where it must not optimize.
- An interaction test with other optimizations.
- An edge-case test.
- A semantic equivalence test.
- A regression test.

For example, allocation elimination should be tested against:

- Unobservable allocation.
- Address-observable allocation.
- Foreign pointer escape.
- Custom allocator effects.
- Allocation failure behavior.
- Destructor side effects.
- Object identity.
- Pinned objects.

### **29.5. Differential testing**

Use multiple implementations or execution modes to compare behavior.

Potential reference implementations:

- A simple semantic interpreter.
- An unoptimized compiler mode.
- The optimized compiler.
- A separate reference backend.

Compare results and observable behavior for generated programs.

This is especially valuable for finding optimizer bugs that ordinary tests miss.

### **29.6. Fuzzing**

You need fuzzers for:

- Lexer.
- Parser.
- AST construction.
- Name resolution.
- Type inference.
- Trait solving.
- Borrow checker.
- Constant evaluator.
- Macro expansion.
- IR construction.
- IR transformations.
- Machine instruction selection.
- Object emission.
- Linker integration.
- Debug information generation.

Also build an end-to-end program fuzzer.

### **29.7. Property-based testing**

Test general properties such as:

- Parsing and printing consistency.
- Type substitution consistency.
- Graph mutation consistency.
- IR verifier preservation.
- Deterministic compilation.
- Serialization and deserialization consistency.
- Semantic equivalence of optimization passes.
- Validity of generated machine instructions.

### **29.8. Runtime validation**

- Address sanitization.
- Undefined-behavior detection.
- Thread sanitization.
- Memory leak detection.
- Stack overflow testing.
- Allocation failure injection.
- Panic injection.
- Concurrency stress tests.
- FFI stress tests.
- Unwinding tests.

### **29.9. Backend testing**

- Instruction encoding tests.
- Register allocation tests.
- Calling convention tests.
- ABI conformance tests.
- Object relocation tests.
- Linker tests.
- Disassembly comparisons.
- Emulator tests.
- Hardware tests.
- Cross-target tests.

## **30. Formal verification**

Given your interest in a formally verified compiler, I'd make this a dedicated workstream rather than an afterthought.

First, decide what "formally verified" means.

There are several different levels:

| Level                    | Assurance                                                |
| ------------------------ | -------------------------------------------------------- |
| Verified data structures | Internal graph and container invariants                  |
| Verified analyses        | Correctness of type, ownership, and effect analyses      |
| Verified transformations | Individual optimization passes preserve semantics        |
| Verified backend         | Machine lowering preserves the defined machine semantics |
| Verified compiler        | End-to-end translation preserves program semantics       |

These are not interchangeable.

### **30.1. Formal language semantics**

Define:

- Source-language operational semantics.
- Type safety properties.
- Memory safety properties.
- Ownership soundness.
- Borrow checker soundness.
- Effect soundness.
- Concurrency safety properties.
- FFI boundary assumptions.
- Undefined behavior semantics.

### **30.2. IR semantics**

For every IR:

- Formal node semantics.
- Graph execution semantics.
- Control-flow semantics.
- Memory semantics.
- Effect semantics.
- Type preservation requirements.
- Execution equivalence.
- Valid graph invariants.

### **30.3. Transformation proofs**

For every optimizing pass:

- Preconditions.
- Transformation definition.
- Postconditions.
- Semantic preservation argument.
- Memory preservation.
- Effect preservation.
- Control-flow preservation.
- Ownership preservation.
- Proof or validation artifact.

### **30.4. Translation correctness**

Prove or validate the transitions:

- Source semantics to frontend semantic representation.
- Frontend representation to IR 1.
- IR 1 to IR 2.
- IR 2 to Machine IR.
- Machine IR to emitted instructions.

### **30.5. Trusted computing base**

Identify every component that is trusted.

Potential components:

- Lexer and parser.
- Type checker.
- Trait solver.
- Borrow checker.
- IR verifier.
- Optimization pass infrastructure.
- Target descriptions.
- Machine instruction encoder.
- Object writer.
- Linker.
- Runtime.
- Formal proof checker.

A formally verified optimization pass does not establish end-to-end compiler correctness if the rest of the compiler remains unverified.

### **30.6. Proof infrastructure**

- Specification language.
- Proof assistant.
- Formal models.
- Verified algorithms.
- Proof automation.
- Proof checking.
- Proof artifact storage.
- Proof reproducibility.
- Specification-to-implementation correspondence.
- Proof regression testing.

You should also decide whether your language specification itself is a formal executable model or a conventional normative document accompanied by formal semantics.

### **30.7. Self-hosting and bootstrapping**

If self-hosting is a project objective, define:

- Initial compiler implementation language.
- Bootstrap compiler.
- Stage-one compiler.
- Stage-two compiler.
- Stage comparison.
- Reproducible bootstrap.
- Trusted bootstrap chain.
- Cross-compilation bootstrap.
- Compiler self-build tests.
- Compiler version transitions.
- Compiler distribution integrity.

A self-hosting compiler is not automatically a trustworthy compiler. Bootstrapping needs its own security and correctness model.

**Deliverables:** Conformance Test Suite, Compiler Validation Plan, Formal Semantics Specification, Proof Plan, Bootstrap Specification.

# **Volume XXIII: Performance and scalability engineering**

## **31. Compiler performance specification**

Your language has aggressive runtime optimization goals. The compiler itself also needs to be fast and scalable.

### **31.1. Compilation performance**

Measure:

- Lexing time.
- Parsing time.
- Name resolution time.
- Type checking time.
- Trait solving time.
- Borrow checking time.
- Effect inference time.
- IR 1 construction time.
- IR 1 optimization time.
- IR 2 construction time.
- IR 2 optimization time.
- Machine lowering time.
- Register allocation time.
- Code emission time.
- Linking time.

### **31.2. Memory usage**

- Peak compiler memory.
- AST memory.
- Type system memory.
- Trait solver memory.
- Graph memory.
- Optimization analysis memory.
- Debug metadata memory.
- Incremental cache memory.
- Parallel compilation memory.
- Memory reclamation.

### **31.3. Incremental performance**

- Rebuild minimization.
- Dependency precision.
- Fine-grained invalidation.
- Cache hit rate.
- Cache serialization overhead.
- Cross-module incremental optimization.
- Generic instance caching.
- Background analysis.

### **31.4. Parallel compilation**

- Module-level parallelism.
- Function-level parallelism.
- IR optimization parallelism.
- Generic specialization parallelism.
- Backend parallelism.
- Parallel code emission.
- Shared compiler state.
- Deterministic parallel execution.
- Synchronization costs.
- Memory scaling.

### **31.5. Optimization scalability**

Track:

- Graph node count.
- Graph edge count.
- Analysis complexity.
- Pass execution time.
- Optimization memory usage.
- Specialization count.
- Inlining growth.
- Alias analysis complexity.
- Memory SSA complexity.
- Code size.
- Compilation-time regressions.

### **31.6. Optimization quality**

Measure generated programs for:

- Execution time.
- Cycles per operation.
- Instructions retired.
- Branch misses.
- Cache misses.
- Memory bandwidth.
- Allocation count.
- Allocation bytes.
- Stack usage.
- Code size.
- Startup time.
- SIMD utilization.

### **31.7. Cost models**

Every major optimization should have a documented profitability model.

Define models for:

- Inlining.
- Specialization.
- Loop unrolling.
- Vectorization.
- Allocation elimination.
- Stack promotion.
- Representation selection.
- Code layout.
- Instruction scheduling.
- Register allocation.

Do not base the models exclusively on abstract operation counts. Architecture and workload matter.

### **31.8. Benchmark infrastructure**

- Compiler microbenchmarks.
- Compiler workload benchmarks.
- Runtime microbenchmarks.
- Real application benchmarks.
- Allocation benchmarks.
- Memory benchmarks.
- Compile-time benchmarks.
- Cross-architecture benchmarks.
- Statistical reporting.
- Baseline preservation.
- Regression detection.

**Deliverables:** Compiler Performance Specification, Runtime Performance Specification, Benchmark Suite, Optimization Cost Model Documentation.

# **Volume XXIV: Security, reliability, and production operations**

## **32. Compiler security**

A production compiler processes untrusted source code, dependencies, build scripts, and potentially compile-time programs.

It therefore needs its own security model.

### **32.1. Compiler input security**

- Malformed source handling.
- Parser resource limits.
- Recursion limits.
- Generic instantiation limits.
- Trait solver limits.
- Constant evaluation limits.
- Macro expansion limits.
- Graph size limits.
- Memory exhaustion behavior.
- Integer overflow in compiler internals.
- Compiler crash handling.

### **32.2. Build execution security**

- Build script sandboxing.
- Procedural macro sandboxing.
- Compile-time execution restrictions.
- Filesystem access controls.
- Network access controls.
- Process execution controls.
- Environment isolation.
- Resource limits.
- Dependency trust.
- Build cache trust.

### **32.3. Supply-chain security**

- Signed compiler releases.
- Signed package manifests.
- Dependency provenance.
- Build artifact integrity.
- Compiler checksum validation.
- Reproducible releases.
- Bootstrap integrity.
- Dependency vulnerability handling.

### **32.4. Compiler reliability**

- Graceful error handling.
- Internal compiler error reports.
- Crash reproduction.
- Compiler state dumps.
- Diagnostic quality.
- Recovery from malformed source.
- Corrupt cache recovery.
- Interrupted build recovery.
- Disk exhaustion handling.
- Out-of-memory behavior.

### **32.5. Runtime security**

- Stack protection.
- Memory protection.
- Hardened allocation options.
- Control-flow protection.
- Platform security features.
- Sanitizer support.
- Secure FFI boundaries.
- Secure cryptographic facilities.

**Deliverable:** Compiler Security and Reliability Specification.

# **Volume XXV: Production, release, and governance**

## **33. Release engineering**

- Version numbering.
- Release channels.
- Stable releases.
- Development releases.
- Nightly builds.
- Compiler distribution.
- Binary packaging.
- Source packaging.
- Cross-platform distribution.
- Installation.
- Uninstallation.
- Upgrade handling.
- Rollback.
- Package signatures.
- Release checksums.
- Reproducible builds.

## **34. Language evolution**

- Feature proposal process.
- Language change proposals.
- Design review.
- Prototype implementation.
- Semantic review.
- Performance evaluation.
- Safety review.
- Compatibility review.
- Stabilization.
- Deprecation.
- Removal.
- Edition policy.
- RFC archival.
- Specification versioning.

## **35. Documentation**

You need documentation for:

- Language specification.
- Compiler architecture.
- Type system.
- Borrow checker.
- Memory model.
- Effects.
- Concurrency.
- Async.
- IR 1.
- IR 2.
- Machine IR.
- Backend architecture.
- Runtime.
- Standard library.
- FFI.
- ABI.
- Build system.
- Package manager.
- Tooling.
- Debugging.
- Profiling.
- Formal verification.
- Compiler development.
- Contribution guidelines.

## **36. Maintenance and ecosystem**

- Issue reporting.
- Bug triage.
- Security disclosures.
- Regression policy.
- Compatibility testing.
- External library ecosystem.
- Package registry operations.
- Documentation maintenance.
- Compiler maintainer roles.
- Release ownership.
- Support policy.
- Long-term maintenance.

**Deliverable:** Production and Governance Specification.

# **Volume XXVI: Optional compiler products and execution modes**

These are not mandatory for the first production release, but the architecture should leave room for them.

## **37. Reference interpreter**

I strongly recommend a small, deliberately simple interpreter for the language's semantic core.

Define:

- Value representation.
- Execution state.
- Memory model.
- Ownership behavior.
- Evaluation order.
- Control-flow behavior.
- Function calls.
- Generic execution.
- Panic behavior.
- Effects.
- Async execution, if supported.
- Foreign function simulation.
- Debugging.
- Execution limits.

Its primary purpose is semantic validation and differential testing. It should favor clarity over optimization.

## **38. JIT compilation**

Only necessary if you intend to support runtime code generation.

Define:

- JIT compilation API.
- Executable memory allocation.
- Memory permission transitions.
- Code installation.
- Symbol resolution.
- Runtime linking.
- Calling conventions.
- Code relocation.
- Code invalidation.
- Thread safety.
- Instruction-cache synchronization.
- Debug information.
- Profiling.
- JIT security.
- Dynamic specialization.
- Deoptimization, if supported.

## **39. Runtime specialization**

If you eventually want adaptive optimization:

- Runtime profiling.
- Value profiling.
- Type profiling.
- Hot-path detection.
- Compilation thresholds.
- Specialized code versions.
- Guard construction.
- Guard failure.
- Deoptimization.
- Recompilation.
- Code version management.
- Concurrent compilation.

I would not include this in the initial compiler. It introduces another substantial correctness problem.

# **The complete deliverable inventory**

Here is the consolidated specification library. The previous outline actually contains 26 volumes, rather than the 24 in my introduction. The individual volumes are the important part, not the numbering.

## **Master specification library**

26 volumes · Language, compiler, runtime, and production

0 / 26 reviewed

0%

**1. Language charter**

Goals, safety, performance, compatibility, targets

**2. Lexical and syntactic specification**

Tokens, grammar, expressions, precedence

**3. Names and modules**

Resolution, visibility, packages, declarations

**4. Type system**

Types, traits, generics, inference, capabilities

**5. Value and memory semantics**

Identity, initialization, moves, drops, layout

**6. Ownership and borrow checker**

Places, loans, lifetimes, pinning, safety

**7. Effects and execution semantics**

Effects, evaluation, control flow, errors

**8. Concurrency and async**

Memory model, tasks, atomics, cancellation

**9. Compile-time programming**

Const evaluation, reflection, macros

**10. Attributes and low-level interfaces**

Intrinsics, assembly, SIMD, FFI, ABI

**11. Frontend architecture**

Lexer, parser, AST, resolution, semantic analysis

**12. Compiler infrastructure**

Types, graphs, symbols, passes, diagnostics

**13. IR 1**

Effect-typed Sea of Nodes

**14. IR 2**

Heterogeneous Sea of Nodes and specialization

**15. Machine IR**

Registers, instructions, constraints, verification

**16. Backend and targets**

Instruction selection, allocation, scheduling

**17. Object generation and linking**

Object formats, relocations, executables

**18. Runtime**

Startup, allocation, unwinding, concurrency

**19. Standard library**

Core, collections, IO, OS, concurrency

**20. Packages and build system**

Dependencies, builds, caching, reproducibility

**21. Developer tooling**

LSP, formatting, debugging, profiling, tests

**22. Correctness and verification**

Conformance, proofs, fuzzing, differential testing

**23. Performance engineering**

Compilation scalability and runtime optimization

**24. Security and reliability**

Sandboxing, supply chain, failure handling

**25. Production and governance**

Releases, compatibility, evolution, maintenance

**26. Optional execution products**

Interpreter, JIT, adaptive optimization

Reset checklist

# **40. The specification rule every feature must follow**

This is something I would enforce throughout the project.

No feature should be considered specified just because its syntax, type signature, and implementation exist.

Every language feature and compiler operation needs a complete definition.

Use this template for every feature:

| Specification requirement | What must be established                 |
| ------------------------- | ---------------------------------------- |
| 1. Syntax                 | Exact grammar                            |
| 2. Static semantics       | What is accepted or rejected             |
| 3. Type semantics         | Input and output types                   |
| 4. Value semantics        | What values are produced                 |
| 5. Evaluation semantics   | What happens during execution            |
| 6. Ownership              | Moves, copies, borrows, destruction      |
| 7. Effects                | Observable state changes                 |
| 8. Control flow           | Normal, exceptional, and diverging paths |
| 9. Compile-time behavior  | Constant evaluation and specialization   |
| 10. Memory behavior       | Storage, aliasing, layout, lifetime      |
| 11. ABI behavior          | Calling and representation requirements  |
| 12. IR 1 mapping          | Semantic graph representation            |
| 13. IR 2 mapping          | Representation and optimization          |
| 14. Machine IR mapping    | Target operation selection               |
| 15. Diagnostics           | Invalid usage and explanations           |
| 16. Validation            | Tests, invariants, proofs                |

For optimization passes, replace the language-facing requirements with:

- Exact preconditions.
- Required analyses.
- Transformation algorithm.
- Preservation conditions.
- Invalid transformation conditions.
- Complexity characteristics.
- Profitability model.
- Verifier requirements.
- Regression tests.
- Correctness proof or validation strategy.

This prevents the project from accumulating features that are implemented but not fully understood.

# **41. Implementation dependency order**

The specification should be written in dependency order, not implementation convenience.

Here is the order I recommend.

## **Development sequence**

1. **Foundation and semantic definitions**

   Volumes 1–10.

   Establish what the language means, how types work, what ownership guarantees exist, and how execution behaves.
2. **Frontend and compiler infrastructure**

   Volumes 11–12.

   Build source handling, parsing, name resolution, type checking, trait solving, constant evaluation, borrow checking, and semantic validation.
3. **First IR and verifier**

   Volume 13.

   Build IR 1, its graph infrastructure, memory SSA, effect dependencies, and graph verifier.
4. **Optimization architecture**

   Volume 14.

   Build heterogeneous representations, specialization, aggregate decomposition, escape analysis, and allocation elimination.
5. **Machine backend**

   Volumes 15–17.

   Build Machine IR, architecture descriptions, instruction selection, register allocation, scheduling, code emission, and object generation.
6. **Runtime and standard library**

   Volumes 18–19.

   Implement the execution environment and foundational APIs.
7. **Production toolchain**

   Volumes 20–21.

   Build package management, build automation, incremental compilation, debugging, formatting, testing, and documentation tooling.
8. **Correctness and production maturity**

   Volumes 22–26.

   Formal validation, performance engineering, security, release infrastructure, self-hosting, and optional execution products.

This is the dependency order for specifications, not a rule that all implementation must occur sequentially. Some implementation work can proceed in parallel once its interfaces and invariants have been defined.

# **42. What needs to exist before calling this a production compiler?**

I would establish concrete completion gates.

| Milestone                    | Required result                                                         |
| ---------------------------- | ----------------------------------------------------------------------- |
| Language definition complete | Every stabilized construct has normative semantics                      |
| Frontend complete            | Valid programs receive complete semantic validation                     |
| Borrow checker complete      | Ownership and lifetime invariants enforced                              |
| IR 1 complete                | Typed, effect-aware graph construction and verification                 |
| IR 2 complete                | Representation transformations verified                                 |
| Backend complete             | Correct executable generation for supported targets                     |
| Runtime complete             | Hosted and/or freestanding execution contracts work                     |
| Standard library complete    | Foundational APIs are usable and documented                             |
| Toolchain complete           | Build, debug, test, format, and package workflows work                  |
| Validation complete          | Conformance, fuzzing, regression, and differential suites established   |
| Production ready             | Release, security, compatibility, and maintenance processes established |

A compiler can be usable before every milestone is complete. But it should not claim support for a feature until its semantics and implementation are actually covered.

# **43. Decisions I would settle before writing the individual specifications**

There are some architectural choices that affect multiple volumes simultaneously.

These should be decided early.

| Decision                          | Affects                                 |
| --------------------------------- | --------------------------------------- |
| Exact undefined behavior model    | Memory, optimizer, unsafe, verification |
| Pointer provenance model          | Memory SSA, alias analysis, IR 2        |
| Object and storage identity       | Value semantics, allocation elimination |
| Default layout guarantees         | Type system, FFI, IR 2                  |
| Destruction and unwinding model   | Ownership, control flow, runtime        |
| Trait coherence rules             | Type checking, specialization           |
| Effect polymorphism               | Type system, function calls, IR 1       |
| Async and cancellation model      | Borrow checker, effects, runtime        |
| Generic specialization strategy   | Type system, IR 2, code generation      |
| ABI and calling convention policy | Machine IR, linker, FFI                 |
| Compiler verification target      | Every compiler stage                    |
| Bootstrap trust model             | Self-hosting and formal verification    |

These are architectural decisions, not minor details to postpone until implementation.
