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

## How a bug moves

```mermaid
flowchart LR
  opened["Opened<br>bug + needs-triage"] --> confirmed["confirmed"]
  confirmed --> pr["Fix PR"]
  pr --> fixed["Closed as completed"]
  opened --> notplanned["Closed as not planned<br>or duplicate"]
  opened -. fix changes gameplay .-> proposal["Becomes a proposal"]
```

A bug fix can go straight to a PR once the bug is `confirmed`. If the fix would change how the game plays, it is handled as a proposal instead.

## How a proposal moves

```mermaid
flowchart LR
  opened["Opened<br>enhancement + proposal"] --> accepted["accepted"]
  accepted --> pr["PR"]
  pr --> done["Closed as completed"]
  opened --> declined["declined<br>closed as not planned"]
  opened --> blocked["blocked"]
  blocked -- unblocked --> accepted
```

## How a question moves

```mermaid
flowchart LR
  opened["Opened<br>question"] --> answered["Answered"]
  answered --> closed["Closed as completed"]
  answered -. leads to .-> new["New bug or proposal"]
```

A question has no status label. If the answer shows a bug or an idea worth doing, it gets its own Bug or Proposal issue.

## Waiting for the author

```mermaid
flowchart LR
  state["Any status"] -- maintainer asks --> waiting["+ waiting-for-author"]
  waiting -- author answers on the issue --> removed["Maintainer removes the label"]
  removed --> state
  waiting -. no reply for 30 days .-> closed["May be closed"]
```

`waiting-for-author` can be added to any issue, including a question, on top of its status, so the issue keeps its place in the flow. When the author has answered, they say so on the issue and the maintainer removes the label. An issue with no reply for 30 days may be closed.
