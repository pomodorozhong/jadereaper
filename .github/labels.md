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

Each chart below shows an issue's states as boxes. The colour of a box says who has to act next, and the arrows say what they do.

| Colour | Whose turn |
| --- | --- |
| Blue | The maintainer |
| Yellow | The author of the issue |
| Green | Any contributor, often the author |
| Grey | Nobody; the issue waits on something else, or it is closed |

`waiting-for-author` is added on top of the issue's status, so the issue keeps its place. When the author has answered, they say so on the issue and the maintainer removes the label. An issue with no reply for 30 days may be closed.

## How a bug moves

```mermaid
flowchart TD
  filed(["Author files a Bug"]) --> triage["needs-triage<br>Maintainer: reproduce it"]
  triage -- "maintainer needs more detail" --> wait["+ waiting-for-author<br>Author: answer on the issue"]
  wait -- "author answers" --> triage
  triage -- "reproduced" --> confirmed["confirmed<br>Contributor: open a fix PR"]
  triage -- "cannot reproduce, or duplicate" --> dropped(["Closed as not planned<br>or duplicate"])
  confirmed -- "fix would change gameplay" --> proposal(["Filed as a Proposal instead"])
  confirmed -- "fix PR merged" --> fixed(["Closed as completed"])

  classDef maintainer fill:#c5def5,stroke:#1f4f8a,color:#000
  classDef author fill:#fbca04,stroke:#8a6d00,color:#000
  classDef contributor fill:#a7e3b4,stroke:#1d6b2f,color:#000
  classDef idle fill:#ededed,stroke:#888,color:#000
  class triage maintainer
  class wait,filed author
  class confirmed contributor
  class dropped,proposal,fixed idle
```

## How a proposal moves

```mermaid
flowchart TD
  filed(["Author files a Proposal"]) --> proposal["proposal<br>Maintainer: decide"]
  proposal -- "maintainer asks a question or for changes" --> wait["+ waiting-for-author<br>Author: answer on the issue"]
  wait -- "author answers" --> proposal
  proposal -- "agrees" --> accepted["accepted<br>Contributor: open a PR"]
  proposal -- "agrees, but it depends on something" --> blocked["blocked<br>Nobody: waits on the named issue"]
  blocked -- "maintainer: it is unblocked" --> accepted
  proposal -- "declines" --> declined(["declined<br>Closed as not planned"])
  accepted -- "PR merged" --> done(["Closed as completed"])

  classDef maintainer fill:#c5def5,stroke:#1f4f8a,color:#000
  classDef author fill:#fbca04,stroke:#8a6d00,color:#000
  classDef contributor fill:#a7e3b4,stroke:#1d6b2f,color:#000
  classDef idle fill:#ededed,stroke:#888,color:#000
  class proposal maintainer
  class wait,filed author
  class accepted contributor
  class blocked,declined,done idle
```

## How a question moves

```mermaid
flowchart TD
  filed(["Author asks a Question"]) --> question["question<br>Maintainer or community: answer"]
  question -- "needs more detail" --> wait["+ waiting-for-author<br>Author: answer on the issue"]
  wait -- "author answers" --> question
  question -- "answered" --> closed(["Closed as completed"])
  question -. "answer shows a bug or an idea" .-> new(["Author files a Bug or Proposal"])

  classDef maintainer fill:#c5def5,stroke:#1f4f8a,color:#000
  classDef author fill:#fbca04,stroke:#8a6d00,color:#000
  classDef idle fill:#ededed,stroke:#888,color:#000
  class question maintainer
  class wait,filed,new author
  class closed idle
```

A question has no status label of its own.
