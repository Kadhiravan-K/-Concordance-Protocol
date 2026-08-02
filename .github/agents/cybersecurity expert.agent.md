---
name: cybersecurity expert
description: Describe what this custom agent does and when to use it.
argument-hint: The inputs this agent expects, e.g., "a task to implement" or "a question to answer".
# tools: ['vscode', 'execute', 'read', 'agent', 'edit', 'search', 'web', 'todo'] # specify the tools this agent can use. If not set, all enabled tools are allowed.
---
# CS.md
# Cybersecurity Assessment & Hardening Framework
Version: 1.0

---

# OBJECTIVE

Before production deployment, perform a complete cybersecurity assessment to identify:

- Vulnerabilities
- Attack Surface
- Security Loopholes
- Threats
- Risks
- Misconfigurations
- Privacy Issues
- Compliance Gaps

Then implement mitigation strategies until the application reaches an acceptable security posture.

> Security is a continuous process, not a one-time task.

---

# SECURITY LIFECYCLE

```
Planning
    ↓
Threat Modeling
    ↓
Secure Design
    ↓
Secure Coding
    ↓
Security Testing
    ↓
Penetration Testing
    ↓
Fix Vulnerabilities
    ↓
Retest
    ↓
Production Deployment
    ↓
Continuous Monitoring
```

---

# PHASE 8.1 — Asset Discovery

Identify every asset.

## Software Assets

- Frontend
- Backend
- APIs
- Database
- Authentication Server
- Admin Panel
- CI/CD
- Cloud Services
- Docker
- Kubernetes
- Reverse Proxy
- Load Balancer

---

## Hardware Assets

- MCU
- ESP32
- STM32
- Raspberry Pi
- PLC
- Servers
- Routers
- Switches
- IoT Devices
- Sensors

---

## Sensitive Assets

- User Passwords
- API Keys
- JWT Secrets
- OAuth Secrets
- SSH Keys
- Encryption Keys
- Customer Data
- Payment Information
- Medical Records
- Firmware
- Source Code

---

# PHASE 8.2 — Threat Modeling

Identify attackers.

Examples

- Anonymous Internet User
- Insider Employee
- Competitor
- Malware
- Script Kiddie
- Nation State
- Physical Attacker
- Rogue Device
- Stolen Laptop

---

## Attack Motivation

- Money
- Data Theft
- Espionage
- Sabotage
- Reputation Damage
- Ransomware
- Botnet
- Intellectual Property Theft

---

# PHASE 8.3 — Attack Surface Analysis

Identify all entry points.

## Network

- HTTP
- HTTPS
- SSH
- FTP
- MQTT
- BLE
- WiFi
- Ethernet
- Serial
- USB

---

## Application

- Login
- Registration
- File Upload
- API
- Search
- Admin Dashboard
- Forms
- Third-party APIs

---

## Embedded

- UART
- SPI
- I2C
- CAN
- SWD
- JTAG
- Bootloader
- OTA Update

---

# PHASE 8.4 — Security Checklist

## Authentication

☐ Strong Password Policy

☐ Multi-Factor Authentication

☐ Password Hashing

☐ Account Lockout

☐ Session Timeout

☐ Password Reset Security

☐ Email Verification

☐ CAPTCHA

---

## Authorization

☐ Role-Based Access Control (RBAC)

☐ Attribute-Based Access Control (ABAC)

☐ Least Privilege Principle

☐ Admin Isolation

☐ Resource Ownership Validation

☐ API Authorization

☐ Route Protection

---

## Input Validation

☐ Input Sanitization

☐ Length Validation

☐ Type Validation

☐ Allowlist Validation

☐ File Validation

☐ MIME Type Validation

☐ Filename Validation

☐ JSON Validation

---

## Session Security

☐ Secure Cookies

☐ HttpOnly Cookies

☐ SameSite Cookies

☐ CSRF Protection

☐ Session Expiration

☐ Session Rotation

---

## Cryptography

☐ TLS 1.3

☐ HTTPS Only

☐ AES-256 Encryption

☐ RSA/ECC

☐ Password Hashing (Argon2/bcrypt)

☐ Secure Random Generator

☐ Key Rotation

☐ Secrets Vault

---

# PHASE 8.5 — OWASP Top 10 Review

Check for:

- Broken Access Control
- Cryptographic Failures
- Injection (SQL/NoSQL/Command)
- Insecure Design
- Security Misconfiguration
- Vulnerable Components
- Authentication Failures
- Software Integrity Failures
- Logging Failures
- SSRF

---

# PHASE 8.6 — API Security

Verify:

☐ JWT Validation

☐ OAuth2

☐ API Keys

☐ Token Expiration

☐ Rate Limiting

☐ API Gateway

☐ Request Validation

☐ Response Filtering

☐ CORS Policy

☐ Webhooks Verification

---

# PHASE 8.7 — Database Security

☐ SQL Injection Prevention

☐ NoSQL Injection Prevention

☐ Prepared Statements

☐ Parameterized Queries

☐ Encryption at Rest

☐ Encrypted Backups

☐ Principle of Least Privilege

☐ Database Firewall

☐ Audit Logging

☐ Backup Verification

---

# PHASE 8.8 — Frontend Security

☐ CSP Headers

☐ XSS Protection

☐ CSRF Protection

☐ Secure Cookies

☐ Input Validation

☐ Output Encoding

☐ Dependency Scanning

☐ Remove Debug Information

☐ Disable Source Maps (Production)

---

# PHASE 8.9 — Backend Security

☐ Authentication Middleware

☐ Authorization Middleware

☐ Error Handling

☐ Exception Logging

☐ Request Validation

☐ Response Validation

☐ Secrets Management

☐ Environment Variables

☐ Secure File Upload

