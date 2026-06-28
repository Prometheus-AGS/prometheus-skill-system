---
id: change-learn-014
title: "learn-certify skill"
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-009
  - change-learn-005
  - change-learn-012
  - change-learn-013
  - change-learn-006
---

# change-learn-014: learn-certify skill

## Problem

Users who complete a learning journey have no portable proof of demonstrated
mastery. Without a verifiable credential, the learning record is local and
non-transferable.

## Proposal

Implement `skills/learn/learn-certify/SKILL.md` with `--checkpoint` (milestone
check) and `--final` (full certification) modes. Final mode requires prerequisite
gates (N feynman-artifacts, M practice results, capstone completion). The skill
emits an OB 3.0 / W3C VC self-issued JSON-LD signed with did-plc, with integrity
guardrails that flag anomalous step-change trajectories. An `--issuer` parameter
enables forwarding to a 1EdTech endpoint.

## Outcome

A credentialing endpoint that turns demonstrated learner-model mastery into a
portable, verifiable open badge or W3C VC.
