# Labels

Every issue has a label for its kind, and bugs and proposals also have one for where they stand. The issue forms add the first ones, and the maintainer changes them after that. When an issue moves on, the maintainer replaces its status label rather than adding a new one, for example `proposal` → `accepted` or `needs-triage` → `confirmed`. The labels are defined in [labels.yml](labels.yml) and applied to GitHub with `tools/sync-labels.sh`.

## What each label means

| Label | Kind | Meaning | Set by |
| --- | --- | --- | --- |
| `bug` | kind | Something does not work as it should | Bug form |
| `enhancement` | kind | A new feature or a change | Proposal form |
| `question` | kind | A question for the maintainer or the community | Question form |
| `needs-triage` | bug status | The maintainer has not reproduced the bug yet | Bug form |
| `confirmed` | bug status | The maintainer reproduced the bug; a fix PR can be opened | maintainer |
| `proposal` | proposal status | Waiting for the maintainer to decide | Proposal form |
| `accepted` | proposal status | Agreed; a PR can be opened | maintainer |
| `declined` | proposal status | The maintainer decided not to do this; the issue is closed | maintainer |
| `blocked` | proposal status | Agreed, but waiting on another issue or decision named on the issue | maintainer |
| `waiting-for-author` | any | The maintainer asked a question or for changes; it is the author's turn | maintainer |

Duplicates and invalid issues have no labels of their own. They are closed with GitHub's close reasons, such as "duplicate" or "not planned".

## Whose turn it is

Each chart has one column per person, and time runs downward. An arrow is one person handing the issue to the next, with the label change it brings. Whoever the last arrow points to has the next turn.

`waiting-for-author` is added on top of the issue's status, so the issue keeps its place. When the author has answered, they say so on the issue and the maintainer removes the label. An issue with no reply for 30 days may be closed.

## How a bug moves

```mermaid
sequenceDiagram
  actor A as Author
  participant M as Maintainer
  participant C as Any contributor
  A->>M: Files a Bug (bug + needs-triage)
  opt More detail needed
    M->>A: Asks on the issue (+ waiting-for-author)
    A->>M: Answers (maintainer removes waiting-for-author)
  end
  M->>M: Tries to reproduce it
  alt Reproduced
    M->>C: needs-triage → confirmed
    C->>M: Opens a fix PR
    M->>M: Merges it, issue closes as completed
  else Cannot reproduce, or a duplicate
    M->>M: Closes it as not planned or duplicate
  end
  Note over C: If the fix would change gameplay, file a Proposal instead
```

## How a proposal moves

```mermaid
sequenceDiagram
  actor A as Author
  participant M as Maintainer
  participant C as Any contributor
  A->>M: Files a Proposal (enhancement + proposal)
  opt More detail needed
    M->>A: Asks on the issue (+ waiting-for-author)
    A->>M: Answers (maintainer removes waiting-for-author)
  end
  alt Agreed
    M->>C: proposal → accepted
  else Agreed, but depends on another issue
    M->>M: proposal → blocked, waits for the named issue
    M->>C: blocked → accepted, once unblocked
  else Not wanted
    M->>A: proposal → declined, closed as not planned
  end
  C->>M: Opens a PR (after accepted)
  M->>M: Merges it, issue closes as completed
```

## How a question moves

```mermaid
sequenceDiagram
  actor A as Author
  participant M as Maintainer or community
  A->>M: Asks a Question (question)
  opt More detail needed
    M->>A: Asks on the issue (+ waiting-for-author)
    A->>M: Answers (maintainer removes waiting-for-author)
  end
  M->>A: Answers, issue closes as completed
  opt The answer shows a bug or an idea
    A->>M: Files a Bug or a Proposal
  end
```

A question has no status label of its own.
