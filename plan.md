# Plan: Introduce QuoteFetcherConfig Trait

## Overview

The goal of this plan is to introduce a new Rust trait, `QuoteFetcherConfig`, which will provide a method `should_fetch_quotes(&self) -> bool`. This trait will be implemented on various configuration structs (`RuleConfig`, `ForConfig`, `WhenConfig`, `DoConfig`, `TacticConfig`) to determine if financial quotes need to be fetched based on their specific configurations.

## Assumptions

1.  **Trait Definition Location**: The `QuoteFetcherConfig` trait will be defined in a new file: `src/config/quote_fetcher_config.rs`.
2.  **`RuleConfig` Logic**: The implementation for `RuleConfig` will return `true` if any of its relevant properties indicate that quotes are needed. This implies that these properties either implement `QuoteFetcherConfig` themselves or can be directly evaluated to a boolean for this purpose. The exact fields of `RuleConfig` will need to be inspected for the precise implementation.
3.  **`TacticConfig` Logic**: The implementation for `TacticConfig` will return `true` if any of its contained `RuleConfig` instances return `true` from their `should_fetch_quotes` method. This assumes `TacticConfig` holds a collection or references to one or more `RuleConfig`s.
4.  **Enum Logic (`ForConfig`, `WhenConfig`, `DoConfig`)**: These are enums. Their `should_fetch_quotes` implementation will depend on the semantics of their variants.
    *   `DoConfig`: Implementation will likely involve a match statement. Certain actions might inherently require quotes.
    *   `WhenConfig`: Implementation will likely involve a match statement. Certain conditions might depend on quoted values.
    *   `ForConfig`: Implementation will likely involve a match statement. Certain targets or scopes might involve quotes.
    If a variant wraps data, that data might also need to implement `QuoteFetcherConfig`.
5.  **Module Structure**: The relevant configuration files are located under `src/config/` and `src/config/tactic/`.

## File Structure Information

The primary files involved in this work are expected to be:

```
src/
└── config/
    ├── mod.rs
    ├── quote_fetcher_config.rs # New file for the trait definition (Task 1)
    ├── tactic/
    │   ├── mod.rs
    │   ├── do.rs              # For DoConfig implementation (Task 2)
    │   ├── when.rs            # For WhenConfig implementation (Task 3)
    │   ├── for.rs             # For ForConfig implementation (Task 4)
    │   ├── rule.rs            # For RuleConfig implementation (Task 5)
    │   └── tactic.rs          # For TacticConfig implementation (Task 6)
    └── ... (other config files)
```

## Task List

- [ ] **Task 1: Define the `QuoteFetcherConfig` trait.**
    - Create `src/config/quote_fetcher_config.rs`.
    - Add the following code:
      ```rust
      pub trait QuoteFetcherConfig {
          fn should_fetch_quotes(&self) -> bool;
      }
      ```
- [ ] **Task 2: Implement `QuoteFetcherConfig` for `DoConfig`.**
    - Open `src/config/tactic/do.rs`.
    - Import the trait: `use crate::config::quote_fetcher_config::QuoteFetcherConfig;`.
    - Implement `impl QuoteFetcherConfig for DoConfig { ... }`. Logic will depend on enum variants. (e.g., return `true` for variants that imply actions on market data).
- [ ] **Task 3: Implement `QuoteFetcherConfig` for `WhenConfig`.**
    - Open `src/config/tactic/when.rs`.
    - Import the trait: `use crate::config::quote_fetcher_config::QuoteFetcherConfig;`.
    - Implement `impl QuoteFetcherConfig for WhenConfig { ... }`. Logic will depend on enum variants. (e.g., return `true` if a condition involves price checks).
- [ ] **Task 4: Implement `QuoteFetcherConfig` for `ForConfig`.**
    - Open `src/config/tactic/for.rs`.
    - Import the trait: `use crate::config::quote_fetcher_config::QuoteFetcherConfig;`.
    - Implement `impl QuoteFetcherConfig for ForConfig { ... }`. Logic will depend on enum variants. (e.g., return `true` if the 'for' target requires live data).
- [ ] **Task 5: Implement `QuoteFetcherConfig` for `RuleConfig`.**
    - Open `src/config/tactic/rule.rs`.
    - Import the trait: `use crate::config::quote_fetcher_config::QuoteFetcherConfig;`.
    - Implement `impl QuoteFetcherConfig for RuleConfig { ... }`. This will involve checking its fields.
        - Example logic:
          ```rust
          // Assuming RuleConfig has fields like:
          // when_config: Option<WhenConfig>,
          // do_config: Option<DoConfig>,
          // for_config: Option<ForConfig>,
          // (and potentially other fields that might implement QuoteFetcherConfig or be booleans)
          //
          // fn should_fetch_quotes(&self) -> bool {
          //     self.when_config.as_ref().map_or(false, |c| c.should_fetch_quotes()) ||
          //     self.do_config.as_ref().map_or(false, |c| c.should_fetch_quotes()) ||
          //     self.for_config.as_ref().map_or(false, |c| c.should_fetch_quotes()) // ||
          //     // ... other conditions based on other fields
          // }
          ```
        - The actual implementation will depend on the fields of `RuleConfig`.
- [ ] **Task 6: Implement `QuoteFetcherConfig` for `TacticConfig`.**
    - Open `src/config/tactic.rs`.
    - Import the trait: `use crate::config::quote_fetcher_config::QuoteFetcherConfig;`.
    - Implement `impl QuoteFetcherConfig for TacticConfig { ... }`.
        - Example logic (assuming `TacticConfig` has a field `rules: Vec<RuleConfig>`):
          ```rust
          // fn should_fetch_quotes(&self) -> bool {
          //     self.rules.iter().any(|rule| rule.should_fetch_quotes())
          // }
          ```
        - The actual implementation will depend on how `TacticConfig` stores its `RuleConfig` instances.
- [ ] **Task 7: Add necessary imports and derive macros.**
    - Ensure all necessary `use` statements are present.
    - Add `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]` or similar macros to the trait or implementations if they become necessary due to trait bounds or usage, though likely not needed for the trait itself.
- [ ] **Task 8: Review and test.**
    - Compile the project to ensure all changes are valid.
    - Write unit tests for each implementation of `QuoteFetcherConfig` to verify correct behavior.
- [ ] **Task 9: Delete this task list (`plan.md`).** 