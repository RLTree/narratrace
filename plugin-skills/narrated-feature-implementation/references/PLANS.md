---
last_edited: 2026-06-23
---

# ExecPlan Guide

Use this guide when a narrated feature implementation needs a durable plan before code changes.

## When To Write A Plan

Use a compact inline plan for trivial changes only when all of these are true:

- the requested behavior is unambiguous after packet review;
- the implementation touches one small area;
- no user-visible workflow, accessibility, data, privacy, or persistence behavior is at stake;
- the validation path is obvious.

Write an ExecPlan file for non-trivial work, including:

- UI behavior with multiple states;
- product or interaction ambiguity;
- cross-file implementation;
- new persistence, API, schema, permission, or privacy behavior;
- low-confidence transcript/action alignment;
- any change that should survive interruption or handoff.

## Required ExecPlan Content

Each ExecPlan must include:

- purpose and observable outcome;
- evidence packet and transcript/action confidence;
- current observed behavior;
- desired behavior;
- decisions, assumptions, and targeted user answers;
- owned paths and forbidden paths;
- implementation units with repo-relative paths;
- exact validation commands;
- proof surface for every positive claim;
- privacy, redaction, and local-artifact handling;
- recovery, rollback, and residual claim ceiling.

## Targeted Questions

Ask targeted questions only after reviewing the recording packet and the relevant code. Ask only questions that change implementation, such as:

- which visible target the user meant by a pointing phrase;
- whether behavior should be preserved or changed;
- which edge state is in scope;
- which acceptance criterion closes the work;
- how to resolve conflict between transcript, screen evidence, and code behavior.

Record each answer in the plan or ExecPlan before binding to it.

## Goal Binding

After the compact plan or ExecPlan is finalized, bind the current agent to that contract with the available goal tool. Use `set_goal()` where that is the exposed API, or the equivalent harness goal-binding tool.

The goal text must name:

- the plan or ExecPlan path, or the inline compact plan;
- the implementation outcome;
- the validation command set;
- the claim ceiling.

Do not bind before blocking questions are answered or explicit assumptions are written into the plan.

## Completion

After goal binding, implement against the plan until completion or a real blocker. Keep the plan current when implementation discovery changes scope, files, risks, or validation. Do not silently outrun the plan.
