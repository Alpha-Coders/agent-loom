# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in AgentLoom, please report it responsibly.

**Do NOT open a public issue for security vulnerabilities.**

Instead, please email the maintainers directly or use GitHub's private vulnerability reporting feature.

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 1 week
- **Resolution**: Depends on severity

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.x.x   | Yes (current)      |

## Security Considerations

AgentLoom manages symlinks and file system operations. Key security areas:

- **Symlink handling**: Prevents symlink attacks and path traversal
- **File permissions**: Respects system file permissions
- **No network access**: Core functionality is entirely local

## Best Practices for Users

- Keep AgentLoom updated to the latest version
- Review skills before installing from untrusted sources
- Use standard OS security practices
