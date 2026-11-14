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

Each chart has one row per person, and time runs from left to right. A box is something that person does, a purple box with a PR tag is a pull request, and a grey box is where the issue ends. The chips on an arrow show how the labels change as the issue passes to the next person: a chip in the label's own colour with + is added, and a red, struck-through chip with − is removed. Whichever row the latest arrow points into has the next turn.

`waiting-for-author` is added on top of the issue's status, so the issue keeps its place. When the author has answered, they say so on the issue and the maintainer removes the label. An issue with no reply for 30 days may be closed.

## How a bug moves

![A bug moves from the author, who files it, to the maintainer, who tries to reproduce it. The maintainer may add waiting-for-author and wait for the author's answer. If the bug is confirmed, any contributor opens a fix PR, which the maintainer merges; if the fix would change gameplay, a Proposal is filed instead. If it cannot be reproduced or is a duplicate, it is closed.](label-flows/bug.svg)

## How a proposal moves

![A proposal moves from the author, who files it, to the maintainer, who decides. The maintainer may add waiting-for-author and wait for the author's answer. If declined, it is closed as not planned. If accepted, any contributor opens a PR, which the maintainer merges. If blocked, it waits for the named issue and is then accepted.](label-flows/proposal.svg)

## How a question moves

![A question moves from the author to the maintainer or community, who answer it. They may add waiting-for-author and wait for more detail. Once answered it is closed as completed, and if the answer shows a bug or an idea, the author files a Bug or a Proposal.](label-flows/question.svg)

A question has no status label of its own.
