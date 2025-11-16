# Contributing to Librefy

We love your input! We want to make contributing to Librefy as easy and transparent as possible.

## How to Contribute

### Reporting Bugs
1. **Check Existing Issues** - Ensure the bug hasn't already been reported
2. **Create New Issue** - Use the bug report template
3. **Describe Clearly** - Include steps to reproduce, expected vs actual behavior

### Suggesting Features
1. **Check Existing Issues** - See if the feature has already been suggested
2. **Create New Issue** - Use the feature request template
3. **Explain Why** - Describe the use case and benefits

### Code Contributions
1. **Fork the Repository**
2. **Create a Feature Branch** - `git checkout -b feature/amazing-feature`
3. **Make Your Changes**
4. **Test Your Changes** - Ensure everything works
5. **Commit Your Changes** - `git commit -m 'Add amazing feature'`
6. **Push to Branch** - `git push origin feature/amazing-feature`
7. **Open a Pull Request**

## Development Setup

### Prerequisites
- Rust 1.70+
- GTK4 development libraries
- GStreamer

### Build Instructions
```bash
git clone https://github.com/Librefy/Librefy.git
cd Librefy
cargo build
cargo run
```

## Code Style

• Follow Rust conventions
• Use *rustfmt* for formatting
• Use *clippy* for linting
• Write meaningful commit messages

## Project Structure
```bash
src/
├── main.rs          # Application entry point
├── ui/              # GTK4 interface components
├── player/          # Audio playback engine
├── library/         # Music library management
└── config/          # Settings and preferences
```

## Pull Request Process 

1. Update README.md if needed
2. Add tests for new functionality
3. Ensure all tests pass
4. Request review from maintainers (no maintainers available lol)

## Questions?

Feel free to open an issue if any questions about contributing!
Thank you for contributing to Librefy! 🎵

