# CrossCopy Examples

This directory contains example applications demonstrating various features and capabilities of CrossCopy.

## Available Examples

### 1. Basic Usage (`basic_usage.rs`)

Demonstrates the simplest way to set up and run CrossCopy with default configuration.

```bash
cargo run --example basic_usage
```

**What it shows:**
- Creating a CrossCopy application with default settings
- Starting the application
- Graceful shutdown handling
- Basic logging setup

### 2. Custom Configuration (`custom_config.rs`)

Shows how to create and use custom configurations with specific network, security, and clipboard settings.

```bash
cargo run --example custom_config
```

**What it shows:**
- Creating custom configuration objects
- Setting up peer connections
- Configuring security options
- Advanced logging configuration

### 3. Encryption Demo (`encryption_demo.rs`)

Demonstrates the encryption and decryption capabilities of CrossCopy.

```bash
cargo run --example encryption_demo
```

**What it shows:**
- Generating encryption keys
- Encrypting and decrypting clipboard content
- Password-based key derivation
- Security verification
- Performance testing of encryption

### 4. Network Demo (`network_demo.rs`)

Shows how to set up multiple CrossCopy instances that communicate over the network.

```bash
cargo run --example network_demo
```

**What it shows:**
- Creating multiple application instances
- Configuring peer-to-peer connections
- Network communication setup
- Connection management

### 5. Performance Metrics (`metrics_demo.rs`)

Demonstrates how to collect and analyze performance metrics from CrossCopy operations.

```bash
cargo run --example metrics_demo
```

**What it shows:**
- Performance metrics collection
- Timing operations
- Counter and gauge metrics
- Metrics analysis and reporting

### 6. Configuration Management (`config_management.rs`)

Shows comprehensive configuration file management capabilities.

```bash
cargo run --example config_management
```

**What it shows:**
- Loading and saving configuration files
- Configuration validation
- File watching for changes
- Different configuration scenarios

## Running Examples

### Prerequisites

Make sure you have Rust installed and the project dependencies are available:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build
```

### Running Individual Examples

Each example can be run independently:

```bash
# Run a specific example
cargo run --example <example_name>

# Run with debug logging
RUST_LOG=debug cargo run --example <example_name>

# Run with release optimizations
cargo run --release --example <example_name>
```

### Running with Features

Some examples may benefit from optional features:

```bash
# Run with compression support
cargo run --features compression --example encryption_demo

# Run with all features
cargo run --features "compression,gui" --example basic_usage
```

## Example Scenarios

### Development and Testing

1. **Start with Basic Usage**: Run `basic_usage` to understand the fundamental concepts
2. **Explore Configuration**: Use `custom_config` to learn about configuration options
3. **Test Security**: Run `encryption_demo` to understand security features
4. **Network Testing**: Use `network_demo` to test multi-device scenarios

### Production Setup

1. **Configuration Management**: Use `config_management` to set up proper configuration files
2. **Performance Monitoring**: Implement `metrics_demo` patterns for production monitoring
3. **Security Setup**: Apply lessons from `encryption_demo` for secure deployments

### Troubleshooting

- **Connection Issues**: Check `network_demo` for network configuration examples
- **Performance Problems**: Use `metrics_demo` to identify bottlenecks
- **Configuration Errors**: Refer to `config_management` for validation examples

## Example Output

Each example provides detailed logging output showing:
- Initialization steps
- Configuration details
- Operation results
- Performance metrics
- Error handling

## Customization

Feel free to modify these examples for your specific use cases:

1. **Change Ports**: Modify port numbers to avoid conflicts
2. **Add Features**: Extend examples with additional functionality
3. **Different Scenarios**: Create variations for your specific network setup
4. **Integration**: Use example code as a starting point for your applications

## Notes

- Examples use temporary files and configurations that are cleaned up automatically
- Network examples use localhost (127.0.0.1) for safety
- Some examples may require specific network permissions
- Clipboard access may be limited in certain environments (CI/CD, headless systems)

## Getting Help

If you encounter issues with the examples:

1. Check the main project documentation
2. Review the example source code for comments
3. Enable debug logging with `RUST_LOG=debug`
4. Ensure all dependencies are properly installed

For more information, see the main project README and documentation.
