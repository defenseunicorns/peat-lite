# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in eche-lite, please report it responsibly. **Do not open a public GitHub issue for security vulnerabilities.**

### How to Report

You have two options:

1. **Email**: Send a detailed report to [security@defenseunicorns.com](mailto:security@defenseunicorns.com)
2. **GitHub Security Advisories**: Use the [private vulnerability reporting](https://github.com/defenseunicorns/eche-lite/security/advisories/new) feature on this repository

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment**: Within 3 business days
- **Initial assessment**: Within 10 business days
- **Fix timeline**: Dependent on severity

### Disclosure Policy

- We will acknowledge reporters in the remediation PR (unless anonymity is requested)
- We follow coordinated disclosure practices
- We aim to release patches before public disclosure

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | Yes       |

## Security Best Practices

eche-lite is designed for resource-constrained environments. When integrating:

- Validate all wire protocol inputs at trust boundaries
- Use the provided header codec rather than manual parsing
- Keep dependencies up to date
