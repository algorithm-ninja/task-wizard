import { Component, Input, ViewEncapsulation } from '@angular/core';
import { ColDef } from 'ag-grid-community';
import gql from 'graphql-tag';
import {
  ContestProblemAssignmentUserTacklingSubmissionListColumnFragment,
  ContestProblemAssignmentUserTacklingSubmissionListFragment,
  ContestProblemAssignmentUserTacklingSubmissionListSubmissionFragment,
} from '../generated/graphql-types';
import { textFragment } from './material/text.pipe';
import { submissionModalFragment } from './submission-modal.component';

@Component({
  selector: 'app-contest-problem-assignment-user-tackling-submission-list',
  templateUrl: './contest-problem-assignment-user-tackling-submission-list.component.html',
  styleUrls: ['./contest-problem-assignment-user-tackling-submission-list.component.scss'],
  encapsulation: ViewEncapsulation.None,
})
export class ContestProblemAssignmentUserTacklingSubmissionListComponent {
  @Input()
  data!: ContestProblemAssignmentUserTacklingSubmissionListFragment;

  getColumns(columns: ContestProblemAssignmentUserTacklingSubmissionListColumnFragment[]): ColDef[] {
    return [
      {
        colId: 'createdAt',
        field: 'createdAt',
      },
      ...columns.map(
        (c, i): ColDef => ({
          colId: `custom.${i}`,
          headerName: c.title.variant,
          // FIXME: generated types should not allow undefineds
          // tslint:disable-next-line: no-null-undefined-union
          valueGetter: ({ data }) => {
            const submission = data as ContestProblemAssignmentUserTacklingSubmissionListSubmissionFragment;
            const f = submission.summaryRow.fields[i];

            if (c.__typename === 'FulfillmentColumn' && f.__typename === 'FulfillmentField') return f.fulfilled;
            if (c.__typename === 'ScoreColumn' && f.__typename === 'ScoreField') return f.score;

            throw new Error(`Invalid or unsupported field ${f.__typename} in column ${c.__typename}`);
          },
        }),
      ),
    ];
  }
}

export const contestProblemAssignmentUserTacklingSubmissionListFragment = gql`
  fragment ContestProblemAssignmentUserTacklingSubmissionList on ContestProblemAssignmentUserTackling {
    submissions {
      ...ContestProblemAssignmentUserTacklingSubmissionListSubmission
    }

    assignmentView {
      assignment {
        problem {
          submissionListColumns {
            ...ContestProblemAssignmentUserTacklingSubmissionListColumn
          }
        }
      }
    }
  }

  fragment ContestProblemAssignmentUserTacklingSubmissionListColumn on Column {
    __typename
    ... on TitledColumn {
      title {
        ...Text
      }
    }
  }

  fragment ContestProblemAssignmentUserTacklingSubmissionListSubmission on Submission {
    id
    createdAt
    # TODO: submission files
    officialEvaluation {
      status
    }
    summaryRow {
      fields {
        ... on FulfillmentField {
          fulfilled
        }
        ... on ScoreField {
          score
        }
      }
    }

    ...SubmissionModal
  }

  ${submissionModalFragment}
  ${textFragment}
`;
