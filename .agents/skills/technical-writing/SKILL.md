---
name: technical-writing
description: Read this before you write or edit any file that contains prose — including .md files, documentation, templates, prompts, commit messages, comments, or any text meant to be read by a human. Do not skip because you think the task is about "structure" or "design" rather than "writing". If the output contains sentences, this skill applies.
---

# Orwell Writing

## Overview

Use Orwell's rules and ASD-STE100 Simplified Technical English (STE) as practical filters for clear, direct, and honest prose. Use STE by default for technical, instructional, business, and product prose. Apply the rules to both drafting and revision, but do not erase deliberate voice, character, rhythm, humor, or genre when the user clearly wants them.

STE has writing rules and a controlled dictionary. Use an approved word with its approved meaning when the dictionary is available. Do not claim strict STE conformance without checking the current ASD-STE100 issue and dictionary.

## Core Rules

Remember these rules from "Politics and the English Language":

1. Never use a metaphor, simile, or other figure of speech which you are used to seeing in print.
2. Never use a long word where a short one will do.
3. If it is possible to cut a word out, always cut it out.
4. Never use the passive where you can use the active.
5. Never use a foreign phrase, a scientific word, or a jargon word if you can think of an everyday English equivalent.
6. Break any of these rules sooner than say anything outright barbarous.

## ASD-STE100 baseline

For technical and instructional prose:

1. Use short sentences. Put one main action or statement in each sentence.
2. Use a clear subject and an active verb. Name the actor when the actor matters.
3. Use the same term for the same thing. Do not change a term only to avoid repetition.
4. Use familiar words with one precise meaning. Avoid idioms, slang, figurative language, and vague verbs.
5. Use a specific technical term when it is necessary for accuracy. Define it or link to its definition.
6. Keep noun groups short. Use prepositions to show relationships between terms.
7. Write procedures as direct instructions. State the condition, action, and expected result.
8. Use positive instructions when they are clear. State what the reader must do.
9. Use consistent American English spelling unless the user's style guide requires another variety.
10. Preserve code, commands, identifiers, product names, legal text, and required quotations. Do not simplify them silently.

When strict STE is not possible, keep the text clear and mark the terms or passages that need a domain-specific exception.

## Markdown formatting

For any Markdown file:

1. Do not insert hard line breaks in the middle of a sentence. Let each sentence run to its natural end on one line. Formatters and linters handle display wrapping.
2. Use blank lines between paragraphs and between block-level elements (headings, lists, code blocks, blockquotes).
3. Keep list items on their own line. Break a long list item only at a natural clause boundary, not arbitrarily.

**Good and bad examples:**

```markdown
# Bad — hard break in the middle of a sentence
Before you reach for a library, a pattern, or even a
function, run the ladder.

# Good — sentence runs to its natural end on one line
Before you reach for a library, a pattern, or even a function, run the ladder.
```

```markdown
# Bad — list item broken mid-clause
- When you accept a shortcut on purpose, name the limit
  and the upgrade trigger.

# Good — break only at a clause boundary
- When you accept a shortcut on purpose, name the limit and the
  upgrade trigger.
# Also good — short enough to keep on one line
- The platform is free. Use it.
```

```markdown
# Bad — no blank line between code block and surrounding text
To run the ladder:
```
# ponytail: global lock on cache writes
```
The ponytail is not a TODO.

# Good — blank line before and after code block
To run the ladder:

```
# ponytail: global lock on cache writes
```

The ponytail is not a TODO.
```

The rule applies to all Markdown files, including AGENTS.md, prompt templates, wiki documents, and skill files.

## Workflow

When writing from scratch:

1. Identify the audience, purpose, and promised tone from the user's request.
2. Draft in concrete, direct English.
3. Remove stock phrases, dead metaphors, filler, pompous diction, needless abstraction, and avoidable jargon.
4. Prefer active verbs and clear subjects unless passive voice better serves emphasis, tact, suspense, or technical accuracy.
5. Keep necessary nuance; do not make prose crude, false, or flat just to make it short.
6. Apply the ASD-STE100 baseline. Check terms, sentence structure, instructions, and technical exceptions.

When revising existing text:

1. Preserve the user's meaning and any explicit tone or format constraints.
2. Cut words, clauses, and sentences that do no work.
3. Replace stale figures of speech with plain phrasing or a fresh, specific image.
4. Replace long, foreign, scientific, or jargon terms with everyday English when accuracy permits.
5. Convert passive constructions to active ones when the actor matters and is known.
6. Flag any remaining jargon, passive voice, or ornate phrasing that is necessary rather than silently removing important precision.
7. Run a final STE pass. Check that each technical term is consistent, each instruction states the required action, and each exception is intentional.

## Creative Writing

For fiction, poetry, memoir, scripts, and lyrical prose, treat STE as a clarity aid, not a requirement that overrides the user's form. Keep intentional ambiguity, cadence, dialogue style, imagery, and character voice when they create a real effect. Remove only language that feels inherited, inflated, evasive, or lazy. Use strict STE when the user explicitly requests it, and note when that request conflicts with a creative effect.
