# Security policy

## Supported versions

The project is developed on `main`, and security fixes land on `main` and in the
next release. There is no long-term-support branch.

## Reporting a vulnerability

**Please do not open a public issue for a security problem.** Use GitHub's
private vulnerability reporting: *Security* → *Advisories* → *Report a
vulnerability*.

Please include:

- what the defect is, and the command or input that triggers it;
- the affected version or commit (`git rev-parse --short HEAD`);
- the impact you believe it has;
- any suggested fix, if you have one.

You can expect an acknowledgement within a few days, and then either an
assessment with a remediation plan or a reasoned decision that the report is out
of scope.

## Scope

This is a compiler and a program-transformation engine, so the interesting
surface is **what it does with input it is given**:

- the lexer, parser and semantic checker, driven by an untrusted `.ref` source;
- the Refal machine, driven by an untrusted program;
- the driver and supercompiler, whose termination and resource use are bounded by
  an explicit step budget.

In scope:

- a defect that lets an input **escape the process it runs in** — a panic
  reachable from a valid program, an unbounded allocation or loop where a budget
  is promised, a file read or write outside the path it was given;
- a **wrong program** — a residue or a compiled artifact that does not agree with
  the source it came from. This project treats that as a correctness defect of
  the first order, and it is the one class of bug its gates exist to catch.

Out of scope: denial of service by handing the tool an intentionally enormous
input, and anything that requires an attacker to already control the machine.

## How the project already hardens itself

- **Secret scanning and push protection** are enabled for this repository.
- **Dependabot** watches the dependency graph and opens security updates.
- Every push to `main` runs `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, and the full corpus gates.
