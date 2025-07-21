# Twine Dependency Management Scripts

This directory contains scripts for managing local dependency sources and documentation to enable AI agents to access complete, accurate dependency information.

## Scripts

### `setup-deps.sh`
Initial setup script that creates the dependency management infrastructure.

**Usage:**
```bash
./scripts/setup-deps.sh
```

**What it does:**
- Creates `deps/vendor/`, `deps/docs/`, and `deps/registry/` directories
- Downloads all dependency sources using `cargo vendor`
- Generates comprehensive documentation for all dependencies
- Copies documentation to the local `deps/docs/` directory
- Verifies the setup was successful

**When to use:**
- First time setting up the project
- After cloning the repository
- When the `deps/` directory gets corrupted or deleted

### `update-deps.sh`
Maintenance script that updates dependency sources and documentation when dependencies change.

**Usage:**
```bash
./scripts/update-deps.sh
```

**What it does:**
- Updates vendored sources to match current `Cargo.toml`
- Cleans old documentation
- Regenerates documentation for all dependencies
- Copies updated documentation to `deps/docs/`
- Verifies the update was successful

**When to use:**
- After adding new dependencies to `Cargo.toml`
- After updating existing dependency versions
- When dependency documentation becomes outdated

## Directory Structure

After running the setup script, the following structure will be created:

```
deps/
├── vendor/          # Vendored dependency source code
│   ├── thiserror/   # Source for thiserror crate
│   ├── proc-macro2/ # Source for proc-macro2 crate
│   └── ...          # Additional dependency sources
├── docs/            # Generated documentation for all dependencies
│   ├── thiserror/   # Documentation for thiserror
│   ├── twine_scheme/ # Documentation for our project
│   └── ...          # Additional dependency docs
└── registry/        # Local registry cache (for future use)
```

## Benefits for AI Development

- **Complete Source Access**: AI agents can reference exact dependency source code
- **Comprehensive Documentation**: Includes private items and implementation details
- **Version Consistency**: Sources are locked to versions in `Cargo.lock`
- **Offline Access**: No network dependency for code analysis
- **Accurate Context**: Eliminates guesswork about dependency behavior

## Notes

- The `deps/` directory is automatically excluded from git via `.gitignore`
- Scripts require bash and standard Unix tools (`cp`, `rm`, `mkdir`)
- Documentation generation can take a few minutes for large dependency trees
- Both scripts include verification steps and will exit on error