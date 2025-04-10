---
title: "Continuous Deployment for Personal CLI Tools"
datePublished: Tue Mar 25 2025 13:37:07 GMT+0000 (Coordinated Universal Time)
cuid: cm8ojidpl000609jyexmoa0wi
slug: continuous-deployment-for-personal-cli-tools
tags: git, cli, development-experience

---

Git pre-push hooks can automate local installation of your personal CLI tools, creating a development workflow where your working environment always has the latest version of your code.

When developing CLI tools, for example for personal use, I often want to run the latest version locally to quickly identify issues and get a feel for changes. Constantly installing updates manually or running from source creates unnecessary friction. This article shows how to automate this process using Git pre-push hooks.

## A Simple Solution: Local Continuous Deployment

Automatically installing an updated CLI command locally whenever you push changes to your repository creates a featherweight continuous deployment pipeline. This keeps your local environment in sync with your latest code changes with minimal effort.

## What Are Git Hooks?

Git hooks are scripts that Git executes before or after events like commit, push, or receive. Pre-push hooks run automatically before your code is pushed to a remote repository.

## Implementation: Git Pre-Push Hooks

The implementation is straightforward. Use git's pre-push hooks to trigger installation automatically:

1. Edit the git `pre-push` hook file:
    

```bash
vim .git/hooks/pre-push
```

2. Make it executable:
    

```bash
chmod +x .git/hooks/pre-push
```

3. Add installation commands to the hook:
    

```bash
#!/bin/sh
install.sh
exit $? # Propagate the exit code
```

The [`install.sh`](http://install.sh) script in this example should contain whatever commands you would normally use to install your CLI tool locally. If you use build tools like `make` or `just`, you can call those commands directly.

For a tool written in Rust your [`install.sh`](http://install.sh) script may contain:

```bash
cargo install --path .
```

This approach automatically rebuilds and installs your application on every push to the repository and ensures you're always using the latest version without any manual steps.

Note:

* This works best when developing directly on the main branch. If you work with feature branches, you'll need to consider which version you want running locally at any given time.
    
* For added robustness, you can extend this approach to run tests or lint your code before installation.
    
* Hooks are not pushed to your repository by default. You will need to set them up on each development machine.
    

Thank you for reading, Hans