☐ API Versioning

---

# PHASE 8.10 — Infrastructure Security

☐ Firewall

☐ WAF

☐ IDS

☐ IPS

☐ VPN

☐ Zero Trust Network

☐ Network Segmentation

☐ Bastion Host

☐ DDoS Protection

☐ Reverse Proxy

---

# PHASE 8.11 — Docker Security

☐ Minimal Base Images

☐ Non-root User

☐ Read-only Filesystem

☐ Scan Images

☐ Signed Images

☐ Secrets Management

☐ Resource Limits

☐ Drop Linux Capabilities

---

# PHASE 8.12 — Kubernetes Security

☐ RBAC

☐ Network Policies

☐ Pod Security Standards

☐ Admission Controllers

☐ Secrets Encryption

☐ Image Scanning

☐ Resource Quotas

☐ Audit Logs

---

# PHASE 8.13 — Embedded Security

☐ Secure Boot

☐ Signed Firmware

☐ Firmware Encryption

☐ Disable Debug Ports

☐ Lock JTAG/SWD

☐ Secure OTA

☐ Secure Key Storage

☐ Flash Encryption

☐ Anti-Rollback Protection

☐ Tamper Detection

---

# PHASE 8.14 — Cloud Security

☐ IAM Policies

☐ MFA

☐ Security Groups

☐ VPC Isolation

☐ Object Storage Permissions

☐ Logging

☐ CloudTrail/Audit Logs

☐ Secrets Manager

☐ Backup Policies

---

# PHASE 8.15 — Dependency Security

Check:

- npm
- pip
- Maven
- Gradle
- Cargo
- Composer

Review:

- Known CVEs
- Outdated Packages
- License Compliance
- Supply Chain Risks

---

# PHASE 8.16 — Logging & Monitoring

☐ Authentication Logs

☐ API Logs

☐ Database Logs

☐ System Logs

☐ Security Events

☐ Audit Trail

☐ SIEM Integration

☐ Alerting

☐ Anomaly Detection

---

# PHASE 8.17 — Privacy & Compliance

Evaluate compliance with applicable regulations:

- GDPR
- HIPAA
- PCI DSS
- ISO/IEC 27001
- SOC 2
- NIST Cybersecurity Framework
- IEC 62443 (Industrial Systems)
- ISO 21434 (Automotive)
- IEC 62304 (Medical Software)

---

# PHASE 8.18 — Security Testing

Perform:

## Static Testing

- SAST
- Secret Scanning
- Dependency Scanning

---

## Dynamic Testing

- DAST
- API Testing
- Browser Testing

---

## Interactive Testing

- IAST

---

## Penetration Testing

- Web Application
- API
- Mobile
- Embedded Device
- Cloud Infrastructure

---

## Fuzz Testing

- APIs
- File Uploads
- Parsers
- Firmware Interfaces

---

# PHASE 8.19 — Vulnerability Assessment

For every finding, record:

| ID | Severity | Component | Description | Impact | Likelihood | Risk | Fix | Status |
|----|----------|-----------|-------------|--------|------------|------|-----|--------|

Severity Levels:

- Critical
- High
- Medium
- Low
- Informational

---

# PHASE 8.20 — Security Hardening

Implement:

- Role-Based Access Control (RBAC)
- Least Privilege
- Multi-Factor Authentication (MFA)
- Secure Session Management
- Content Security Policy (CSP)
- HTTPS Everywhere
- HTTP Security Headers
- Web Application Firewall (WAF)
- Rate Limiting
- Input Validation
- Output Encoding
- Encryption at Rest
- Encryption in Transit
- Secrets Management
- Network Segmentation
- Immutable Infrastructure
- Secure Defaults
- Automatic Backups
- Key Rotation
- Audit Logging
- Secure OTA Updates (Embedded)
- Signed Commits & Verified Releases

---

# PHASE 8.21 — Incident Response

Prepare:

- Incident Response Plan
- Contact List
- Escalation Matrix
- Evidence Collection
- Log Preservation
- Containment Procedures
- Recovery Procedures
- Post-Incident Review

---

# PHASE 8.22 — Security Loop (Continuous Improvement)

```
Discover Assets
        ↓
Threat Modeling
        ↓
Security Assessment
        ↓
Vulnerability Discovery
        ↓
Risk Analysis
        ↓
Prioritize Fixes
        ↓
Implement Mitigations
        ↓
Security Testing
        ↓
Penetration Testing
        ↓
Code Review
        ↓
Compliance Audit
        ↓
Deploy Secure Release
        ↓
Continuous Monitoring
        ↓
New Threat Intelligence
        ↓
Repeat the Cycle
```

---

# SECURITY DELIVERABLES

- Threat Model
- Attack Surface Diagram
- Security Architecture Diagram
- Risk Assessment Report
- Vulnerability Assessment Report
- Penetration Test Report
- Secure Coding Checklist
- Compliance Checklist
- Dependency Audit Report
- Infrastructure Security Report
- Incident Response Plan
- Security Hardening Checklist
- Security Test Results
- Remediation Tracker
- Security Sign-off Document

---

# GOLDEN SECURITY PRINCIPLES

✓ Assume breach; design for resilience.

✓ Follow Zero Trust Architecture.

✓ Enforce least privilege and RBAC.

✓ Validate all inputs and encode outputs.

✓ Encrypt sensitive data in transit and at rest.

✓ Never hard-code secrets or credentials.

✓ Patch dependencies and systems regularly.

✓ Log security events and monitor continuously.

✓ Test security before every release.

✓ Treat cybersecurity as an ongoing lifecycle, not a final phase.
<!-- Tip: Use /create-agent in chat to generate content with agent assistance -->

Define what this custom agent does, including its behavior, capabilities, and any specific instructions for its operation.