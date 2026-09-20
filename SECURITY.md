# Security Policy

The security and supply chain integrity of `rust-recursive-deps-updater` (`rrdu`) are paramount. This document outlines our security principles, supported versions, and the process for reporting vulnerabilities.

---

## Security Principles

- **Zero-Privilege Execution:** `rrdu` operates strictly within the user's privilege domain. It does not require, request, or attempt to acquire elevated (root or administrator) permissions on any operating system.
- **100% Safe Rust:** The codebase enforces `#![forbid(unsafe_code)]` at the crate root. No `unsafe` blocks, traits, or foreign function interfaces (FFI) are permitted.
- **Path Traversal Defense:** All paths read from configuration files (`.rrduconfig`) or user input are strictly validated. Directory traversal attempts (e.g. `../` or `/`) are rejected immediately with a controlled exit.
- **Static Operational Configuration:** `rrdu` forbids arbitrary CLI override flags to eliminate argument injection vulnerabilities. All configuration is bound to statically validated files or workflow definitions.
- **Encrypted Networking:** All communication with external registries (e.g. `crates.io`) and GitHub Releases uses HTTPS (TLS 1.2/1.3) with verified `rustls` backends, strict timeouts, and compliant User-Agent headers.

---

## Supported Versions

Only the latest release of `rrdu` receives active security patches.

| Version | Supported |
| ------- | --------- |
| 0.0.x   | Yes       |
| < 0.0.1 | No        |

For older releases you can request a security patch inside the Issues. But based on the request it could be closed as 'legacy version request'.

---

## Reporting a Vulnerability

If you discover a security vulnerability within this project, please report it privately:

1. **GitHub Security Advisory (Preferred):**  
   Open a private report via the [GitHub Security Advisory](https://github.com/AnnoDomine/rust-recursive-deps-updater/security/advisories/new) page.
2. **Email Disclosure:**  
   Alternatively, contact the maintainers directly via email. Please include detailed steps to reproduce the issue, proof of concept code, and potential impact.

### Disclosure Timeline
- We will acknowledge receipt of your report within 48 hours.
- We will provide a status update or assessment within 5 business days.
- Once a fix is verified, a patch release and public security advisory will be issued simultaneously.

Please do not open public issues or pull requests for undisclosed security vulnerabilities.
